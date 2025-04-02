use indexmap::IndexMap;
use std::sync::Arc;

use crate::client_connection::ClientConnectionDescriptor;
use crate::handler::submit_packet;
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::sharedstate::SharedState;
use crate::handler::theater::TheaterHandler;


pub async fn send_ping(
    fh: &TheaterHandler,
    con: &ClientConnectionDescriptor,
    sstate: &Arc<SharedState>,
    delay: i64,
) -> Result<(), &'static str> {
    let mut request_hm = IndexMap::new();
    request_hm.insert("TXN".to_string(), "Ping".to_string());
    request_hm.insert("TID".to_string(), "0".to_string());

    let ping_request = DataPacket::new(
        DataMode::THEATER_PING,
        PacketMode::TheaterRequest,
        0,
        request_hm,
    );

    // We don't need to await the result here...
    submit_packet(ping_request, con, sstate, delay).await;
    Ok(())
}
