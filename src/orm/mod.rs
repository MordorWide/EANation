pub mod model;
use model::{account, ban, config, game, participant, persona, session};
use sea_orm::entity::prelude::*;
use sea_orm::entity::*;
use sea_orm::{DbBackend, DbErr, Schema};

pub fn build_database_conn_string(
    proto: &String,
    name: &String,
    user: &String,
    password: &String,
    host: &String,
    port: &String,
    params: &String,
) -> String {
    let mut conn_string = format!("{}://", proto);
    if user.len() > 0 {
        conn_string.push_str(user);
        if password.len() > 0 {
            conn_string.push_str(":");
            conn_string.push_str(password);
        }
        conn_string.push_str("@");
    }
    if host.len() > 0 {
        conn_string.push_str(host);
        if port.len() > 0 {
            conn_string.push_str(":");
            conn_string.push_str(port);
        }
    }
    if proto != "sqlite" {
        // Only add slash if a host is present.
        // sqlite does not connect to a host.
        conn_string.push_str("/");
    }
    conn_string.push_str(name);
    if params.len() > 0 {
        conn_string.push_str("?");
        conn_string.push_str(params);
    }
    conn_string
}

pub async fn create_tables(db: &DbConn) {
    // Setup Schema helper
    let schema = Schema::new(DbBackend::Sqlite);

    // Setup table Session
    if let Err(_) = db
        .execute(
            db.get_database_backend()
                .build(&schema.create_table_from_entity(session::Entity)),
        )
        .await
    {
        println!("Unable to create a new table InGameSession. The table probably already exists.");
    }

    // Setup table Account
    if let Err(_) = db
        .execute(
            db.get_database_backend()
                .build(&schema.create_table_from_entity(account::Entity)),
        )
        .await
    {
        println!("Unable to create a new table Account. The table probably already exists.");
    }

    // Setup table Persona
    if let Err(_) = db
        .execute(
            db.get_database_backend()
                .build(&schema.create_table_from_entity(persona::Entity)),
        )
        .await
    {
        println!("Unable to create a new table Persona. The table probably already exists.");
    }

    // Setup table Game
    if let Err(_) = db
        .execute(
            db.get_database_backend()
                .build(&schema.create_table_from_entity(game::Entity)),
        )
        .await
    {
        println!("Unable to create a new table Game. The table probably already exists.");
    }

    // Setup table Participant
    if let Err(_) = db
        .execute(
            db.get_database_backend()
                .build(&schema.create_table_from_entity(participant::Entity)),
        )
        .await
    {
        println!("Unable to create a new table Participant. The table probably already exists.");
    }

    // Setup table Ban
    if let Err(_) = db
        .execute(
            db.get_database_backend()
                .build(&schema.create_table_from_entity(ban::Entity)),
        )
        .await
    {
        println!("Unable to create a new table Ban. The table probably already exists.");
    }

    // Setup table Config + defaults
    if let Err(_) = db
        .execute(
            db.get_database_backend()
                .build(&schema.create_table_from_entity(config::Entity)),
        )
        .await
    {
        println!("Unable to create a new table Config. The table probably already exists.");
    }
}

pub async fn clear_old_db_data(db: &DbConn) {
    // Clear old ingamesessions
    if let Err(_) = session::Entity::delete_many().exec(&*db).await {
        println!("Failed to clear data in table InGameSession");
    }
    // Clear old participants
    if let Err(_) = participant::Entity::delete_many().exec(&*db).await {
        println!("Failed to clear data in table participant");
    }
    // Clear old games
    if let Err(_) = game::Entity::delete_many().exec(&*db).await {
        println!("Failed to clear data in table game");
    }
}

