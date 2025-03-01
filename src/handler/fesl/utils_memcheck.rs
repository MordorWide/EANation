use indexmap::IndexMap;
use rand::Rng;
use std::sync::Arc;

use crate::client_connection::ClientConnectionDescriptor;
use crate::handler::submit_packet;
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::sharedstate::SharedState;
use crate::handler::fesl::FeslHandler;


pub async fn send_memcheck(
    fh: &FeslHandler,
    con: &ClientConnectionDescriptor,
    sstate: &Arc<SharedState>,
    delay: i64,
) -> Result<(), &'static str> {
    let mut request_hm = IndexMap::new();

    // Get the shared random number generator
    let mut rng = sstate.rng.write().await;
    let salt: u64 = rng.gen_range(1..10);

    request_hm.insert("TXN".to_string(), "MemCheck".to_string());
    request_hm.insert("memcheck.[]".to_string(), "0".to_string());
    request_hm.insert("type".to_string(), "0".to_string());
    request_hm.insert("salt".to_string(), salt.to_string());

    let memcheck_request = DataPacket::new(
        DataMode::FESL_FSYS,
        PacketMode::FeslSinglePacketRequest,
        0,
        request_hm,
    );

    submit_packet(memcheck_request, con, sstate, delay).await;
    Ok(())
}
