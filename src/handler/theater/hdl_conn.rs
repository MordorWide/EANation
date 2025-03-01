use chrono::Utc;
use indexmap::IndexMap;

use crate::handler::submit_packet;
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::theater::TheaterHandler;


pub async fn handle_rq_conn(
    fh: &TheaterHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    //{"PROT": "2", "PROD": "lotr-pandemic-pc", "VERS": "1.0", "PLAT": "PC", "LOCALE": "de", "SDKVERSION": "4.3.6.0.0", "TID": "1"} }
    let tid = prq.packet.data.get("TID").unwrap();
    let prot = prq.packet.data.get("PROT").unwrap();
    let prod = prq.packet.data.get("PROD").unwrap();
    let vers = prq.packet.data.get("VERS").unwrap();
    let plat = prq.packet.data.get("PLAT").unwrap();
    let locale = prq.packet.data.get("LOCALE").unwrap();
    let sdkversion = prq.packet.data.get("SDKVERSION").unwrap();

    const ACTIVITY_TIMEOUT_SECS: u32 = 0;

    // Send the response
    let mut response_hm = IndexMap::new();
    response_hm.insert("TID".to_string(), tid.to_string());
    response_hm.insert("TIME".to_string(), Utc::now().timestamp().to_string());
    response_hm.insert(
        "activityTimeoutSecs".to_string(),
        ACTIVITY_TIMEOUT_SECS.to_string(),
    );
    response_hm.insert("PROT".to_string(), prot.to_string());

    let response_packet = DataPacket {
        packet_mode: PacketMode::FeslPingOrTheaterResponse,
        mode: DataMode::THEATER_CONN,
        packet_id: prq.packet.packet_id,
        data: response_hm,
    };

    // Enqueue the response
    submit_packet(response_packet, &prq.con, &prq.sstate, 0).await;

    // Enqueue ping as well
    let _ = fh.send_ping(&prq.con, &prq.sstate, 0 as i64).await;

    Ok(())
}
