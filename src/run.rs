use crate::{
    entity::{commodity, consumer, evaluation, inventory, order},
    rand::rand_i64,
};
use anyhow::Result;
use chrono::Local;
use flume::{Receiver, Sender};
use futures::{future::join_all, Future};
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait, Set,
    TransactionTrait,
};
use std::time::Instant;
use std::{pin::Pin, time::Duration};
use tokio::time::sleep;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    commodity_count: u32,
    consumer_count: u32,
    concurrent: u32,
    downgrade: bool,
    rate_limit: u32,
}

impl From<&super::Args> for Config {
    fn from(args: &super::Args) -> Self {
        let (commodity_count, consumer_count, downgrade, rate_limit) = match args.command {
            crate::SubCommandArgs::Prepare { .. } => unreachable!(),
            crate::SubCommandArgs::Run {
                commodity_count,
                consumer_count,
                downgrade,
                rate_limit,
            } => (commodity_count, consumer_count, downgrade, rate_limit),
        };
        Self {
            commodity_count,
            consumer_count,
            concurrent: args.concurrent,
            downgrade,
            rate_limit,
        }
    }
}

const TOKEN_NUMBER_PRE_SECOND: u32 = 50;

pub async fn execute<T: Into<Config>>(db: &DatabaseConnection, config: T) -> Result<()> {
    let config = config.into();
    let (token_tx, token_rx) = flume::bounded(10);
    let (martix_tx, martix_rx) = flume::unbounded();
    let token_generator_handle = token_generator(token_tx, config.rate_limit);
    let evaluation_service_handle =
        evaluation_service(db, token_rx.clone(), martix_tx.clone(), config);
    let martix_service_handle = martix_service(martix_rx);
    if config.downgrade {
        println!("Running with downgrade mode");
        std::mem::drop(martix_tx);
        tokio::join!(
            token_generator_handle,
            evaluation_service_handle,
            martix_service_handle
        )
        .1?;
        return Ok(());
    }
    println!("Running with normal mode");
    let orders_service_handle = orders_service(db, token_rx.clone(), martix_tx.clone(), config);
    let change_price_service_handle = change_price_service(db, token_rx, martix_tx, config);
    tokio::join!(
        token_generator_handle,
        orders_service_handle,
        evaluation_service_handle,
        change_price_service_handle,
        martix_service_handle
    )
    .1?;
    Ok(())
}

async fn token_generator(token_tx: Sender<u32>, rate_limit: u32) {
    let (exit_tx, exit_rx) = flume::bounded(1);
    ctrlc::set_handler(move || {
        exit_tx.send(()).expect("Failed to send exit signal");
    })
    .expect("Error setting Ctrl-C handler");
    let mut rate_unit = rate_limit / TOKEN_NUMBER_PRE_SECOND;
    if rate_unit == 0 {
        rate_unit += 1;
    }
    tokio::spawn(async move {
        loop {
            if let Ok(_) = exit_rx.try_recv() {
                println!("receive the exit signal, exit...");
                return;
            }
            token_tx.send_timeout(rate_unit, Duration::from_millis(50));
            sleep(Duration::from_millis(1000 / TOKEN_NUMBER_PRE_SECOND as u64)).await;
        }
    });
}

async fn orders_service(
    db: &DatabaseConnection,
    token_rx: Receiver<u32>,
    martix_tx: Sender<(u32, u32)>,
    config: Config,
) -> Result<()> {
    run_service(db, token_rx, martix_tx, config.concurrent, move |txn| {
        Box::pin(async move {
            let consumer_id = rand_i64(1, config.consumer_count as i64);
            let commodity_id = rand_i64(1, config.commodity_count as i64);
            let commodity = match commodity::Entity::find_by_id(commodity_id).one(txn).await? {
                Some(e) => e,
                None => {
                    println!(
                        "[WARN] Can't find the commodity({}), retrying.",
                        commodity_id
                    );
                    return Ok(0);
                }
            };
            let inventory = inventory::Entity::find_by_id(commodity_id)
                .one(txn)
                .await?
                .expect("Can't find the inventory");
            if inventory.inventory <= 0 {
                return Ok(0);
            }
            consumer::Entity::find_by_id(consumer_id)
                .one(txn)
                .await?
                .expect("Can't find the consumer");
            let inventory_number = inventory.inventory;
            let mut sold_number = rand_i64(1, 5);
            if inventory_number < sold_number {
                sold_number = inventory_number;
            }
            let mut inventory_active: inventory::ActiveModel = inventory.into();
            inventory_active.updated_at = Set(Local::now().naive_local());
            inventory_active.inventory = Set(inventory_number - sold_number);
            inventory_active.update(txn).await?;

            let mut order_active = order::ActiveModel::new();
            order_active.consumer_id = Set(consumer_id);
            order_active.commodity_id = Set(commodity_id);
            order_active.sold_uint_price = Set(commodity.price);
            order_active.sold_number = Set(sold_number);
            order_active.insert(txn).await?;
            Ok(2)
        })
    })
    .await?;
    Ok(())
}

