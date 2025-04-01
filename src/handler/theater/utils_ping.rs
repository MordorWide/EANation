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
