use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "Participant")]
pub struct Model {
    #[sea_orm(primary_key, column_name = "id")]
    pub id: i64,
    #[sea_orm(column_name = "game_id")]
    pub game_id: i64,
    #[sea_orm(column_name = "persona_id")]
    pub persona_id: i64,
    #[sea_orm(column_name = "queue_pos")]
    pub queue_pos: i32,
    #[sea_orm(column_name = "ticket")]
    pub ticket: String,

    // Only for potential TURN mappings
    #[sea_orm(column_name = "client_expected_host_port")]
    pub client_expected_host_port: i32,
    #[sea_orm(column_name = "client_expected_host_ip")]
    pub client_expected_host_ip: String,

    #[sea_orm(column_name = "host_expected_client_port")]
    pub host_expected_client_port: i32,
    #[sea_orm(column_name = "host_expected_client_ip")]
    pub host_expected_client_ip: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