async fn evaluation_service(
    db: &DatabaseConnection,
    token_rx: Receiver<u32>,
    martix_tx: Sender<(u32, u32)>,
    config: Config,
) -> Result<()> {
    run_service(db, token_rx, martix_tx, config.concurrent, move |txn| {
        Box::pin(async move {
            let consumer_id = rand_i64(1, config.consumer_count as i64);
            let commodity_id = rand_i64(1, config.commodity_count as i64);
            commodity::Entity::find_by_id(commodity_id)
                .one(txn)
                .await?
                .expect("Can't find the commodity");
            consumer::Entity::find_by_id(consumer_id)
                .one(txn)
                .await?
                .expect("Can't find the consumer");
            evaluation::ActiveModel::rand_fake_new(consumer_id, commodity_id)
                .insert(txn)
                .await?;
            Ok(1)
        })
    })
    .await?;
    Ok(())
}

async fn change_price_service(
    db: &DatabaseConnection,
    token_rx: Receiver<u32>,
    martix_tx: Sender<(u32, u32)>,
    config: Config,
) -> Result<()> {
    run_service(db, token_rx, martix_tx, config.concurrent, move |txn| {
        Box::pin(async move {
            let commodity_id = rand_i64(1, config.commodity_count as i64);
            let mut commodity: commodity::ActiveModel = commodity::Entity::find_by_id(commodity_id)
                .one(txn)
                .await?
                .expect("Can't find the commodity")
                .into();
            commodity.price = Set(rand_i64(1, 1000));
            commodity.updated_at = Set(Local::now().naive_local());
            commodity.update(txn).await?;
            Ok(1)
        })
    })
    .await?;
    Ok(())
}

async fn run_service<F>(
    db: &DatabaseConnection,
    token_rx: Receiver<u32>,
    martix_tx: Sender<(u32, u32)>,
    concurrent: u32,
    callback: F,
) -> Result<()>
where
    F: for<'c> Fn(
            &'c DatabaseTransaction,
        )
            -> Pin<Box<dyn Future<Output = std::result::Result<u32, DbErr>> + Send + 'c>>
        + Send
        + Sync
        + Copy
        + 'static,
{
    let mut join_handle_vec = Vec::new();
    for i in 0..concurrent {
        let db = db.clone();
        let token_rx = token_rx.clone();
        let martix_tx = martix_tx.clone();
        let handler = tokio::spawn(async move {
            while let Ok(mut token) = token_rx.recv_async().await {
                while token > 0 {
                    let now = Instant::now();
                    let result = db
                        .transaction::<_, u32, DbErr>(callback)
                        .await
                        .map_err(|err| match err {
                            sea_orm::TransactionError::Connection(err) => err,
                            sea_orm::TransactionError::Transaction(err) => err,
                        });
                    let elapsed = now.elapsed().as_millis() as u32;
                    match result {
                        Ok(changed_row) => {
                            if token >= changed_row {
                                token -= changed_row;
                            } else {
                                token = 0;
                            }
                            martix_tx.send((changed_row, elapsed));
                        }
                        Err(err) => return Err(err),
                    }
                }
            }
            Ok(())
        });
        join_handle_vec.push(handler);
    }
    let join_result = join_all(join_handle_vec).await;
    for handle in join_result {
        handle??;
    }
    Ok(())
}

async fn martix_service(martix_rx: Receiver<(u32, u32)>) {
    let mut now = Instant::now();
    let mut execute_time_vec = vec![];
    let mut changed_row_per_sec = 0;
    while let Ok((changed_row, execute_time)) = martix_rx.recv_async().await {
        execute_time_vec.push(execute_time);
        changed_row_per_sec += changed_row;
        if now.elapsed() > Duration::from_secs(1) {
            execute_time_vec.sort();
            let max = execute_time_vec[execute_time_vec.len() - 1];
            let p50 = execute_time_vec[(execute_time_vec.len() as f32 * 0.5) as usize];
            let p80 = execute_time_vec[(execute_time_vec.len() as f32 * 0.8) as usize];
            let p95 = execute_time_vec[(execute_time_vec.len() as f32 * 0.95) as usize];
            let p99 = execute_time_vec[(execute_time_vec.len() as f32 * 0.99) as usize];
            let p999 = execute_time_vec[(execute_time_vec.len() as f32 * 0.999) as usize];

            println!(
                "{} Txn Execute Time(P50:{}ms, P80:{}ms, P95:{}ms, P99:{}ms, P999:{}ms, Max:{}ms), {} Row/s",
                Local::now(), p50, p80, p95, p99, p999, max, changed_row_per_sec
            );
            now = Instant::now();
            changed_row_per_sec = 0;
            execute_time_vec.clear();
        }
    }
}
