use chrono::Local;
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Set};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "tiny_orders_order")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub consumer_id: i64,
    pub commodity_id: i64,
    pub sold_uint_price: i64,
    pub sold_number: i64,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn new() -> Self {
        Self {
            id: NotSet,
            consumer_id: NotSet,
            commodity_id: NotSet,
            sold_uint_price: NotSet,
            sold_number: NotSet,
            created_at: Set(Local::now().naive_local()),
        }
    }
}
