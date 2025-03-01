use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub services: Vec<ServiceConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ServiceConfig {
    pub service_type: String,
    pub handler: String,
    pub tcp_listeners: Option<Vec<TcpListenerConfig>>,
    pub udp_listeners: Option<Vec<UdpListenerConfig>>,
}

#[derive(Debug, Deserialize)]
pub struct TcpListenerConfig {
    pub host: String,
    pub port: u16,
    pub crypto: CryptoConfig,
}

#[derive(Debug, Deserialize)]
pub struct UdpListenerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct CryptoConfig {
    pub crypto_type: String,
    pub priv_key: Option<String>, // For TLS
    pub pub_key: Option<String>,  // For TLS
}
