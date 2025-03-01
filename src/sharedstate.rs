use dashmap::DashMap;
use sea_orm::{Database, DatabaseConnection};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;

use crate::client_connection::{ClientConnection, ClientConnectionDescriptor};
use crate::orm::{
    add_default_configuration_keys, check_config_table_exists, clear_old_db_data, create_tables,
};
use crate::utils::stun_turn::{STUNInfo, TURNInfo};

#[derive(Debug, Clone)]
pub struct SharedState {
    pub database: Arc<DatabaseConnection>,
    pub connections: Arc<DashMap<ClientConnectionDescriptor, ClientConnection>>,
    pub udp_sockets: Arc<DashMap<u16, Arc<UdpSocket>>>,
    pub rng: Arc<RwLock<rand::rngs::OsRng>>,
    pub server_secret: String,
    pub stunrelay: Arc<STUNInfo>,
    pub turn: Arc<TURNInfo>,
}

impl SharedState {
    pub async fn new(
        db_connection_str: String,
        server_secret: String,
        init_schemas: bool,
        set_default_values: bool,
        stunrelay: STUNInfo,
        turn: TURNInfo,
    ) -> Self {
        let db = Database::connect(db_connection_str).await.unwrap();
        let rng = rand::rngs::OsRng::default();

        // Init DB
        if init_schemas {
            let _ = create_tables(&db).await;
        }

        // Wait till the Config table is created...
        loop {
            if let Ok(true) = check_config_table_exists(&db).await {
                break;
            }
            println!("Waiting for Config table to be created...");
            // Wait for 5 seconds before checking again.
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }

        // Set default values
        if set_default_values {
            let _ = add_default_configuration_keys(&db).await;
        }

        // Clear old session-related data
        clear_old_db_data(&db).await;

        Self {
            database: Arc::new(db),
            connections: Arc::new(DashMap::new()),
            udp_sockets: Arc::new(DashMap::new()),
            rng: Arc::new(RwLock::new(rng)),
            server_secret: server_secret,
            stunrelay: Arc::new(stunrelay),
            turn: Arc::new(turn),
        }
    }
}
