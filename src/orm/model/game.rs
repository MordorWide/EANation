use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "Game")]
pub struct Model {
    #[sea_orm(primary_key, column_name = "id")]
    pub id: i64,
    #[sea_orm(column_name = "lobby_id")]
    pub lobby_id: i32,
    #[sea_orm(column_name = "reserve_host")]
    pub reserve_host: bool,
    #[sea_orm(column_name = "name")]
    pub name: String,
    #[sea_orm(column_name = "persona_id")]
    pub persona_id: i64,
    #[sea_orm(column_name = "port")]
    pub port: i32, // Don't use u16, as u16 is not supported well by sqlx-postgres
    #[sea_orm(column_name = "host_type")]
    pub host_type: String,
    #[sea_orm(column_name = "game_type")]
    pub game_type: String,
    #[sea_orm(column_name = "queue_length")]
    pub queue_length: i32,
    #[sea_orm(column_name = "disable_autodequeue")]
    pub disable_autodequeue: bool,
    #[sea_orm(column_name = "hxfr")]
    pub hxfr: String,
    #[sea_orm(column_name = "internal_port")]
    pub internal_port: i32, // Don't use u16, as u16 is not supported well by sqlx-postgres
    #[sea_orm(column_name = "internal_ip")]
    pub internal_ip: String,
    #[sea_orm(column_name = "max_players")]
    pub max_players: i32,
    #[sea_orm(column_name = "max_observers")]
    pub max_observers: i32,
    #[sea_orm(column_name = "user_group_id")]
    pub user_group_id: String,
    #[sea_orm(column_name = "secret")]
    pub secret: String,
    #[sea_orm(column_name = "user_friends_only")]
    pub user_friends_only: bool,
    #[sea_orm(column_name = "user_pcdedicated")]
    pub user_pcdedicated: bool,
    #[sea_orm(column_name = "user_dlc")]
    pub user_dlc: String,
    #[sea_orm(column_name = "user_playmode")]
    pub user_playmode: String,
    #[sea_orm(column_name = "user_ranked")]
    pub user_ranked: bool,
    #[sea_orm(column_name = "user_levelkey")]
    pub user_levelkey: String,
    #[sea_orm(column_name = "user_levelname")]
    pub user_levelname: String,
    #[sea_orm(column_name = "user_mode")]
    pub user_mode: String,
    #[sea_orm(column_name = "client_version")]
    pub client_version: String,
    #[sea_orm(column_name = "server_version")]
    pub server_version: String,
    #[sea_orm(column_name = "join_mode")]
    pub join_mode: String,
    #[sea_orm(column_name = "rt")]
    pub rt: String,
    #[sea_orm(column_name = "encryption_key")]
    pub encryption_key: String,
    #[sea_orm(column_name = "other_as_json")]
    pub other_as_json: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /*#[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::UserId",
        to = "super::account::Column::Id"
    )]
    Account,*/
}
/*
impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
} */

impl ActiveModelBehavior for ActiveModel {}
