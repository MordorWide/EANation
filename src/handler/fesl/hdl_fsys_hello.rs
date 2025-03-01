use chrono::Utc;
use indexmap::IndexMap;

use crate::handler::submit_packet;
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::fesl::FeslHandler;


pub const LOTRCQ_DOMAIN: &str = "eadm";
pub const LOTRCQ_SUBDOMAIN: &str = "eadm";

pub async fn fsys_hello(
    fh: &FeslHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    let clientType = prq.packet.data.get("clientType").cloned(); // Either "" => Client, or "server" => Server

    let mut response_hm = IndexMap::new();

    let theaterIp = "theater.mordorwi.de";
    let theaterPort = "18885";
    let messengerIp = "messenger.mordorwi.de";
    let messengerPort = "0";

    // Build response prq.packet payload
    response_hm.insert("TXN".to_string(), "Hello".to_string());
    response_hm.insert(
        "curTime".to_string(),
        Utc::now().format("%h-%d-%Y %H:%M:%S UTC").to_string(),
    );

    response_hm.insert("activityTimeoutSecs".to_string(), "0".to_string());
    response_hm.insert("messengerIp".to_string(), messengerIp.to_string());
    response_hm.insert("messengerPort".to_string(), messengerPort.to_string());
    response_hm.insert("theaterIp".to_string(), theaterIp.to_string());
    response_hm.insert("theaterPort".to_string(), theaterPort.to_string());

    let client_string_parts: Vec<&str> = prq
        .packet
        .data
        .get("clientString")
        .unwrap()
        .split('-')
        .collect();
    let (game_id, dev, platform) = match client_string_parts.as_slice() {
        [game_id, dev, platform] => (game_id, dev, platform),
        _ => return Err("Invalid clientString format"),
    };

    let domain = platform;
    let subdomain = game_id.to_uppercase();
    //let partition = format!("/{}/{}", platform, subdomain);
    response_hm.insert(
        "domainPartition.domain".to_string(),
        LOTRCQ_DOMAIN.to_string(),
    );
    response_hm.insert(
        "domainPartition.subDomain".to_string(),
        LOTRCQ_SUBDOMAIN.to_string(),
    );

    let response = DataPacket::new(
        DataMode::FESL_FSYS,
        PacketMode::FeslSinglePacketResponse,
        prq.packet.packet_id,
        response_hm,
    );

    // Enqueue the response
    submit_packet(response, &prq.con, &prq.sstate, 0).await;

    // Send memcheck request NOW
    let _ = fh.send_memcheck(&prq.con, &prq.sstate, 0).await;

    // Lets also enqueue the ping
    let _ = fh.send_ping(&prq.con, &prq.sstate, 0).await;

    Ok(())
}
