use indexmap::IndexMap;
use sea_orm::entity::*;
use sea_orm::query::*;

use crate::handler::{submit_packet, to_error_packet};
use crate::orm::model::{game, participant, session};
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_errors::EAError;
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::theater::TheaterHandler;


pub async fn handle_rq_ecnl(
    fh: &TheaterHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    // Cancel Game Entry (notified by the client)
    let tid = prq.packet.data.get("TID").unwrap();
    let lid = prq.packet.data.get("LID").unwrap();
    let gid = prq.packet.data.get("GID").unwrap();

    // Parse GID first
    let Ok(gid_int) = gid.parse::<i64>() else {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Game ID not parsable");
    };
    // Get Game from the database
    let Ok(Some(db_game)) = game::Entity::find_by_id(gid_int)
        .one(&*prq.sstate.database)
        .await
    else {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Game not found");
    };

    // Get the session via the con descr
    let Ok(Some(db_session)) = session::Entity::find()
        .filter(session::Column::TheaterTcpHandle.eq(prq.con.to_string()))
        .one(&*prq.sstate.database)
        .await
    else {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_AuthFail as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Session not found");
    };

    // ToDo: Send a notification to the other players to join the game...??

    let client_pid = db_session.persona_id;
    // Remove the participant entry
    let _ = participant::Entity::delete_many()
        .filter(
            Condition::all()
                .add(participant::Column::GameId.eq(gid_int))
                .add(participant::Column::PersonaId.eq(client_pid)),
        )
        .exec(&*prq.sstate.database)
        .await;

    /*
    // If no one is there anymore, delete the game entry (we don't have dedicated servers anymore anyhow...)
    if let Ok(n_participants) = participant::Entity::find()
        .filter(participant::Column::GameId.eq(gid_int))
        .count(&*prq.sstate.database)
        .await {
        if n_participants == 0 {
            // Remove the game entry
            let _ = game::Entity::delete_by_id(gid_int).exec(&*prq.sstate.database).await;
        }
    }
    */

    let mut response_hm: IndexMap<_, _, _> = IndexMap::new();
    response_hm.insert("TID".to_string(), tid.to_string());
    response_hm.insert("LID".to_string(), lid.to_string());
    response_hm.insert("GID".to_string(), gid.to_string());

    let response_packet = DataPacket {
        packet_mode: PacketMode::FeslPingOrTheaterResponse,
        mode: DataMode::THEATER_ECNL,
        packet_id: prq.packet.packet_id,
        data: response_hm,
    };

    // Enqueue the response
    submit_packet(response_packet, &prq.con, &prq.sstate, 0).await;

    Ok(())
}
