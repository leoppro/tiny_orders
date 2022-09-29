use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Set};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "tiny_orders_inventory")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub commodity_id: i64,
    pub inventory: i64,
    pub updated_at: DateTime,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn rand_fake_new() -> Self {
        Self {
            commodity_id: NotSet,
            inventory: Set(100000),
            updated_at: NotSet,
            created_at: NotSet,
        }
    }
}
