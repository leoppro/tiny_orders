use sea_orm::{
    sea_query::{Table, TableCreateStatement},
    ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, Schema,
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
    let create_stmt: TableCreateStatement = schema.create_table_from_entity(entity);
    let mut drop_stmt = Table::drop();
    drop_stmt
        .if_exists()
        .table(create_stmt.get_table_name().expect("msg").clone());
    db.execute(db.get_database_backend().build(&drop_stmt))
        .await?;
    db.execute(db.get_database_backend().build(&create_stmt))
        .await?;
    Ok(())
}

pub async fn schema_setup(db: &DatabaseConnection) -> Result<(), DbErr> {
    _schema_setup(db, commodity::Entity).await?;
    _schema_setup(db, consumer::Entity).await?;
    _schema_setup(db, evaluation::Entity).await?;
    _schema_setup(db, inventory::Entity).await?;
    _schema_setup(db, order::Entity).await?;
    Ok(())
}
