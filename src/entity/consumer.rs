use chrono::Local;
use fakeit::name;
use rand::thread_rng;
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Set};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "tiny_orders_consumer")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: u32,
    pub name: String,
    pub updated_at: DateTime,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn rand_fake_new() -> Self {
        let create_at = Local::now().naive_local();
        Self {
            id: NotSet,
            name: Set(name::full()),
            updated_at: Set(create_at.clone()),
            created_at: Set(create_at),
        }
    }
}
