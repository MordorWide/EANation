use indexmap::IndexMap;
use sea_orm::entity::*;
use sea_orm::query::*;

use crate::handler::submit_packet;
use crate::orm::model::game;
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::theater::TheaterHandler;


pub async fn handle_rq_llst(
    fh: &TheaterHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    // Enqueue the response
    /* {
        "FILTER-FAV-ONLY": "0",
        "FILTER-NOT-FULL": "0",
        "FILTER-NOT-PRIVATE": "0",
        "FILTER-NOT-CLOSED": "0",
        "FILTER-MIN-SIZE": "0",
        "FAV-PLAYER": "",
        "FAV-GAME": "",
        "FAV-PLAYER-UID": "",
        "FAV-GAME-UID": "",
        "TID": "3"
    }*/
    let tid = prq.packet.data.get("TID").unwrap();
    let lid = match prq.packet.data.get("LID") {
        Some(lid) => lid,
        None => &"1".to_string(),
    };
    let lobby_id: i32 = lid.parse().unwrap();

    // Prepare lobby list
    let mut lobby_list = IndexMap::new();
    lobby_list.insert("TID".to_string(), tid.to_string());
    lobby_list.insert("NUM-LOBBIES".to_string(), "1".to_string());

    let lobby_list_response = DataPacket {
        packet_mode: PacketMode::FeslPingOrTheaterResponse,
        mode: DataMode::THEATER_LLST,
        packet_id: prq.packet.packet_id,
        data: lobby_list,
    };

    // Enqueue the response
    submit_packet(lobby_list_response, &prq.con, &prq.sstate, 0).await;

    // Prepare lobby data
    const PASSING: u32 = 1;
    const NAME: &str = "lotr-pandemic";
    const LOCALE: &str = "en_US";
    const MAX_GAMES: u32 = 1000;
    const FAVORITE_GAMES: u32 = 0;
    const FAVORITE_PLAYERS: u32 = 0;
    //const NUM_GAMES: u32 = 1;

    //let games = sstate.database.get_games_by_lobby_id(lid.parse().unwrap(), MAX_GAMES as usize).unwrap();
    let Ok(num_games) = game::Entity::find()
        .filter(
            Condition::all()
                .add(game::Column::LobbyId.eq(lobby_id))
                .add(game::Column::UserFriendsOnly.eq(false)), // Hide 'private' games
        )
        .count(&*prq.sstate.database)
        .await
    else {
        return Err("Failed to get number of games");
    };

    let mut lobby_data = IndexMap::new();
    lobby_data.insert("TID".to_string(), tid.to_string());
    lobby_data.insert("LID".to_string(), lid.to_string());
    lobby_data.insert("PASSING".to_string(), PASSING.to_string());
    lobby_data.insert("NAME".to_string(), NAME.to_string());
    lobby_data.insert("LOCALE".to_string(), LOCALE.to_string());
    lobby_data.insert("MAX-GAMES".to_string(), MAX_GAMES.to_string());
    lobby_data.insert("FAVORITE-GAMES".to_string(), FAVORITE_GAMES.to_string());
    lobby_data.insert("FAVORITE-PLAYERS".to_string(), FAVORITE_PLAYERS.to_string());
    lobby_data.insert("NUM-GAMES".to_string(), num_games.to_string());

    let lobby_data_response = DataPacket {
        packet_mode: PacketMode::FeslPingOrTheaterResponse,
        mode: DataMode::THEATER_LDAT,
        packet_id: prq.packet.packet_id,
        data: lobby_data,
    };
    submit_packet(lobby_data_response, &prq.con, &prq.sstate, 0).await;

    Ok(())
}
