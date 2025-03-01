use indexmap::IndexMap;

use crate::handler::submit_packet;
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::theater::TheaterHandler;


pub async fn handle_rq_ubra(
    fh: &TheaterHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    // Update Bracket
    // {"LID": "1", "GID": "23", "START": "1", "TID": "8"} }
    // Seems like UBRA requests a "lock" on the game info table to subsequently
    // perform changes via UGAM, and exits with UBRA with START=0 to release the lock.
    // Therefore, UGAM is a "response" packet (accoding to the packet mode)

    // ToDo: Implement useful locking mechanism...
    let lid = prq.packet.data.get("LID").unwrap();
    let gid = prq.packet.data.get("GID").unwrap();
    let start = prq.packet.data.get("START").unwrap();
    let tid = prq.packet.data.get("TID").unwrap();

    // Just plainly respond with the TID
    let mut response_hm = IndexMap::new();
    response_hm.insert("TID".to_string(), tid.to_string());

    let response_packet = DataPacket {
        packet_mode: PacketMode::FeslPingOrTheaterResponse,
        mode: DataMode::THEATER_UBRA,
        packet_id: 0,
        data: response_hm,
    };

    submit_packet(response_packet, &prq.con, &prq.sstate, 0).await;

    Ok(())
}
