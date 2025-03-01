use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use indexmap::IndexMap;
use sea_orm::entity::*;
use sea_orm::query::*;
use std::cmp::max;
use uuid::Uuid;

use crate::client_connection::{ClientConnectionDescriptor, ProtoType, ServiceType};
use crate::handler::{submit_packet, to_error_packet};
use crate::orm::model::{account, game, participant, persona, session};
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_errors::EAError;
use crate::plasma_handle::PlasmaRequestBundle;
use crate::utils::stun_turn::{TurnRequestBody, TurnResponseBody};
use crate::handler::theater::TheaterHandler;


const DEFAULT_GAME_PORT: i32 = 11900;

pub async fn handle_rq_egam(
    fh: &TheaterHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    // Enter Game

    /* // Joining Game of persona 'lookingforgofp1'
    "PORT": "11900",
    "R-INT-PORT": "11900",
    "R-INT-IP": "192.168.1.53",
    "PTYPE": "P",
    "USER": "lookingforgofp1",
    "R-USER": "lookingforgofp1",
    "TYPE": "G",
    "TID": "4"
    */

    /* Xbox joining:
        "PORT": "11900",
        "R-XNADDR": "ZoTqaGaE6miMoAAiSE54mgAiSE54mgAAAAAAAAAAAAAAAAAA",
        "PTYPE": "P",
        "LID": "1",
        "GID": "94",
        "TID": "5"} }
    */

    // Try to check if the game exists (and find it)
    let db_game;
    if !prq.packet.data.contains_key("GID") {
        // The client is looking for a game of USER / R-USER to join
        // ToDo: Look for game and proceed....
        let persona_host_to_join = prq.packet.data.get("USER").unwrap();

        // Lets find the persona first...
        let Ok(Some(db_host_persona)) = persona::Entity::find()
            .filter(persona::Column::Name.eq(persona_host_to_join))
            .one(&*prq.sstate.database)
            .await
        else {
            let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
            submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
            return Err("Persona not found");
        };
        let persona_id_host_to_join = db_host_persona.id;

        let Ok(Some(db_persona_game)) = game::Entity::find()
            .filter(game::Column::PersonaId.eq(persona_id_host_to_join))
            .one(&*prq.sstate.database)
            .await
        else {
            let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
            submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
            return Err("Game not found");
        };
        db_game = db_persona_game;
    } else {
        // We know that the request contains a GID -> Convert first!
        let raw_gid = prq.packet.data.get("GID").unwrap();
        let Ok(gid_int) = raw_gid.parse::<i64>() else {
            let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
            submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
            return Err("Game ID not parsable");
        };
        // Search for the game...
        let Ok(Some(db_gid_game)) = game::Entity::find_by_id(gid_int)
            .one(&*prq.sstate.database)
            .await
        else {
            let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
            submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
            return Err("Game not found");
        };
        db_game = db_gid_game;
    }

    if &db_game.join_mode != "O" {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Game is not open for joining");
    };

    // ToDo: Check if the player is allowed to join the game (by checking at the game state)
    let tid = prq.packet.data.get("TID").unwrap();
    let lid = db_game.lobby_id.to_string();
    let gid = db_game.id.to_string();

    // Determine the number of current players:
    let gid = db_game.id;
    let Ok(n_all_players) = participant::Entity::find()
        .filter(participant::Column::GameId.eq(gid))
        .count(&*prq.sstate.database)
        .await
    else {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Failed to get number of players");
    };

    let n_max_players = db_game.max_players as i32;
    let n_all_players = n_all_players as i32;

    let n_open_slots = max(0, n_max_players - n_all_players);
    let queue_len = max(0, n_all_players - n_max_players);
    let can_join = n_open_slots > 0;

    let mut response_hm = IndexMap::new();
    response_hm.insert("TID".to_string(), tid.to_string());
    response_hm.insert("LID".to_string(), lid.to_string());
    response_hm.insert("GID".to_string(), gid.to_string());

    // Add QLEN and QPOS
    if !can_join {
        response_hm.insert("QLEN".to_string(), queue_len.to_string());
        response_hm.insert("QPOS".to_string(), queue_len.to_string());
    }
    let response_packet = DataPacket {
        packet_mode: PacketMode::FeslPingOrTheaterResponse,
        mode: DataMode::THEATER_EGAM,
        packet_id: prq.packet.packet_id,
        data: response_hm,
    };
    submit_packet(response_packet, &prq.con, &prq.sstate, 0).await;

    // {"PORT": "11900", "R-INT-PORT": "6001", "R-INT-IP": "192.168.1.53", "PTYPE": "P", "LID": "1", "GID": "2", "TID": "5"} }

    // Now, handle EGRQ or QENT (packet should be sent to the server)

    // Collect the game data
    let remote_int_ip: &String;
    let remote_int_port: u16;

    // Handle Xbox specifics
    let xbox_bytes: Vec<u8>;
    let xbox_r_ip: String;
    let xbox_r_int_ip: String;
    let xbox_r_int_port: u16;
    if !prq.packet.data.contains_key("R-INT-IP")
        && !prq.packet.data.contains_key("R-INT-PORT")
        && prq.packet.data.contains_key("R-XNADDR")
    {
        // We need to extract some info from the XNADDR
        // See: https://github.com/xenia-project/xenia/blob/3d30b2eec3ab1f83140b09745bee881fb5d5dde2/src/xenia/kernel/xam/xam_net.cc#L51C1-L51C17

        let b64engine = STANDARD;
        let xnaddr = prq.packet.data.get("R-XNADDR").unwrap();
        xbox_bytes = b64engine.decode(&xnaddr).unwrap();

        xbox_r_int_ip = format!(
            "{}.{}.{}.{}",
            xbox_bytes[0], xbox_bytes[1], xbox_bytes[2], xbox_bytes[3]
        );
        xbox_r_ip = format!(
            "{}.{}.{}.{}",
            xbox_bytes[4], xbox_bytes[5], xbox_bytes[6], xbox_bytes[7]
        );
        xbox_r_int_port = (xbox_bytes[8] as u16) << 8 | xbox_bytes[9] as u16;
        remote_int_ip = &xbox_r_int_ip;
        let xbox_r_int_port = 11900;
        remote_int_port = xbox_r_int_port;

        // We need to add the (assumed) udp connection info to the session
        let Ok(Some(xbox_session)) = session::Entity::find()
            .filter(session::Column::TheaterTcpHandle.eq(prq.con.to_string()))
            .one(&*prq.sstate.database)
            .await
        else {
            return Err("Session (Xbox) not found");
        };
        let mut active_session = xbox_session.into_active_model();
        // Set the assumed UDP connection
        active_session.theater_udp_handle = Set(ClientConnectionDescriptor::new(
            ProtoType::Udp,
            ServiceType::Theater,
            18885,
            xbox_r_ip.clone(),
            xbox_r_int_port,
        )
        .to_string());
        // Set the NAT type to restricted to enforce the use of the TURN server

        // Update: Overwrite it to be NAT_OPEN
        active_session.nat_type = Set(1);
        let Ok(_) = active_session.update(&*prq.sstate.database).await else {
            return Err("Failed to update session (Xbox)");
        };
    } else {
        remote_int_ip = prq.packet.data.get("R-INT-IP").unwrap();
        remote_int_port = prq.packet.data.get("R-INT-PORT").unwrap().parse().unwrap();
    }

    // Query client data
    let Ok(Some(db_client_session)) = session::Entity::find()
        .filter(session::Column::TheaterTcpHandle.eq(prq.con.to_string()))
        .one(&*prq.sstate.database)
        .await
    else {
        return Err("Client session not found");
    };

    let Ok(Some(db_client_persona)) = persona::Entity::find_by_id(db_client_session.persona_id)
        .one(&*prq.sstate.database)
        .await
    else {
        return Err("Client persona not found");
    };

    let Ok(Some(db_client_account)) = account::Entity::find_by_id(db_client_session.user_id)
        .one(&*prq.sstate.database)
        .await
    else {
        return Err("Client account not found.");
    };

    // Look for the server session + theater handle
    let Ok(Some(db_host_session)) = session::Entity::find()
        .filter(session::Column::PersonaId.eq(db_game.persona_id))
        .one(&*prq.sstate.database)
        .await
    else {
        return Err("Server session not found");
    };

    // Get the host account
    let Ok(Some(db_host_account)) = account::Entity::find_by_id(db_host_session.user_id)
        .one(&*prq.sstate.database)
        .await
    else {
        return Err("Host account not found");
    };

    let enter_own_game = db_client_session.persona_id == db_game.persona_id;

    // Update NAT type if the player is not the host...
    if !enter_own_game {
        if db_client_session.nat_type == 3 {
            // NAT_SIMPLE if
            // - the internal port matches the external port, and
            // - the actual udp port matches the external/internal port
            // - the port is 11900 (the default game port for the client) (<- This is a bit of a hack, not sure if it is really necessary)
            let advertised_port = prq.packet.data.get("PORT").unwrap().parse::<i32>().unwrap();

            let udp_handle =
                ClientConnectionDescriptor::from_string(&db_client_session.theater_udp_handle);
            if udp_handle.client_port as i32 == advertised_port
                && udp_handle.client_port as i32 == DEFAULT_GAME_PORT
            {
                // We have a simple NAT :)
                let mut active_session = db_client_session.clone().into_active_model();
                active_session.nat_type = Set(2);
                let _ = active_session.update(&*prq.sstate.database).await;
            } else {
                // We have a strict NAT :(
                // thus, we don't need to update the DB...
            }
        }
    }

    // Reload session from DB
    let Ok(Some(db_client_session)) = session::Entity::find_by_id(db_client_session.id)
        .one(&*prq.sstate.database)
        .await
    else {
        return Err("Client session not found after updating NAT type");
    };

    // Get the THEATER handle of the host
    let host_con_descr =
        ClientConnectionDescriptor::from_string(&db_host_session.theater_tcp_handle);

    // Get the actual UDP connection data of host and client
    let host_udp_con = ClientConnectionDescriptor::from_string(&db_host_session.theater_udp_handle);
    let client_udp_con =
        ClientConnectionDescriptor::from_string(&db_client_session.theater_udp_handle);

    let actual_client_ip = &client_udp_con.client_ip;
    let actual_client_port = client_udp_con.client_port;
    let actual_host_ip = &host_udp_con.client_ip;
    let actual_host_port = host_udp_con.client_port;

    let mut host_expected_client_ip: &String;
    let mut host_expected_client_port: u16;
    let mut client_expected_host_ip: &String;
    let mut client_expected_host_port: u16;

    // First, set the default values if no TURN server is available
    host_expected_client_ip = &actual_client_ip;
    host_expected_client_port = actual_client_port;
    client_expected_host_ip = &actual_host_ip;
    client_expected_host_port = actual_host_port;

    // Check if we need to use a TURN server if it is available
    let mut need_turn;
    if !enter_own_game && prq.sstate.turn.enabled {
        need_turn = true;
        let host_wants_turn = db_host_account.force_server_turn;
        let client_wants_turn = db_client_account.force_client_turn;

        let client_nat_type = db_client_session.nat_type;
        let host_nat_type = db_host_session.nat_type;

        // We can avoid using the TURN server if
        // - the host NAT type is open
        // - the host NAT type is moderate and the client NAT type is moderate (or open)
        // Otherwise, we probably need to use the TURN server if
        // - the host NAT type is strict
        // - the host NAT type is moderate and the client NAT type is strict
        if host_nat_type == 1 {
            need_turn = false;
        }
        if host_nat_type == 2 && (client_nat_type == 2 || client_nat_type == 1) {
            need_turn = false;
        }

        need_turn = need_turn || host_wants_turn || client_wants_turn;

        if need_turn {
            // Use the TURN server for the connection
            let turn_request_body = TurnRequestBody {
                client_ip_0: actual_client_ip.clone(),
                client_port_0: actual_client_port,
                client_ip_1: actual_host_ip.clone(),
                client_port_1: actual_host_port,
            };
            // Send the TURN request to the TURN server
            let Ok(response) = reqwest::Client::new()
                .post(&format!(
                    "http://{}:{}/launch",
                    prq.sstate.turn.control_host, prq.sstate.turn.control_port
                ))
                .json(&turn_request_body)
                .send()
                .await
            else {
                return Err("Failed to send TURN request");
            };

            let Ok(turn_response) = response.json::<TurnResponseBody>().await else {
                return Err("Failed to parse TURN response");
            };

            if !turn_response.success {
                return Err("TURN server failed to create connection");
            }

            println!("TURN response: {:?}", turn_response);

            let turn_client_port = turn_response.relay_port_0.unwrap();
            let turn_host_port = turn_response.relay_port_1.unwrap();

            // Set TURN-relayed connection data
            host_expected_client_ip = &prq.sstate.turn.external_ip;
            host_expected_client_port = turn_host_port;
            client_expected_host_ip = &prq.sstate.turn.external_ip;
            client_expected_host_port = turn_client_port;
        } else {
            // Apparently, no TURN server required.
            // Therefore, we advertise the direct connection.
        }
    } else {
        // We
        // - are entering our own game, or
        // - don't have any TURN server available.
        // Let's hope for the best....
        need_turn = false;
    }
    println!("Need TURN: {}", need_turn);

    // generate a random UUID ticket
    let join_ticket = Uuid::new_v4().to_string();

    let uid = db_client_account.id;

    // Add the player to the participant table
    let db_new_participant = participant::ActiveModel {
        game_id: Set(gid),
        persona_id: Set(db_client_session.persona_id),
        queue_pos: Set(queue_len),
        ticket: Set(join_ticket.clone()),

        client_expected_host_ip: Set(client_expected_host_ip.clone()),
        client_expected_host_port: Set(client_expected_host_port as i32),
        host_expected_client_ip: Set(host_expected_client_ip.clone()),
        host_expected_client_port: Set(host_expected_client_port as i32),

        ..Default::default()
    };
    let Ok(db_new_participant) = db_new_participant.insert(&*prq.sstate.database).await else {
        return Err("Failed to insert new participant");
    };
    // Is PID the Persona ID or the Participant ID?
    let pid = db_client_persona.id;

    match can_join {
        false => {
            // Send a QENT request to the server (enter queue)
            // This is not tested, because TLotR:CQ does not implement QUEUEs.

            let mut qent_hm = IndexMap::new();
            //qent_hm.insert("R-INT-PORT".to_string(), remote_int_port.to_string());
            //qent_hm.insert("R-INT-IP".to_string(), remote_int_ip.to_string());
            qent_hm.insert("R-INT-PORT".to_string(), remote_int_port.to_string());
            qent_hm.insert("R-INT-IP".to_string(), remote_int_ip.to_string());
            qent_hm.insert("NAME".to_string(), db_client_persona.name.to_string());
            qent_hm.insert("PID".to_string(), pid.to_string());

            qent_hm.insert("UID".to_string(), uid.to_string());
            qent_hm.insert("LID".to_string(), lid.to_string());
            qent_hm.insert("GID".to_string(), gid.to_string());

            let qent_request = DataPacket {
                packet_mode: PacketMode::TheaterRequest,
                mode: DataMode::THEATER_QENT,
                packet_id: 0,
                data: qent_hm,
            };
            submit_packet(qent_request, &host_con_descr, &prq.sstate, 0).await;
        }
        true => {
            // Send EGRQ to the server (join server request)
            let port: usize = prq.packet.data.get("PORT").unwrap().parse().unwrap();
            let ptype = prq.packet.data.get("PTYPE").unwrap();

            // Send EGRQ to the server (join server)
            let mut egrq_hm = IndexMap::new();
            egrq_hm.insert("R-INT-PORT".to_string(), remote_int_port.to_string());
            egrq_hm.insert("R-INT-IP".to_string(), remote_int_ip.to_string());

            // Add the client IP and port (TURN-aware)
            egrq_hm.insert("IP".to_string(), host_expected_client_ip.to_string());
            egrq_hm.insert("PORT".to_string(), host_expected_client_port.to_string());
            //egrq_hm.insert("IP".to_string(), con.client_ip.to_string());
            //egrq_hm.insert("PORT".to_string(), port.to_string());

            egrq_hm.insert("NAME".to_string(), db_client_persona.name.to_string());
            egrq_hm.insert("PTYPE".to_string(), "P".to_string());
            egrq_hm.insert("TICKET".to_string(), join_ticket.to_string());
            egrq_hm.insert("PID".to_string(), pid.to_string());
            egrq_hm.insert("UID".to_string(), uid.to_string());
            egrq_hm.insert("LID".to_string(), lid.to_string());
            egrq_hm.insert("GID".to_string(), gid.to_string());

            let egrq_request = DataPacket {
                packet_mode: PacketMode::FeslPingOrTheaterResponse,
                mode: DataMode::THEATER_EGRQ,
                packet_id: 0,
                data: egrq_hm,
            };
            submit_packet(egrq_request, &host_con_descr, &prq.sstate, 0).await;
        }
    }
    Ok(())
}