pub async fn add_default_configuration_keys(db: &DbConn) {
    // Add TOS_VERSION
    if let Ok(None) = config::Entity::find()
        .filter(config::Column::Key.eq("TOS_VERSION"))
        .one(&*db)
        .await
    {
        let tos_version_entry = config::ActiveModel {
            key: Set("TOS_VERSION".to_string()),
            value: Set("1.0".to_string()),
            ..Default::default()
        };
        let db_tos_version = tos_version_entry.insert(&*db).await.unwrap();
    }
    // Add TOS_TEXT_US
    if let Ok(None) = config::Entity::find()
        .filter(config::Column::Key.eq("TOS_TEXT_US"))
        .one(&*db)
        .await
    {
        let tos_text_us_entry = config::ActiveModel {
            key: Set("TOS_TEXT_US".to_string()),
            value: Set(
                "Welcome to the community-hosted Lord of the Rings: Conquest Server.".to_string(),
            ),
            ..Default::default()
        };
        let db_tos_text_us = tos_text_us_entry.insert(&*db).await.unwrap();
    }
    // Add ENABLE_TOS_CHECK
    if let Ok(None) = config::Entity::find()
        .filter(config::Column::Key.eq("ENABLE_TOS_CHECK"))
        .one(&*db)
        .await
    {
        let enable_tos_check_entry = config::ActiveModel {
            key: Set("ENABLE_TOS_CHECK".to_string()),
            value: Set("1".to_string()),
            ..Default::default()
        };
        let db_enable_tos_check = enable_tos_check_entry.insert(&*db).await.unwrap();
    }
    // Add ENABLE_ENTITLEMENT
    if let Ok(None) = config::Entity::find()
        .filter(config::Column::Key.eq("ENABLE_ENTITLEMENT"))
        .one(&*db)
        .await
    {
        let enable_entitlement_entry = config::ActiveModel {
            key: Set("ENABLE_ENTITLEMENT".to_string()),
            value: Set("1".to_string()),
            ..Default::default()
        };
        let db_enable_entitlement = enable_entitlement_entry.insert(&*db).await.unwrap();
    }
    // Add ENABLE_SHARED_ENTITLEMENT
    if let Ok(None) = config::Entity::find()
        .filter(config::Column::Key.eq("ENABLE_SHARED_ENTITLEMENT"))
        .one(&*db)
        .await
    {
        let enable_shared_entitlement_entry = config::ActiveModel {
            key: Set("ENABLE_SHARED_ENTITLEMENT".to_string()),
            value: Set("1".to_string()),
            ..Default::default()
        };
        let db_enable_shared_entitlement =
            enable_shared_entitlement_entry.insert(&*db).await.unwrap();
    }
    // Add MAX_PERSONAS
    if let Ok(None) = config::Entity::find()
        .filter(config::Column::Key.eq("MAX_PERSONAS"))
        .one(&*db)
        .await
    {
        let max_personas_entry = config::ActiveModel {
            key: Set("MAX_PERSONAS".to_string()),
            value: Set("5".to_string()),
            ..Default::default()
        };
        let db_max_personas = max_personas_entry.insert(&*db).await.unwrap();
    }

    // Add GetPingSites_minPingSitesToPing
    if let Ok(None) = config::Entity::find()
        .filter(config::Column::Key.eq("GetPingSites_minPingSitesToPing"))
        .one(&*db)
        .await
    {
        let min_ping_sites_to_ping_entry = config::ActiveModel {
            key: Set("GetPingSites_minPingSitesToPing".to_string()),
            value: Set("0".to_string()),
            ..Default::default()
        };
        let db_min_ping_sites_to_ping = min_ping_sites_to_ping_entry.insert(&*db).await.unwrap();
    }

     // Add GetPingSites_minPingSitesToPing
     if let Ok(None) = config::Entity::find()
        .filter(config::Column::Key.eq("GetPingSites_PingSites"))
        .one(&*db)
        .await
    {
        // Build JSON string from configuration object
        // The sites will be pinged using ICMP echo requests
        // The connecting games and clients will report their ping times to these sites
        // using UGAM commands with key:
        // 'B-U-PingSite' = '<ping site name>'
        // 'B-U-<ping site name>' = '<ping time>'
        let default_ping_sites = serde_json::json!([{
            "name": "ping0",
            "addr": "theater.mordorwi.de",
            "type": "0", // Should be set to 0!
        }]);

        let ping_sites_entry = config::ActiveModel {
            key: Set("GetPingSites_PingSites".to_string()),
            value: Set(default_ping_sites.to_string()),
            ..Default::default()
        };
        let db_ping_sites = ping_sites_entry.insert(&*db).await.unwrap();
    }
}

pub async fn check_config_table_exists(db: &DbConn) -> Result<bool, DbErr> {
    match config::Entity::find().one(db).await {
        Ok(_) => Ok(true),
        Err(DbErr::Exec(_)) => Ok(false),
        Err(err) => Err(err),
    }
}
