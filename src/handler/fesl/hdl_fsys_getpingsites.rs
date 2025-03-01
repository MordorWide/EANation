use indexmap::IndexMap;

use crate::handler::submit_packet;
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::fesl::FeslHandler;

pub async fn fsys_getpingsites(
    fh: &FeslHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    const PING_SERVER_IP: &str = "theater.mordorwi.de"; // Point to the current server

    let mut response_hm: IndexMap<_, _, _> = IndexMap::new();
    response_hm.insert("TXN".to_string(), "GetPingSites".to_string());
    response_hm.insert("pingSites.[]".to_string(), "2".to_string());
    response_hm.insert("pingSites.0.addr".to_string(), PING_SERVER_IP.to_string());
    response_hm.insert("pingSites.0.type".to_string(), "0".to_string());
    response_hm.insert("pingSites.0.name".to_string(), "eu".to_string());

    response_hm.insert(
        "pingSites.1.addr".to_string(),
        "natneg.mordorwi.de".to_string(),
    );
    response_hm.insert("pingSites.1.type".to_string(), "0".to_string());
    response_hm.insert("pingSites.1.name".to_string(), "eu2".to_string());
    response_hm.insert("minPingSitesToPing".to_string(), "2".to_string());

    let response = DataPacket::new(
        DataMode::FESL_FSYS,
        PacketMode::FeslSinglePacketResponse,
        prq.packet.packet_id,
        response_hm,
    );

    // Enqueue the response
    submit_packet(response, &prq.con, &prq.sstate, 0).await;
    Ok(())
}
