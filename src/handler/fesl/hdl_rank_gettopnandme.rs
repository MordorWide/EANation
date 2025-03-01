use indexmap::IndexMap;
use rand::Rng;
use sea_orm::entity::*;
use sea_orm::query::*;

use crate::handler::{submit_packet, to_error_packet};
use crate::orm::model::persona;
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_errors::EAError;
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::fesl::FeslHandler;


pub async fn rank_gettopnandme(
    fh: &FeslHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    // Check if the user is authenticated
    if !prq.is_authenticated_user().await {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_AuthFail as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("User not authenticated.");
    }

    /*
    {"TXN": "GetTopNAndMe", "key": "3.2137219119.score", "ownerType": "1", "minRank": "1", "maxRank": "50", "periodId": "4", "periodPast": "0", "rankOrder": "0", "includeUser": "1"} }
    */
    let key_opt = prq.packet.data.get("key");
    let owner_type_opt = prq.packet.data.get("ownerType");
    let min_rank_opt = prq.packet.data.get("minRank");
    let max_rank_opt = prq.packet.data.get("maxRank");
    let period_id_opt = prq.packet.data.get("periodId");
    let period_past_opt = prq.packet.data.get("periodPast");
    let rank_order_opt = prq.packet.data.get("rankOrder");
    let include_user_opt = prq.packet.data.get("includeUser");

    let Some(rank_key) = key_opt else {
        return Err("No key provided");
    };

    let rank_min = min_rank_opt
        .unwrap_or(&"1".to_string())
        .parse::<u32>()
        .unwrap();
    let rank_max = max_rank_opt
        .unwrap_or(&"10".to_string())
        .parse::<u32>()
        .unwrap();
    let n_entries = rank_max - rank_min + 1;

    let Ok(db_personas) = persona::Entity::find()
        .limit(n_entries as u64)
        .all(&*prq.sstate.database)
        .await
    else {
        return Err("Failed to retrieve persona data");
    };

    let mut response_hm = IndexMap::new();
    response_hm.insert("TXN".to_string(), "GetTopNAndMe".to_string());

    let mut rng = prq.sstate.rng.write().await;

    let top_n = db_personas.len();
    // Add the top N personas (faked)
    response_hm.insert("stats.[]".to_string(), top_n.to_string());
    for (persona_idx, db_persona) in db_personas.iter().enumerate() {
        let rank_data = vec![rank_key];
        response_hm.insert(
            format!("stats.{}.addStats.[]", persona_idx.to_string()),
            rank_data.len().to_string(),
        );
        for (stat_idx, stat_key) in rank_data.iter().enumerate() {
            // Generate a random number between 0 and 1000
            let random_stat = rng.gen_range(0..1000);
            response_hm.insert(
                format!(
                    "stats.{}.addStats.{}.key",
                    persona_idx.to_string(),
                    stat_idx.to_string()
                ),
                stat_key.to_string(),
            );
            response_hm.insert(
                format!(
                    "stats.{}.addStats.{}.value",
                    persona_idx.to_string(),
                    stat_idx.to_string()
                ),
                random_stat.to_string(),
            );
        }
        response_hm.insert(
            format!("stats.{}.owner", persona_idx),
            db_persona.user_id.to_string(),
        );
        response_hm.insert(
            format!("stats.{}.name", persona_idx),
            db_persona.name.to_string(),
        );
        response_hm.insert(
            format!("stats.{}.rank", persona_idx),
            (persona_idx + 1).to_string(),
        );
    }

    let response = DataPacket::new(
        DataMode::FESL_RANK,
        PacketMode::FeslSinglePacketResponse,
        prq.packet.packet_id,
        response_hm,
    );

    submit_packet(response, &prq.con, &prq.sstate, 0).await;
    Ok(())
}
