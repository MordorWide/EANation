use indexmap::IndexMap;
use tracing::debug;

use crate::handler::submit_packet;
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::fesl::FeslHandler;


pub async fn pres_setpresencestatus(
    fh: &FeslHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    let status_show: String = prq.packet.data.get("status.show").unwrap().to_string(); // = 'disc'

    // Check if the status is 'disc'
    if status_show != "disc" {
        debug!(target: "fesl", "PRES/SetPresenceStatus - Unexpected status.show: {:?}", status_show);
        return Err("Unexpected status.show");
    }

    let mut response_hm: IndexMap<String, String> = IndexMap::new();
    response_hm.insert("TXN".to_string(), "SetPresenceStatus".to_string());

    let response = DataPacket::new(
        DataMode::FESL_PRES,
        PacketMode::FeslSinglePacketResponse,
        prq.packet.packet_id,
        response_hm,
    );

    submit_packet(response, &prq.con, &prq.sstate, 0).await;
    Ok(())
}
