use sea_orm::{
    sea_query::{Table, TableCreateStatement},
    ConnectionTrait, DatabaseBackend, DatabaseConnection, DbErr, EntityTrait, Schema, Statement,
    TransactionTrait,
};

pub mod commodity;
pub mod consumer;
pub mod evaluation;
pub mod inventory;
pub mod order;

async fn _schema_setup<E>(db: &DatabaseConnection, entity: E) -> Result<(), DbErr>
where
    E: EntityTrait,
{
    let schema = Schema::new(db.get_database_backend());
    let create_stmt = schema.create_table_from_entity(entity);
    let mut drop_stmt = Table::drop();
    drop_stmt
        .if_exists()
        .table(create_stmt.get_table_name().expect("msg").clone());

    let drop_stmt = db.get_database_backend().build(&drop_stmt);
    let create_stmt = db.get_database_backend().build(&create_stmt);

    db.execute(drop_stmt).await?;

    if db.get_database_backend() == DatabaseBackend::Postgres {
        // for crdb
        let txn = db.begin().await?;
        let serial_normalization = Statement::from_string(
            DatabaseBackend::Postgres,
            "set local serial_normalization = sql_sequence;".to_owned(),
        );
        txn.execute(serial_normalization).await?;
        txn.execute(create_stmt).await?;
        txn.commit().await?;
    } else {
        // for other db
        db.execute(create_stmt).await?;
    }
    Ok(())
}

pub async fn schema_setup(db: &DatabaseConnection) -> Result<(), DbErr> {
    _schema_setup(db, commodity::Entity).await?;
    println!("commodity schema created");
    _schema_setup(db, consumer::Entity).await?;
    println!("consumer schema created");
    _schema_setup(db, evaluation::Entity).await?;
    println!("evaluation schema created");
    _schema_setup(db, inventory::Entity).await?;
    println!("inventory schema created");
    _schema_setup(db, order::Entity).await?;
    println!("order schema created");
    Ok(())
}
