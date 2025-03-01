pub mod fesl;
pub mod theater;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use indexmap::IndexMap;
use std::sync::Arc;
use tokio::net::UdpSocket;

use crate::client_connection::{ClientConnectionDescriptor, ProtoType, SendDataType, ServiceType};
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::sharedstate::SharedState;
use crate::utils::stun_turn::{
    StunRelayRequestBody, StunRelayResponseBody,
};

#[async_trait::async_trait]
pub trait Handler: Send + Sync {
    async fn handle_packet(
        &self,
        packet: DataPacket,
        con: ClientConnectionDescriptor,
        sstate: Arc<SharedState>,
    ) -> Result<(), &'static str>; // Handle incoming packet
    async fn connection_closed(&self, con: ClientConnectionDescriptor, sstate: Arc<SharedState>);
    fn handler_type(&self) -> ServiceType;
}

async fn submit_packet(
    packet: DataPacket,
    con: &ClientConnectionDescriptor,
    sstate: &Arc<SharedState>,
    delay: i64,
) {
    // Spawn new thread to delay the packet
    let delayed_sstate = sstate.clone();
    let delayed_con = con.clone();

    let _ = tokio::spawn(async move {
        if delay > 0 {
            tokio::time::sleep(tokio::time::Duration::from_secs(delay as u64)).await;
        }

        // Send the packet
        match delayed_con.proto_type {
            ProtoType::Tcp => {
                if let Some(client_con) = delayed_sstate.connections.get(&delayed_con) {
                    //println!("Sending packet to client: {:?}", &packet);
                    client_con.send(SendDataType::Data(packet)).await;
                } else {
                    println!("Client not found");
                }
            }
            ProtoType::Udp => {
                if let Some(udp_socket) = delayed_sstate.udp_sockets.get(&delayed_con.host_port) {
                    // Send packet to client
                    println!("[Server=>{}]: {:?}", delayed_con.to_string(), &packet);
                    let addr = format!("{}:{}", delayed_con.client_ip, delayed_con.client_port);
                    udp_socket.send_to(&packet.to_bytes(), addr).await.unwrap();
                } else {
                    // Bind to UDP socket for STUN on same host but with different port
                    let stunrelay = &delayed_sstate.stunrelay;
                    if stunrelay.enabled {
                        let Ok(udp_socket) =
                            UdpSocket::bind(("0.0.0.0", stunrelay.internal_source_port)).await
                        else {
                            // Failed to bind to socket
                            return;
                        };
                        println!(
                            "[STUNSameServer=>{}]: {:?}",
                            delayed_con.to_string(),
                            &packet
                        );
                        let addr = format!("{}:{}", delayed_con.client_ip, delayed_con.client_port);
                        udp_socket.send_to(&packet.to_bytes(), addr).await.unwrap();
                    }
                };
            }
            ProtoType::RemoteUdp => {
                let stunrelay = &delayed_sstate.stunrelay;
                if stunrelay.enabled {
                    // Transfer packet to STUNRelay
                    if stunrelay.relay_source_port == 0 {
                        // STUNRelay not configured properly
                        return;
                    }

                    println!(
                        "[STUNRelayServer=>{}]: {:?}",
                        delayed_con.to_string(),
                        &packet
                    );
                    let b64_data = STANDARD.encode(&packet.to_bytes());

                    // Prepare JSON payload
                    let payload = StunRelayRequestBody {
                        client_ip: delayed_con.client_ip.clone(),
                        client_port: delayed_con.client_port,
                        source_port: stunrelay.relay_source_port,
                        b64_payload: b64_data,
                    };

                    // Send packet to STUNRelay via POST request
                    let Ok(response) = reqwest::Client::new()
                        .post(&format!(
                            "http://{}:{}/send",
                            stunrelay.host, stunrelay.port
                        ))
                        .json(&payload)
                        .send()
                        .await
                    else {
                        // Failed to send packet to STUNRelay
                        return;
                    };

                    // Check response (not really required though...)
                    let Ok(response_body) = response.json::<StunRelayResponseBody>().await else {
                        // Failed to parse response body
                        return;
                    };

                    if !response_body.success {
                        // STUNRelay failed to send packet
                        return;
                    }
                } else {
                    // STUNRelay not enabled... Do nothing
                }
            }
        }
    });
}

fn to_error_packet(packet: &DataPacket, error_code: i32, error_text: Option<String>) -> DataPacket {
    let mut error_hm: IndexMap<String, String> = IndexMap::new();
    let mut packet_id = 0;
    match packet.mode {
        DataMode::FESL_FSYS
        | DataMode::FESL_PNOW
        | DataMode::FESL_ACCT
        | DataMode::FESL_RECP
        | DataMode::FESL_ASSO
        | DataMode::FESL_PRES
        | DataMode::FESL_RANK
        | DataMode::FESL_XMSG
        | DataMode::FESL_MTRX
            if packet.data.contains_key("TXN") =>
        {
            // FESL packet -> copy TXN and re-use packet ID
            error_hm.insert(
                "TXN".to_string(),
                packet.data.get("TXN").unwrap().to_string(),
            );
            packet_id = packet.packet_id;
        }
        _ => {}
    }

    if let Some(actual_text) = error_text {
        error_hm.insert("localizedMessage".to_string(), actual_text);
    } else {
        error_hm.insert(
            "localizedMessage".to_string(),
            format!("ErrorCode:{}", error_code).to_string(),
        );
    }

    error_hm.insert("errorCode".to_string(), error_code.to_string());
    error_hm.insert("errorContainer.[]".to_string(), "0".to_string());

    DataPacket {
        mode: packet.mode.clone(),
        packet_mode: match packet.packet_mode {
            PacketMode::FeslPingOrTheaterResponse => PacketMode::FeslPingOrTheaterResponse,
            PacketMode::FeslSinglePacketResponse => PacketMode::FeslSinglePacketResponse,
            PacketMode::FeslMultiPacketResponse => PacketMode::FeslMultiPacketResponse,
            PacketMode::FeslSinglePacketRequest => PacketMode::FeslSinglePacketResponse,
            PacketMode::FeslMultiPacketRequest => PacketMode::FeslMultiPacketResponse,
            PacketMode::TheaterRequest => PacketMode::FeslPingOrTheaterResponse,
        },
        packet_id: packet_id,
        data: error_hm,
    }
}
