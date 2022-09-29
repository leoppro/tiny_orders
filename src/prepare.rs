use crate::entity::{commodity, consumer, inventory, schema_setup};
use anyhow::{Context, Result};
use futures::future::join_all;
use sea_orm::ActiveModelTrait;
use sea_orm::{DatabaseConnection, DatabaseTransaction, DbErr, Set, TransactionTrait};
use std::time::{Duration, Instant};
use std::{future::Future, pin::Pin};

pub struct Config {
    commodity_count: u32,
    consumer_count: u32,
    txn_size: u32,
    concurrent: u32,
}

impl From<&super::Args> for Config {
    fn from(args: &super::Args) -> Self {
        Self {
            commodity_count: match args.command {
                super::SubCommandArgs::Prepare {
                    commodity_count, ..
                } => commodity_count,
                super::SubCommandArgs::Run { .. } => unreachable!(),
            },
            consumer_count: match args.command {
                super::SubCommandArgs::Prepare { consumer_count, .. } => consumer_count,
                super::SubCommandArgs::Run { .. } => unreachable!(),
            },
            txn_size: args.txn_size,
            concurrent: args.concurrent,
        }
    }
}

pub async fn execute<T: Into<Config>>(db: &DatabaseConnection, config: T) -> Result<()> {
    let config = config.into();
    // create table
    schema_setup(db).await.context("Failed to setup schema")?;
    println!("Finished setup schema.");
    // insert rows
    insert_commodity(db, &config)
        .await
        .context("Failed to insert commodity")?;
    println!("Finished insert commodity.");
    insert_consumer(db, &config)
        .await
        .context("Failed to insert consumer")?;
    println!("Finished insert commodity.");
    Ok(())
}

async fn insert_commodity(db: &DatabaseConnection, config: &Config) -> Result<()> {
    batch_exec(
        db,
        config.commodity_count * 2,
        config.txn_size,
        config.concurrent,
        |txn| {
            Box::pin(async move {
                let commodity_inserted =
                    commodity::ActiveModel::rand_fake_new().insert(txn).await?;
                let mut inventory_active = inventory::ActiveModel::rand_fake_new();
                inventory_active.commodity_id = Set(commodity_inserted.id);
                inventory_active.updated_at = Set(commodity_inserted.created_at.clone());
                inventory_active.created_at = Set(commodity_inserted.created_at.clone());
                inventory_active.insert(txn).await?;
                Ok(2)
            })
        },
    )
    .await?;
    Ok(())
}

async fn insert_consumer(db: &DatabaseConnection, config: &Config) -> Result<()> {
    batch_exec(
        db,
        config.consumer_count,
        config.txn_size,
        config.concurrent,
        |txn| {
            Box::pin(async move {
                consumer::ActiveModel::rand_fake_new().insert(txn).await?;
                Ok(1)
            })
        },
    )
    .await?;
    Ok(())
}

async fn batch_exec<F>(
    db: &DatabaseConnection,
    count: u32,
    txn_size_limit: u32,
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
        let handle = tokio::spawn(async move {
            let mut unit_count = count / concurrent;
            if i == concurrent - 1 {
                unit_count += count - (unit_count * concurrent);
            }
            let mut rows = 0;
            let mut now = Instant::now();
            while unit_count > 0 {
                let mut txn_size = txn_size_limit;
                if unit_count < txn_size {
                    txn_size = unit_count;
                }
                unit_count -= txn_size;
                let result = db
                    .transaction::<_, u32, DbErr>(|txn| {
                        Box::pin(async move {
                            let mut rows = 0;
                            while rows < txn_size {
                                rows += callback(txn).await?;
                            }
                            Ok(rows)
                        })
                    })
                    .await;
                match result {
                    Ok(txn_rows) => rows += txn_rows,
                    Err(err) => return Err(err),
                }
                if now.elapsed() > Duration::from_secs(1) {
                    now = Instant::now();
                    println!("[thread {}] Insert rows:{}", i, rows);
                    rows = 0;
                }
            }
            if rows > 0 {
                println!("[thread {}] Insert rows:{}", i, rows);
            }
            Ok(())
        });
        join_handle_vec.push(handle);
    }
    let join_result = join_all(join_handle_vec).await;
    for handle in join_result {
        handle??;
    }
    Ok(())
}
