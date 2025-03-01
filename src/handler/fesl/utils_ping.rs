use indexmap::IndexMap;
use std::sync::Arc;

use crate::client_connection::ClientConnectionDescriptor;
use crate::handler::submit_packet;
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::sharedstate::SharedState;
use crate::handler::fesl::FeslHandler;

pub async fn send_ping(
    fh: &FeslHandler,
    con: &ClientConnectionDescriptor,
    sstate: &Arc<SharedState>,
    delay: i64,
) -> Result<(), &'static str> {
    let mut request_hm = IndexMap::new();

    request_hm.insert("TXN".to_string(), "Ping".to_string());

    let ping_request = DataPacket::new(
        DataMode::FESL_FSYS,
        PacketMode::FeslSinglePacketRequest,
        0,
        request_hm,
    );

    submit_packet(ping_request, con, sstate, delay).await;
    Ok(())
}
