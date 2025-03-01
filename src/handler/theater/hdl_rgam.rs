use indexmap::IndexMap;
use sea_orm::entity::*;
use sea_orm::query::*;

use crate::handler::submit_packet;
use crate::orm::model::{game, participant};
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::theater::TheaterHandler;


pub async fn handle_rq_rgam(
    fh: &TheaterHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    // Remove Game
    // {"LID": "1", "GID": "2", "TID": "8"} }
    let lid = prq.packet.data.get("LID").unwrap();
    let gid = prq.packet.data.get("GID").unwrap();
    let tid = prq.packet.data.get("TID").unwrap();

    // Remove game
    let Ok(gid_int) = gid.parse::<i64>() else {
        return Err("Game ID not parsable");
    };

    participant::Entity::delete_many()
        .filter(participant::Column::GameId.eq(gid_int))
        .exec(&*prq.sstate.database)
        .await
        .unwrap();
    game::Entity::delete_by_id(gid_int)
        .exec(&*prq.sstate.database)
        .await
        .unwrap();

    let mut response_hm = IndexMap::new();
    response_hm.insert("TID".to_string(), tid.to_string());

    let response_packet = DataPacket {
        packet_mode: PacketMode::FeslPingOrTheaterResponse,
        mode: DataMode::THEATER_RGAM,
        packet_id: 0,
        data: response_hm,
    };
    submit_packet(response_packet, &prq.con, &prq.sstate, 0).await;

    Ok(())
}
