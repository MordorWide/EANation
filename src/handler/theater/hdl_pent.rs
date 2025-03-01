use indexmap::IndexMap;
use sea_orm::entity::*;
use sea_orm::query::*;

use crate::handler::{submit_packet, to_error_packet};
use crate::orm::model::{game, participant, session};
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_errors::EAError;
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::theater::TheaterHandler;


pub async fn handle_rq_pent(
    fh: &TheaterHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    // Player Enter (notified by the game host)
    // {"PID": "1", "TID": "7"} }
    let pid = prq.packet.data.get("PID").unwrap();
    let tid = prq.packet.data.get("TID").unwrap();
    let gid = prq.packet.data.get("GID").unwrap();

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
    // Lookup GID game in the database
    let Ok(Some(db_game)) = game::Entity::find_by_id(gid_int)
        .one(&*prq.sstate.database)
        .await
    else {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Game not found");
    };
    // Search for participant entry
    let Ok(Some(db_participant)) = participant::Entity::find()
        .filter(
            Condition::all()
                .add(participant::Column::PersonaId.eq(client_persona_id))
                .add(participant::Column::GameId.eq(gid_int)),
        )
        // .filter(participant::Column::GameId.eq(gid.parse::<i64>().unwrap()))
        .one(&*prq.sstate.database)
        .await
    else {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Participant of game not found.");
    };

    // Set player entry as active (queue_len = -1)
    let mut db_participant = db_participant.into_active_model();
    db_participant.queue_pos = Set(-1 as i32);
    let Ok(db_participant) = db_participant.update(&*prq.sstate.database).await else {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Failed to set participant as active player.");
    };

    // Get session of client
    let Ok(Some(db_client_session)) = session::Entity::find()
        .filter(session::Column::PersonaId.eq(db_participant.persona_id))
        .one(&*prq.sstate.database)
        .await
    else {
        return Err("Session not found");
    };

    // Get connection of client
    //let client_con_descr = ClientConnectionDescriptor::from_string(&db_client_session.theater_tcp_handle);

    let mut response_hm = IndexMap::new();
    response_hm.insert("TID".to_string(), tid.to_string());
    response_hm.insert("PID".to_string(), pid.to_string());

    let response_packet = DataPacket {
        packet_mode: PacketMode::FeslPingOrTheaterResponse,
        mode: DataMode::THEATER_PENT,
        packet_id: 0,
        data: response_hm,
    };

    submit_packet(response_packet, &prq.con, &prq.sstate, 0).await;

    Ok(())
}
