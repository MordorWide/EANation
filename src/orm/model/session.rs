use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Default)]
#[sea_orm(table_name = "InGameSession")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    #[sea_orm(column_name = "lobby_key")]
    pub lobby_key: String,
    #[sea_orm(column_name = "user_id")]
    pub user_id: i64,
    #[sea_orm(column_name = "persona_id")]
    pub persona_id: i64,

    #[sea_orm(column_name = "fesl_tcp_handle")]
    pub fesl_tcp_handle: String,
    #[sea_orm(column_name = "theater_tcp_handle")]
    pub theater_tcp_handle: String,
    #[sea_orm(column_name = "theater_udp_handle")]
    pub theater_udp_handle: String,
    #[sea_orm(column_name = "nat_type")]
    pub nat_type: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
