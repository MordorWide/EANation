use indexmap::IndexMap;
use sea_orm::entity::*;
use sea_orm::query::*;

use crate::client_connection::ClientConnectionDescriptor;
use crate::handler::{submit_packet, to_error_packet};
use crate::orm::model::{account, game, participant, persona, session};
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_errors::EAError;
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::theater::TheaterHandler;


pub async fn handle_rq_egrs(
    fh: &TheaterHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    // The EGRS comes in from the game host.

    // Enter Game Response
    // {"LID": "1", "GID": "10", "ALLOWED": "1", "PID": "1", "TID": "6"} }
    let lid = prq.packet.data.get("LID").unwrap();
    let tid = prq.packet.data.get("TID").unwrap();
    let gid = prq.packet.data.get("GID").unwrap();
    let pid = prq.packet.data.get("PID").unwrap();
    let allowed = prq.packet.data.get("ALLOWED").unwrap_or(&"1".to_string()) == "1";

    let mut response_hm = IndexMap::new();
    response_hm.insert("TID".to_string(), tid.to_string());

    let response_packet = DataPacket {
        packet_mode: PacketMode::FeslPingOrTheaterResponse,
        mode: DataMode::THEATER_EGRS,
        packet_id: 0,
        data: response_hm,
    };
    submit_packet(response_packet, &prq.con, &prq.sstate, 0).await;

    // Now, send the EGEG packet to the actual client of the corresponding PID
    // ToDo: Handle queue stuff

    // Get the game from the database
    let Ok(gid_int) = gid.parse::<i64>() else {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Game ID not parsable");
    };
    let Ok(Some(db_game)) = game::Entity::find_by_id(gid_int)
        .one(&*prq.sstate.database)
        .await
    else {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Game not found");
    };

    // Get the host session from the database
    let Ok(Some(db_host_session)) = session::Entity::find()
        .filter(session::Column::TheaterTcpHandle.eq(prq.con.to_string()))
        .one(&*prq.sstate.database)
        .await
    else {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Host session not found");
    };

    // Get the IP of the host of the game
    let host_persona_id = db_game.persona_id;
    //let host_persona_name = db_game.name;
    /*let Ok(Some(db_host_persona)) = persona::Entity::find()
        .filter(persona::Column::Name.eq(&host_persona_name))
        .one(&*prq.sstate.database)
        .await else {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32,None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Host persona not found");
    };*/
    let Ok(Some(db_host_persona)) = persona::Entity::find_by_id(host_persona_id)
        .one(&*prq.sstate.database)
        .await
    else {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Host persona not found");
    };

    let host_user_id = db_host_persona.user_id;
    let Ok(Some(db_host_account)) = account::Entity::find_by_id(host_user_id)
        .one(&*prq.sstate.database)
        .await
    else {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Host account not found");
    };

    let Ok(client_persona_id) = pid.parse::<i64>() else {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("PID not parsable");
    };
    // Get the client participant from the database
    let Ok(Some(db_client_participant)) = participant::Entity::find()
        .filter(
            Condition::all()
                .add(participant::Column::GameId.eq(gid_int))
                .add(participant::Column::PersonaId.eq(client_persona_id)),
        )
        .one(&*prq.sstate.database)
        .await
    else {
        return Err("Client participant entry not found");
    };

    // Get the session of the client
    let Ok(Some(db_client_session)) = session::Entity::find()
        .filter(session::Column::PersonaId.eq(db_client_participant.persona_id))
        .one(&*prq.sstate.database)
        .await
    else {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Client session not found");
    };

    let ticket = &db_client_participant.ticket;

    let host_con_descr = prq.con.clone();
    let client_con_descr =
        ClientConnectionDescriptor::from_string(&db_client_session.theater_tcp_handle);

    let host_ip = &host_con_descr.client_ip;
    let host_port = &host_con_descr.client_port;
    let client_ip = &client_con_descr.client_ip;
    let client_port = &client_con_descr.client_port;

    // Load the connection values that can incorporate TURN server settings
    let host_expected_client_ip = &db_client_participant.host_expected_client_ip;
    let host_expected_client_port = db_client_participant.host_expected_client_port;
    let client_expected_host_ip = &db_client_participant.client_expected_host_ip;
    let client_expected_host_port = db_client_participant.client_expected_host_port;

    if !allowed {
        // Delete the participation entry
        let _ = participant::Entity::delete_by_id(db_client_participant.id)
            .exec(&*prq.sstate.database)
            .await;

        // TODO: Send a notification to the client that joining is not allowed
        return Err("Joining not allowed");
    }

    // Now, send EGEG to the client!
    let mut response_hm = IndexMap::new();
    response_hm.insert("PL".to_string(), "pc".to_string());
    response_hm.insert("TICKET".to_string(), ticket.to_string());
    // PID := Player "Ticket" ID??? -> Just needs to be identical to the egam PID!

    //let joiningplayer_id = 1501; // db_game.num_players + db_game.queue_length + 1;
    response_hm.insert("PID".to_string(), pid.to_string());

    // Host Port (TURN-aware)
    response_hm.insert("P".to_string(), client_expected_host_port.to_string()); // Port of the host
                                                                                // response_hm.insert("P".to_string(), db_game.port.to_string()); // Port of the host
                                                                                // Shortcut, we need to store the UID of the host in the game data in the future
                                                                                // HUID should be the persona id of the host
    response_hm.insert("HUID".to_string(), db_host_persona.user_id.to_string()); // Or stick to persona id?
    response_hm.insert("INT-PORT".to_string(), db_game.internal_port.to_string());
    response_hm.insert("EKEY".to_string(), db_game.encryption_key.to_string());
    response_hm.insert("INT-IP".to_string(), db_game.internal_ip.to_string());
    response_hm.insert("UGID".to_string(), db_game.user_group_id.to_string()); // Is this correct?!

    // Host IP (TURN-aware)
    response_hm.insert("I".to_string(), client_expected_host_ip.to_string()); // IP-Address of the host
                                                                              //response_hm.insert("I".to_string(), host_ip.to_string()); // IP-Address of the host
    response_hm.insert("LID".to_string(), lid.to_string());
    response_hm.insert("GID".to_string(), gid.to_string());

    let response_packet = DataPacket {
        packet_mode: PacketMode::FeslPingOrTheaterResponse,
        mode: DataMode::THEATER_EGEG,
        packet_id: 0,
        data: response_hm,
    };

    submit_packet(response_packet, &client_con_descr, &prq.sstate, 0).await;

    Ok(())
}
