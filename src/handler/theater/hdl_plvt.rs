use indexmap::IndexMap;
use sea_orm::entity::*;
use sea_orm::query::*;

use crate::handler::{submit_packet, to_error_packet};
use crate::orm::model::participant;
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_errors::EAError;
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::theater::TheaterHandler;


pub async fn handle_rq_plvt(
    fh: &TheaterHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    // Player Leave (notified by the game host)
    // {"LID": "1", "GID": "6", "PID": "3", "TID": "90"} }

    let pid = prq.packet.data.get("PID").unwrap();
    let gid = prq.packet.data.get("GID").unwrap();
    let tid = prq.packet.data.get("TID").unwrap();

    // Parse PID
    let Ok(client_persona_id) = pid.parse::<i64>() else {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Client PID not parsable");
    };

    // Parse GID
    let Ok(gid_int) = gid.parse::<i64>() else {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Game ID not parsable");
    };

    // Search for participant entry
    // Note: The participant may be removed eariler, so don't throw an error here!
    if let Err(_) = participant::Entity::delete_many()
        .filter(
            Condition::all()
                .add(participant::Column::GameId.eq(gid_int))
                .add(participant::Column::PersonaId.eq(client_persona_id)),
        )
        .exec(&*prq.sstate.database)
        .await
    {
        return Err("Failed to remove the participant from the table.");
    };

    let mut response_hm = IndexMap::new();
    response_hm.insert("TID".to_string(), tid.to_string());

    let response_packet = DataPacket {
        packet_mode: PacketMode::FeslPingOrTheaterResponse,
        mode: DataMode::THEATER_PLVT,
        packet_id: 0,
        data: response_hm,
    };

    submit_packet(response_packet, &prq.con, &prq.sstate, 0).await;

    Ok(())
}
