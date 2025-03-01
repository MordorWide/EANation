use indexmap::IndexMap;
use sea_orm::entity::*;

use crate::handler::submit_packet;
use crate::orm::model::game;
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::theater::TheaterHandler;


pub async fn handle_rsp_ugam(
    fh: &TheaterHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    // Update Game Info
    let tid = prq.packet.data.get("TID").unwrap();
    let lid = prq.packet.data.get("LID").unwrap();
    let gid = prq.packet.data.get("GID").unwrap();
    let gid_int: i64 = gid.parse().unwrap();

    let Ok(Some(db_game)) = game::Entity::find_by_id(gid_int)
        .one(&*prq.sstate.database)
        .await
    else {
        return Err("Game not found");
    };
    let mut db_game: game::ActiveModel = db_game.into_active_model();

    for (key, value) in prq.packet.data.iter() {
        match key.as_ref() {
            "LID" | "GID" | "TID" => continue,
            "JOIN" => {
                db_game.join_mode = Set(value.to_string());
            }
            "B-maxObservers" => {
                db_game.max_observers = Set(value.parse().unwrap());
            }
            "MAX-PLAYERS" => {
                db_game.max_players = Set(value.parse().unwrap());
            }
            "NAME" => {
                db_game.name = Set(value.to_string());
            }
            "B-U-LevelKey" => {
                db_game.user_levelkey = Set(value.to_string());
            }
            "B-U-LevelName" => {
                db_game.user_levelname = Set(value.to_string());
            }
            "B-U-Mode" => {
                db_game.user_mode = Set(value.to_string());
            }
            "B-U-FriendsOnly" => {
                db_game.user_friends_only = Set(value == "1");
            }
            "B-U-Ranked" => {
                db_game.user_ranked = Set(value == "1");
            }
            _ => {
                println!(
                    "[THEATER][REQ][UGAM] Unknown game key value pair: {} := {}",
                    key, value
                );
            }
        }
    }
    // Update game data
    let _ = db_game.update(&*prq.sstate.database).await;

    let mut response_hm = IndexMap::new();
    response_hm.insert("TID".to_string(), tid.to_string());

    let response_packet = DataPacket {
        packet_mode: PacketMode::FeslPingOrTheaterResponse,
        mode: DataMode::THEATER_UGAM,
        packet_id: 0,
        data: response_hm,
    };

    submit_packet(response_packet, &prq.con, &prq.sstate, 0).await;
    Ok(())
}
