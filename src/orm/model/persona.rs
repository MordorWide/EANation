use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "Persona")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(column_name = "user_id")]
    pub user_id: i64,
    #[sea_orm(column_name = "name")]
    pub name: String,
    #[sea_orm(column_name = "allow_insecure_login")]
    pub allow_insecure_login: bool,
    #[sea_orm(column_name = "created_at")]
    pub created_at: chrono::DateTime<chrono::Utc>,
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
}*/

impl ActiveModelBehavior for ActiveModel {}
