use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct STUNInfo {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub relay_source_port: u16,
    pub internal_source_port: u16,
}

#[derive(Serialize, Debug)]
pub struct StunRelayRequestBody {
    pub client_ip: String,
    pub client_port: u16,
    pub source_port: u16,
    pub b64_payload: String,
}

#[derive(Deserialize, Debug)]
pub struct StunRelayResponseBody {
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct TURNInfo {
    pub enabled: bool,
    pub control_host: String,
    pub control_port: u16,
    pub external_ip: String,
}

#[derive(Serialize, Debug)]
pub struct TurnRequestBody {
    pub client_ip_0: String,
    pub client_port_0: u16,
    pub client_ip_1: String,
    pub client_port_1: u16,
}

#[derive(Deserialize, Debug)]
pub struct TurnResponseBody {
    pub success: bool,
    pub relay_port_0: Option<u16>,
    pub relay_port_1: Option<u16>,
}
