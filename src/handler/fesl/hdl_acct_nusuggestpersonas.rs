use indexmap::IndexMap;
use sea_orm::entity::*;
use sea_orm::query::*;

use crate::handler::submit_packet;
use crate::orm::model::persona;
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::fesl::FeslHandler;


pub async fn acct_nusuggestpersonas(
    fh: &FeslHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    /* {"TXN": "NuSuggestPersonas", "name": "test12p1", "maxSuggestions": "4", "keywords.[]": "0"} */
    let name: &String = prq.packet.data.get("name").unwrap();
    let max_suggestions: usize = prq
        .packet
        .data
        .get("maxSuggestions")
        .unwrap_or(&"3".to_string())
        .parse()
        .unwrap();

    let mut suggestions = Vec::with_capacity(max_suggestions);
    let mut ctr: usize = 1;

    while suggestions.len() < max_suggestions {
        let suggested_name = format!("{}-{}", name, ctr);
        // Look up whether the persona name already exists...
        let Ok(n_hits) = persona::Entity::find()
            .filter(persona::Column::Name.eq(&suggested_name))
            .count(&*prq.sstate.database)
            .await
        else {
            return Err("Failed to retrieve persona data");
        };
        if n_hits == 0 {
            suggestions.push(suggested_name);
        }
        ctr += 1;
    }

    // Prepare response
    let mut response_hm = IndexMap::new();
    response_hm.insert("TXN".to_string(), "NuSuggestPersonas".to_string());
    response_hm.insert("names.[]".to_string(), suggestions.len().to_string());
    for (idx, name) in suggestions.iter().enumerate() {
        response_hm.insert(format!("names.{}", idx), name.to_string());
    }

    let response = DataPacket::new(
        DataMode::FESL_ACCT,
        PacketMode::FeslSinglePacketResponse,
        prq.packet.packet_id,
        response_hm,
    );

    submit_packet(response, &prq.con, &prq.sstate, 0).await;
    Ok(())
}
