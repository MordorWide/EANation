use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "Config")]
pub struct Model {
    #[sea_orm(primary_key, column_name = "id")]
    pub id: i64,
    #[sea_orm(unique, column_name = "key")]
    pub key: String,
    #[sea_orm(column_name = "value")]
    pub value: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
