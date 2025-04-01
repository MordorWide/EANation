use indexmap::IndexMap;
use sea_orm::entity::*;
use sea_orm::query::*;

use crate::handler::submit_packet;
use crate::orm::model::{game, participant, session};
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::theater::TheaterHandler;
use crate::utils::config_values::get_cfg_value;


pub async fn handle_rq_glst(
    fh: &TheaterHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    // Game List
    /* {
        "LID": "1",
        "TYPE": "G",
        "FILTER-FAV-ONLY": "0",
        "FILTER-NOT-FULL": "0",
        "FILTER-NOT-PRIVATE": "0",
        "FILTER-NOT-CLOSED": "0",
        "FILTER-MIN-SIZE": "0",
        "FILTER-ATTR-U-FriendsOnly": "0",
        "FILTER-ATTR-U-Ranked": "0",
        "FILTER-ATTR-U-Version": "245478296",
        "FAV-PLAYER": "",
        "FAV-GAME": "",
        "COUNT": "-1",
        "FAV-PLAYER-UID": "",
        "FAV-GAME-UID": "",
        "TID": "4"
    } */
    let tid = prq.packet.data.get("TID").unwrap();
    let lid = prq.packet.data.get("LID").unwrap();

    let lid_int: i32 = lid.parse().unwrap();
    // TODO: Implement filtering
    const MAX_GAMES: usize = 1000;
    // TODO: Add limit filter
    /*let Ok(db_games_in_lobby) = game::Entity::find()
        .filter(
            Condition::all()
                .add(game::Column::LobbyId.eq(lid_int))
                .add(game::Column::UserFriendsOnly.eq(false)) // Hide 'private' games
        )
        .all(&*prq.sstate.database)
        .await else {
        return Err("Unable to query games.");
    };*/
    let db_games_in_lobby = game::Entity::find()
        .filter(
            Condition::all()
                .add(game::Column::LobbyId.eq(lid_int))
                .add(game::Column::UserFriendsOnly.eq(false)), // Hide 'private' games
        )
        .all(&*prq.sstate.database)
        .await
        .unwrap();

    // Transform into a vector
    let mut games_in_lobby: Vec<IndexMap<String, String>> = Vec::new();
    for db_game in db_games_in_lobby {
        let mut game_hm = IndexMap::new();
        game_hm.insert("id".to_string(), db_game.id.to_string());
        game_hm.insert("lobby_id".to_string(), db_game.lobby_id.to_string());
        game_hm.insert(
            "reserve_host".to_string(),
            (if db_game.reserve_host { "1" } else { "0" }).to_string(),
        );
        game_hm.insert("name".to_string(), db_game.name.to_string());
        game_hm.insert("persona_id".to_string(), db_game.persona_id.to_string());
        game_hm.insert("port".to_string(), db_game.port.to_string());
        game_hm.insert("host_type".to_string(), db_game.host_type.to_string());
        game_hm.insert("game_type".to_string(), db_game.game_type.to_string());
        game_hm.insert("queue_length".to_string(), db_game.queue_length.to_string());
        game_hm.insert(
            "disable_autodequeue".to_string(),
            (if db_game.disable_autodequeue {
                "1"
            } else {
                "0"
            })
            .to_string(),
        );
        game_hm.insert("hxfr".to_string(), db_game.hxfr.to_string());
        game_hm.insert(
            "internal_port".to_string(),
            db_game.internal_port.to_string(),
        );
        game_hm.insert("internal_ip".to_string(), db_game.internal_ip.to_string());
        game_hm.insert("max_players".to_string(), db_game.max_players.to_string());
        game_hm.insert(
            "max_observers".to_string(),
            db_game.max_observers.to_string(),
        );
        // The game does not support observers.
        game_hm.insert("num_observers".to_string(), 0.to_string());
        game_hm.insert(
            "user_group_id".to_string(),
            db_game.user_group_id.to_string(),
        );
        game_hm.insert("secret".to_string(), db_game.secret.to_string());
        game_hm.insert(
            "user_friends_only".to_string(),
            (if db_game.user_friends_only { "1" } else { "0" }).to_string(),
        );
        game_hm.insert(
            "user_pcdedicated".to_string(),
            (if db_game.user_pcdedicated { "1" } else { "0" }).to_string(),
        );
        game_hm.insert(
            "user_dlc".to_string(),
            db_game.user_dlc.to_string(),
        );
        game_hm.insert(
            "user_playmode".to_string(),
            db_game.user_playmode.to_string(),
        );
        game_hm.insert(
            "user_ranked".to_string(),
            (if db_game.user_ranked { "1" } else { "0" }).to_string(),
        );
        game_hm.insert(
            "user_levelkey".to_string(),
            db_game.user_levelkey.to_string(),
        );
        game_hm.insert(
            "user_levelname".to_string(),
            db_game.user_levelname.to_string(),
        );
        game_hm.insert("user_mode".to_string(), db_game.user_mode.to_string());
        game_hm.insert(
            "client_version".to_string(),
            db_game.client_version.to_string(),
        );
        game_hm.insert(
            "server_version".to_string(),
            db_game.server_version.to_string(),
        );
        game_hm.insert("join_mode".to_string(), db_game.join_mode.to_string());
        game_hm.insert("rt".to_string(), db_game.rt.to_string());
        game_hm.insert(
            "encryption_key".to_string(),
            db_game.encryption_key.to_string(),
        );
        game_hm.insert("other".to_string(), db_game.other_as_json.to_string());

        games_in_lobby.push(game_hm);
    }

    let num_games = games_in_lobby.len() as usize;

    let mut response_hm = IndexMap::new();
    response_hm.insert("TID".to_string(), tid.to_string());
    response_hm.insert("LID".to_string(),  lid_int.to_string());
    response_hm.insert("LOBBY-NUM-GAMES".to_string(), num_games.to_string());
    response_hm.insert("LOBBY-MAX-GAMES".to_string(), MAX_GAMES.to_string());
    response_hm.insert("FAVORITE-GAMES".to_string(), "0".to_string());
    response_hm.insert("FAVORITE-PLAYERS".to_string(), "0".to_string());
    response_hm.insert("NUM-GAMES".to_string(), num_games.to_string());

    let response_packet = DataPacket {
        packet_mode: PacketMode::FeslPingOrTheaterResponse,
        mode: DataMode::THEATER_GLST,
        packet_id: 0,
        data: response_hm,
    };
    submit_packet(response_packet, &prq.con, &prq.sstate, 0).await;

    // Send individual game infos
    let Ok(Some(session_info)) = session::Entity::find()
        .filter(session::Column::TheaterTcpHandle.eq(prq.con.to_string()))
        .one(&*prq.sstate.database)
        .await
    else {
        return Err("Session not found");
    };

    let mut entries: Vec<IndexMap<String, String>> = Vec::new();

    for (i_game, game) in games_in_lobby.iter().enumerate() {
        // Determine number of current players:
        let Ok(n_cur_players) = participant::Entity::find()
            .filter(
                Condition::all()
                    .add(
                        participant::Column::GameId.eq(game
                            .get("id")
                            .unwrap()
                            .parse::<i64>()
                            .unwrap()),
                    )
                    .add(participant::Column::QueuePos.eq(-1 as i32)),
            )
            .count(&*prq.sstate.database)
            .await
        else {
            return Err("Failed to get number of players");
        };
        let mut game_data_response = IndexMap::new();
        game_data_response.insert("TID".to_string(), tid.to_string());
        game_data_response.insert("LID".to_string(), lid.to_string());
        game_data_response.insert("GID".to_string(), game.get("id").unwrap().to_string());

        // Host Name (normally == Persona Name)
        game_data_response.insert("HN".to_string(), game.get("name").unwrap().to_string());
        // Host ID (Persona ID)
        game_data_response.insert("HU".to_string(), game.get("persona_id").unwrap().to_string());
        // Server Name (normally == Persona Name)
        game_data_response.insert("N".to_string(), game.get("name").unwrap().to_string());

        ////game_data_response.insert("PING".to_string(), "10".to_string());
        //game_data_response.insert("B-U-PING".to_string(), "10".to_string());
        //game_data_response.insert("B-U-Ping".to_string(), "10".to_string());
        //game_data_response.insert("Ping".to_string(), "10".to_string());
        // IP/Port of the host
        game_data_response.insert("I".to_string(), prq.con.client_ip.to_string());
        game_data_response.insert("P".to_string(), game.get("port").unwrap().to_string());

        // Version of the host
        //game_data_response.insert("V".to_string(), "1.0".to_string());

        // Platform
        //game_data_response.insert("PL".to_string(), "pc".to_string());

        // Max Players
        game_data_response.insert(
            "MP".to_string(),
            game.get("max_players").unwrap().to_string(),
        );
        // Current Players
        game_data_response.insert("AP".to_string(), n_cur_players.to_string());
        // Current Queue
        game_data_response.insert("QP".to_string(), "0".to_string());

        // Is favorite server
        game_data_response.insert("F".to_string(), "0".to_string());
        // Number of favorite players?
        game_data_response.insert("NF".to_string(), "0".to_string());
        // Join mode
        game_data_response.insert("J".to_string(), game.get("join_mode").unwrap().to_string());
        // #Players joining
        game_data_response.insert("JP".to_string(), "0".to_string());
        // Game type
        game_data_response.insert(
            "TYPE".to_string(),
            game.get("game_type").unwrap().to_string(),
        );

        // Server requires password?
        game_data_response.insert("PW".to_string(), "0".to_string());

        game_data_response.insert(
            "B-version".to_string(),
            game.get("server_version").unwrap().to_string(),
        );
        game_data_response.insert(
            "B-numObservers".to_string(),
             game.get("b_num_observers")
                .unwrap_or(&"0".to_string())
                .to_string(),
        );
        game_data_response.insert(
            "B-maxObservers".to_string(),
            game.get("max_observers").unwrap().to_string(),
        );

        if game.get("user_levelkey").unwrap() != "" {
            game_data_response.insert(
                "B-U-LevelKey".to_string(),
                game.get("user_levelkey").unwrap().to_string(),
            );
        }

        if game.get("user_levelname").unwrap() != "" {
            game_data_response.insert(
                "B-U-LevelName".to_string(),
                game.get("user_levelname").unwrap().to_string(),
            );
        }
        if game.get("user_mode").unwrap() != "" {
            game_data_response.insert(
                "B-U-Mode".to_string(),
                game.get("user_mode").unwrap().to_string(),
            );
        }
        if game.get("user_ranked").unwrap() == "1" {
            game_data_response.insert("B-U-Ranked".to_string(), "1".to_string());
        }

        if game.get("user_pcdedicated").unwrap() == "1" {
            game_data_response.insert("B-U-PCDedicated".to_string(), "1".to_string());
        }

        if game.get("user_dlc").unwrap() != "" {
            game_data_response.insert(
                "B-U-DLC".to_string(),
                game.get("user_dlc").unwrap().to_string(),
            );
        }

        // Parse the remaining, JSON-encoded fields
        if game.get("other").unwrap() != "" {
            let others = game.get("other").unwrap().as_str();
            if let Ok(serde_json::Value::Array(items)) = serde_json::from_str(others) {
                for item in items {
                    if let serde_json::Value::Object(obj) = item {
                        for (key, value) in obj.iter() {
                            game_data_response.insert(key.to_string(), value.to_string());
                        }
                    }
                }
            } else {
                println!(
                    "[THEATER][REQ][GLST] Failed to parse other field: {}",
                    game.get("other").unwrap()
                );
            }
        }

        let response_packet = DataPacket {
            packet_mode: PacketMode::FeslPingOrTheaterResponse,
            mode: DataMode::THEATER_GDAT,
            packet_id: 0,
            data: game_data_response,
        };
        submit_packet(response_packet, &prq.con, &prq.sstate, 0).await;
    }

    Ok(())
}
