use anyhow::Result;
use chrono::Local;
use fakeit::hipster;
use rand::{thread_rng, Rng};
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, ConnectionTrait, Schema, Set};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "tiny_orders_commodity")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: u32,
    pub title: String,
    pub price: u32,
    pub description: String,
    pub updated_at: DateTime,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn rand_fake_new() -> Self {
        let mut rng = thread_rng();
        let create_at = Local::now().naive_local();
        Self {
            id: NotSet,
            title: Set(hipster::sentence(5)),
            price: Set(rng.gen_range(1..1000)),
            description: Set(hipster::sentence(30)),
            updated_at: Set(create_at.clone()),
            created_at: Set(create_at),
        }
    }
}