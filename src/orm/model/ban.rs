use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "Ban")]
pub struct Model {
    #[sea_orm(primary_key, column_name = "id")]
    pub id: i64,
    #[sea_orm(column_name = "email_hash")]
    pub email_hash: String,
    #[sea_orm(column_name = "reason")]
    pub reason: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
