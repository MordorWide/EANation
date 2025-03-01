use crate::orm::model::config;
use sea_orm::entity::*;
use sea_orm::query::*;
use sea_orm::DatabaseConnection;

pub async fn get_cfg_value(key: &str, db: &DatabaseConnection) -> Option<String> {
    if let Ok(Some(db_cfg_entry)) = config::Entity::find()
        .filter(config::Column::Key.eq(key))
        .one(db)
        .await
    {
        return Some(db_cfg_entry.value);
    }
    None
}
