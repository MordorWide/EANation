mod client_connection;
mod config;
mod crypto;
mod handler;
mod listener;
mod mordorwide_errors;
mod orm;
mod packet;
mod plasma_errors;
mod plasma_handle;
mod service;
mod sharedstate;
mod utils;

use std::env;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;
use tracing::debug;

use crate::config::{
    CryptoConfig, ServiceConfig, TcpListenerConfig, UdpListenerConfig,
};
use crate::orm::build_database_conn_string;
use crate::service::Service;
use crate::sharedstate::SharedState;
use crate::utils::stun_turn::{STUNInfo, TURNInfo};

#[tokio::main]
async fn main() {
    // Setup tracing
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_env("MORDORWIDE_LOG")).init();

    // Load configuration
    dotenv::dotenv().ok();

    // Load DB configuration infos
    let DB_PROTO = env::var("DB_PROTO").unwrap_or("sqlite".to_string());
    let DB_NAME = env::var("DB_NAME").unwrap_or("db.sqlite".to_string());
    let DB_USER = env::var("DB_USER").unwrap_or("".to_string());
    let DB_PASSWORD = env::var("DB_PASSWORD").unwrap_or("".to_string());
    let DB_HOST = env::var("DB_HOST").unwrap_or("".to_string());
    let DB_PORT = env::var("DB_PORT").unwrap_or("".to_string());
    let DB_PARAMS = env::var("DB_PARAMS").unwrap_or("".to_string());

    let DB_CONN_STRING = build_database_conn_string(
        &DB_PROTO,
        &DB_NAME,
        &DB_USER,
        &DB_PASSWORD,
        &DB_HOST,
        &DB_PORT,
        &DB_PARAMS,
    );
    debug!(target:"args", "Database connection string: {}", DB_CONN_STRING);


    // Load other configuration infos
    let SERVER_SECRET =
        env::var("SECRET_KEY").unwrap_or("UNSAFE_SERVER_SECRET_123456789".to_string());
    let INIT_SCHEMAS = env::var("INIT_SCHEMAS").unwrap_or("1".to_string()) == "1";

    let PATH_PRIVATE_KEY = env::var("PATH_PRIVATE_KEY").unwrap_or("data/priv.pem".to_string());
    let PATH_PUBLIC_KEY = env::var("PATH_PUBLIC_KEY").unwrap_or("data/pub.pem".to_string());

    // STUNRelay Configuration
    let STUN_ENABLED = env::var("STUN_ENABLED").unwrap_or("0".to_string()) == "1";
    // STUNRelay Hostname or IP
    let STUN_RELAY_HOST = env::var("STUN_RELAY_HOST").unwrap_or("".to_string());
    let STUN_RELAY_PORT = env::var("STUN_RELAY_PORT")
        .unwrap_or("8001".to_string())
        .parse::<u16>()
        .unwrap();
    // STUNRelay Source Port to send packets
    let STUN_RELAY_SOURCE_PORT = env::var("STUN_RELAY_SOURCE_PORT")
        .unwrap_or("39999".to_string())
        .parse::<u16>()
        .unwrap();
    let STUN_INTERNAL_SOURCE_PORT = env::var("STUN_INTERNAL_SOURCE_PORT")
        .unwrap_or("39999".to_string())
        .parse::<u16>()
        .unwrap();

    // TURN Configuration
    let TURN_ENABLED = env::var("TURN_ENABLED").unwrap_or("0".to_string()) == "1";
    // TURN Hostname or IP to contact the control port
    let TURN_RELAY_INTERNAL_HOST = env::var("TURN_RELAY_INTERNAL_HOST").unwrap_or("".to_string());
    let TURN_RELAY_PORT = env::var("TURN_RELAY_PORT")
        .unwrap_or("8002".to_string())
        .parse::<u16>()
        .unwrap();
    // TURN IP to be sent to clients
    let TURN_RELAY_EXTERNAL_IP = env::var("TURN_RELAY_EXTERNAL_IP").unwrap_or("".to_string());

    // Create STUN and TURN objects
    let stun_info = STUNInfo {
        enabled: STUN_ENABLED,
        host: STUN_RELAY_HOST,
        port: STUN_RELAY_PORT,
        relay_source_port: STUN_RELAY_SOURCE_PORT,
        internal_source_port: STUN_INTERNAL_SOURCE_PORT,
    };

    let turn_info = TURNInfo {
        enabled: TURN_ENABLED,
        control_host: TURN_RELAY_INTERNAL_HOST,
        control_port: TURN_RELAY_PORT,
        external_ip: TURN_RELAY_EXTERNAL_IP,
    };

    let shared_state = Arc::new(
        SharedState::new(
            DB_CONN_STRING,
            SERVER_SECRET,
            INIT_SCHEMAS,
            true,
            stun_info,
            turn_info,
        )
        .await,
    );

    let mut handles = Vec::new();

    // Configure Fesl service
    let fesl_service_pc = ServiceConfig {
        service_type: "Fesl".to_string(),
        handler: "FeslHandler".to_string(),
        tcp_listeners: Some(vec![TcpListenerConfig {
            crypto: CryptoConfig {
                crypto_type: "tls".to_string(),
                priv_key: Some(PATH_PRIVATE_KEY.clone()),
                pub_key: Some(PATH_PUBLIC_KEY.clone()),
            },
            host: "0.0.0.0".to_string(),
            port: 18880,
        }]),
        udp_listeners: None,
    };
    let fesl_service_ps3 = ServiceConfig {
        service_type: "Fesl".to_string(),
        handler: "FeslHandler".to_string(),
        tcp_listeners: Some(vec![TcpListenerConfig {
            crypto: CryptoConfig {
                crypto_type: "tls".to_string(),
                priv_key: Some(PATH_PRIVATE_KEY.clone()),
                pub_key: Some(PATH_PUBLIC_KEY.clone()),
            },
            host: "0.0.0.0".to_string(),
            port: 18870,
        }]),
        udp_listeners: None,
    };
    let fesl_service_xbox360 = ServiceConfig {
        service_type: "Fesl".to_string(),
        handler: "FeslHandler".to_string(),
        tcp_listeners: Some(vec![TcpListenerConfig {
            // crypto: CryptoConfig {
            //     crypto_type: "tls".to_string(),
            //     priv_key: Some(PATH_PRIVATE_KEY.clone()),
            //     pub_key: Some(PATH_PUBLIC_KEY.clone()),
            // },
            crypto: CryptoConfig {
                crypto_type: "plain".to_string(),
                priv_key: None,
                pub_key: None,
            },
            host: "0.0.0.0".to_string(),
            port: 18860,
        }]),
        udp_listeners: None,
    };

    // Configure Theater service
    let theater_service = ServiceConfig {
        service_type: "Theater".to_string(),
        handler: "TheaterHandler".to_string(),
        tcp_listeners: Some(vec![TcpListenerConfig {
            crypto: CryptoConfig {
                crypto_type: "plain".to_string(),
                priv_key: None,
                pub_key: None,
            },
            host: "0.0.0.0".to_string(),
            port: 18885,
        }]),
        udp_listeners: Some(vec![UdpListenerConfig {
            host: "0.0.0.0".to_string(),
            port: 18885,
        }]),
    };

    handles.extend(Service::spawn(fesl_service_pc, shared_state.clone()).await);
    handles.extend(Service::spawn(fesl_service_ps3, shared_state.clone()).await);
    handles.extend(Service::spawn(fesl_service_xbox360, shared_state.clone()).await);
    handles.extend(Service::spawn(theater_service, shared_state.clone()).await);

    // Join handles...
    for handle in handles {
        let _ = handle.await;
    }
}
