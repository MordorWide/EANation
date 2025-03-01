use indexmap::IndexMap;

use crate::handler::submit_packet;
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::fesl::FeslHandler;


pub async fn fsys_goodbye(
    fh: &FeslHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    // Deactivate session...?
    let reason = prq
        .packet
        .data
        .get("reason")
        .unwrap_or(&"No reason given".to_string());
    let message = prq
        .packet
        .data
        .get("message")
        .unwrap_or(&"No message given".to_string());

    // Submit empty response
    let response = DataPacket::new(
        DataMode::FESL_FSYS,
        PacketMode::FeslSinglePacketResponse,
        prq.packet.packet_id,
        IndexMap::new(),
    );

    submit_packet(response, &prq.con, &prq.sstate, 0).await;
    Ok(())
}
