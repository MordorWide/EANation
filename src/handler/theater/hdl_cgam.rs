use indexmap::IndexMap;
use sea_orm::entity::*;
use sea_orm::query::*;

use crate::client_connection::ClientConnectionDescriptor;
use crate::handler::{submit_packet, to_error_packet};
use crate::orm::model::{game, persona, session};
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_errors::EAError;
use crate::plasma_handle::PlasmaRequestBundle;
use crate::utils::data_validation::game_name::game_name_validate;
use crate::handler::theater::TheaterHandler;


pub async fn handle_rq_cgam(
    fh: &TheaterHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    /*{
    "LID": "-1", # LobbyID
    "RESERVE-HOST": "1", # ??? -> bool? -> dedicated? Reserve slot for host?
    "NAME": "LegolasWTF", # Persona Name of Host
    "PORT": "11900", # This is the client UDP port that also pings the Theater instances at UDP/18885
    "HTTYPE": "A", # D or A, ascending, descending (Host migration)
    "TYPE": "G", # G = Game,  "Game, Playgroup, Count)"
    "QLEN": "0",
    "DISABLE-AUTO-DEQUEUE": "0",
    "HXFR": "0", # Enable Host Migration
    "INT-PORT": "11900", # Internal (Game?) port
    "INT-IP": "192.168.1.53", // This is the local IP of the client
    "MAX-PLAYERS": "16",
    "B-maxObservers": "0",
    "B-numObservers": "0",
    "UGID": "", # Unique?/User? Game ID
    "SECRET": "",
    "B-U-FriendsOnly": "0", # Only friends can join?!
    // If a dedicated client is active: "B-U-PCDedicated": "1"
    "B-U-PlayMode": "0",
    "B-U-Ranked": "0",
    "B-U-Version": "245478296",
    "B-version": "",
    "JOIN": "O", # Open, Closed, Wait
    "RT": "",
    "TID": "4"
    }*/
    /* Dedicated:
        { mode: CGAM, packet_mode: TheaterRequest, packet_id: 0, data: {"LID": "-1", "RESERVE-HOST": "0"
    , "NAME": "Game1", "PORT": "11900", "HTTYPE": "A", "TYPE": "G", "QLEN": "0", "DISABLE-AUTO-DEQUEUE": "0", "HXFR": "0", "INT-PORT": "11900", "INT-IP": "192.168.1.53", "MAX-PLAYERS": "16", "B-maxObservers": "0", "B-numObservers": "0", "UGID": "", "SECRET": "", "B-U-FriendsOnly": "0", "B-U-PCDedicated": "1", "B-U-Play
    Mode": "0", "B-U-Ranked": "0", "B-U-Version": "245478296", "B-version": "", "JOIN": "O", "RT": "", "TID": "4"} }
            */

    // Extract Game Data
    let tid = prq.packet.data.get("TID").unwrap();
    let lid: usize = 1; // prq.packet.data.get("LID").unwrap();
    let reserve_host: bool = prq.packet.data.get("RESERVE-HOST").unwrap() == "1";
    let name: &str = prq.packet.data.get("NAME").unwrap();
    let port: i32 = prq.packet.data.get("PORT").unwrap().parse().unwrap();
    let httype = prq.packet.data.get("HTTYPE").unwrap();
    let game_type = prq.packet.data.get("TYPE").unwrap();
    let queue_len: usize = prq.packet.data.get("QLEN").unwrap().parse().unwrap();
    let disable_auto_dequeue: bool = prq.packet.data.get("DISABLE-AUTO-DEQUEUE").unwrap() == "1";
    let hxfr: &str = prq.packet.data.get("HXFR").unwrap();
    let int_port: i32 = prq.packet.data.get("INT-PORT").unwrap().parse().unwrap();
    let int_ip = prq.packet.data.get("INT-IP").unwrap();
    let max_players: usize = prq.packet.data.get("MAX-PLAYERS").unwrap().parse().unwrap();
    let b_max_observers: usize = prq
        .packet
        .data
        .get("B-maxObservers")
        .unwrap()
        .parse()
        .unwrap();
    let b_num_observers: usize = prq
        .packet
        .data
        .get("B-numObservers")
        .unwrap()
        .parse()
        .unwrap();
    let ugid = "NOGUID"; //packet.data.get("UGID").unwrap();
    let secret = "NOSECRET"; // prq.packet.data.get("SECRET").unwrap();
    let b_u_friends_only: bool = prq
        .packet
        .data
        .get("B-U-FriendsOnly")
        .unwrap_or(&String::from("0"))
        == "1";
    let b_u_pcdedicated: bool = prq
        .packet
        .data
        .get("B-U-PCDedicated")
        .unwrap_or(&String::from("0"))
        == "1";
    let b_u_play_mode: &str = prq.packet.data.get("B-U-PlayMode").unwrap();
    let b_u_ranked: bool = prq
        .packet
        .data
        .get("B-U-Ranked")
        .unwrap_or(&String::from("0"))
        == "1";
    let client_version: &str = prq.packet.data.get("B-U-Version").unwrap();
    let server_version: &str = prq.packet.data.get("B-version").unwrap();
    let join_mode = prq.packet.data.get("JOIN").unwrap();
    let rt = prq.packet.data.get("RT").unwrap();

    const EKEY: &str = "NOENCYRPTIONKEY";
    //const SECRET: &str = "NOSECRET";
    //const UGID: &str = "NOGUID";

    // Do not use the name from the packet, but the persona name from the session
    // (dedicated servers do not set the name to the persona name)
    let Ok(Some(db_session)) = session::Entity::find()
        .filter(session::Column::TheaterTcpHandle.eq(prq.con.to_string()))
        .one(&*prq.sstate.database)
        .await
    else {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Session not found");
    };
    let persona_id = db_session.persona_id;

    // Find the persona from the persona id
    let Ok(Some(db_persona)) = persona::Entity::find_by_id(persona_id)
        .one(&*prq.sstate.database)
        .await
    else {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Persona not found");
    };

    // Validate game name
    if let Err(game_validation_error) = game_name_validate(&name.to_string()) {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Invalid game name");
    }

    // Check if games with the same name exist to avoid duplicate game names...
    let Ok(n_same_gamename) = game::Entity::find()
        .filter(game::Column::Name.eq(name.to_string()))
        .count(&*prq.sstate.database)
        .await
    else {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Failed to get number of games");
    };
    if n_same_gamename > 0 {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Game with same name already exists");
    }

    // Re-visit the NAT type check because we now know the advertised external IP and port.
    // if the current NAT type is NAT_STRICT...
    if db_session.nat_type == 3 {
        // NAT_SIMPLE if
        // - the internal port matches the external port, and
        // - the actual udp port matches the external/internal port

        let udp_handle = ClientConnectionDescriptor::from_string(&db_session.theater_udp_handle);
        if udp_handle.client_port as i32 == port && udp_handle.client_port as i32 == int_port {
            // We have a simple NAT :)
            let mut active_session = db_session.clone().into_active_model();
            active_session.nat_type = Set(2);
            let _ = active_session.update(&*prq.sstate.database).await;
        } else {
            // We have a strict NAT :(
            // thus, we don't need to update the DB...
        }
    }

    // Create a new game entry
    let db_new_game = game::ActiveModel {
        lobby_id: Set(lid as i32),
        reserve_host: Set(reserve_host),
        name: Set(name.to_string()), // Persona Name (or Game Name if dedicated)
        persona_id: Set(persona_id), // Persona ID
        port: Set(port),
        host_type: Set(httype.to_string()),
        game_type: Set(game_type.to_string()),
        queue_length: Set(queue_len as i32),
        disable_autodequeue: Set(disable_auto_dequeue),
        hxfr: Set(hxfr.to_string()),
        internal_port: Set(int_port),
        internal_ip: Set(int_ip.to_string()),
        max_players: Set(max_players as i32),
        max_observers: Set(b_max_observers as i32),
        user_group_id: Set(ugid.to_string()),
        secret: Set(secret.to_string()),
        user_friends_only: Set(b_u_friends_only),
        user_pcdedicated: Set(b_u_pcdedicated),
        user_playmode: Set(b_u_play_mode.to_string()),
        user_ranked: Set(b_u_ranked),
        user_levelkey: Set("".to_string()),
        user_levelname: Set("".to_string()),
        user_mode: Set("".to_string()),
        client_version: Set(client_version.to_string()),
        server_version: Set(server_version.to_string()),
        join_mode: Set(join_mode.to_string()),
        rt: Set(rt.to_string()),
        encryption_key: Set(EKEY.to_string()),
        ..Default::default()
    };
    let Ok(db_new_game) = db_new_game.insert(&*prq.sstate.database).await else {
        return Err("Failed to insert new game");
    };

    let db_new_game: game::Model = db_new_game.into();
    let game_id = db_new_game.id;

    let mut response_hm = IndexMap::new();
    response_hm.insert("TID".to_string(), tid.to_string());
    response_hm.insert("MAX-PLAYERS".to_string(), max_players.to_string());
    response_hm.insert("EKEY".to_string(), EKEY.to_string());
    response_hm.insert("UGID".to_string(), ugid.to_string());
    response_hm.insert("JOIN".to_string(), join_mode.to_string());
    response_hm.insert("LID".to_string(), lid.to_string());
    response_hm.insert("SECRET".to_string(), secret.to_string());
    response_hm.insert("J".to_string(), join_mode.to_string());
    response_hm.insert("GID".to_string(), game_id.to_string());
    response_hm.insert("HXFR".to_string(), hxfr.to_string());

    let response_packet = DataPacket {
        packet_mode: PacketMode::FeslPingOrTheaterResponse,
        mode: DataMode::THEATER_CGAM,
        packet_id: prq.packet.packet_id,
        data: response_hm,
    };

    // Enqueue the response
    submit_packet(response_packet, &prq.con, &prq.sstate, 0).await;

    Ok(())
}
