use std::sync::Arc;
use tokio::task::JoinHandle;

use crate::config::ServiceConfig;
use crate::crypto::CryptoMode;
use crate::handler::fesl::FeslHandler;
use crate::handler::theater::TheaterHandler;
use crate::handler::Handler;
use crate::listener::Listener;
use crate::sharedstate::SharedState;

pub struct Service;

impl Service {
    pub async fn spawn(
        config: ServiceConfig,
        shared_state: Arc<SharedState>,
    ) -> Vec<JoinHandle<()>> {
        let handler: Arc<dyn Handler> = match config.handler.as_str() {
            "FeslHandler" => Arc::new(FeslHandler),
            "TheaterHandler" => Arc::new(TheaterHandler),
            _ => panic!("Unknown handler type"),
        };

        let mut handles = Vec::new();

        if let Some(tcp_listeners) = config.tcp_listeners {
            for listener in tcp_listeners {
                let crypto_mode = CryptoMode::from(listener.crypto);
                handles.push(
                    Listener::start_tcp(
                        &listener.host,
                        listener.port,
                        crypto_mode,
                        handler.clone(),
                        shared_state.clone(),
                    )
                    .await,
                );
            }
        }

        if let Some(udp_listeners) = config.udp_listeners {
            for listener in udp_listeners {
                handles.push(
                    Listener::start_udp(
                        &listener.host,
                        listener.port,
                        handler.clone(),
                        shared_state.clone(),
                    )
                    .await,
                );
            }
        }

        handles
    }
}
