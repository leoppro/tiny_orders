use chrono::Local;
use fakeit::hipster;
use sea_orm::{entity::prelude::*, Set};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "tiny_orders_evaluation")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub consumer_id: u32,
    #[sea_orm(primary_key)]
    pub commodity_id: u32,
    pub evaluation: String,
    pub updated_at: DateTime,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn rand_fake_new(consumer_id: u32, commodity_id: u32) -> Self {
        let create_at = Local::now().naive_local();
        Self {
            consumer_id: Set(consumer_id),
            commodity_id: Set(commodity_id),
            evaluation: Set(hipster::sentence(30)),
            updated_at: Set(create_at.clone()),
            created_at: Set(create_at),
        }
    }
}
