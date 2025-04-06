use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "Account")]
pub struct Model {
    #[sea_orm(primary_key, column_name = "id")]
    pub id: i64,
    #[sea_orm(column_name = "email")]
    pub email: String,
    #[sea_orm(column_name = "password_hashed")]
    pub password_hashed: String,

    #[sea_orm(column_name = "lobby_key")]
    pub lobby_key: String,

    // Only for Django compatibility
    #[sea_orm(column_name = "is_staff")]
    pub is_staff: bool,
    #[sea_orm(column_name = "is_superuser")]
    pub is_superuser: bool,
    #[sea_orm(column_name = "is_verified")]
    pub is_verified: bool,

    #[sea_orm(column_name = "created_at")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[sea_orm(column_name = "last_login")]
    pub last_login: chrono::DateTime<chrono::Utc>,

    #[sea_orm(column_name = "force_client_turn")]
    pub force_client_turn: bool,
    #[sea_orm(column_name = "force_server_turn")]
    pub force_server_turn: bool,
    #[sea_orm(column_name = "name_mod_ping_site")]
    pub name_mod_ping_site: String,

    #[sea_orm(column_name = "optin_global")]
    pub optin_global: bool,
    #[sea_orm(column_name = "optin_thirdparty")]
    pub optin_thirdparty: bool,
    #[sea_orm(column_name = "parental_email")]
    pub parental_email: String,
    #[sea_orm(column_name = "birthdate")]
    pub birthdate: chrono::NaiveDate,

    #[sea_orm(column_name = "zipcode")]
    pub zipcode: String,
    #[sea_orm(column_name = "country")]
    pub country: String,
    #[sea_orm(column_name = "language")]
    pub language: String,
    #[sea_orm(column_name = "accepted_tos")]
    pub accepted_tos: String,
    #[sea_orm(column_name = "entitlement_key")]
    pub entitlement_key: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
