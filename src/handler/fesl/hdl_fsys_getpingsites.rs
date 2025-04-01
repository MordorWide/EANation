use indexmap::IndexMap;

use crate::handler::submit_packet;
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::fesl::FeslHandler;
use crate::utils::config_values::get_cfg_value;

pub async fn fsys_getpingsites(
    fh: &FeslHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {

    let mut response_hm: IndexMap<_, _, _> = IndexMap::new();
    response_hm.insert("TXN".to_string(), "GetPingSites".to_string());

    let mut n_ping_sites = 0;
    // Get the ping sites from the database
    if let Some(ping_sites_str) =
        get_cfg_value("GetPingSites_PingSites", &*prq.sstate.database).await
    {
        // Parse the ping sites string into a JSON structure
        if let Ok(ping_sites) = serde_json::from_str::<Vec<IndexMap<String, String>>>(&ping_sites_str) {
            ping_sites.iter().for_each(|ping_site| {
                response_hm.insert(
                    format!("pingSite.{}.addr", n_ping_sites),
                    ping_site.get("addr").unwrap().to_string(),
                );
                response_hm.insert(
                    format!("pingSite.{}.type", n_ping_sites),
                    ping_site.get("type").unwrap().to_string(),
                );
                response_hm.insert(
                    format!("pingSite.{}.name", n_ping_sites),
                    ping_site.get("name").unwrap().to_string(),
                );
                n_ping_sites += 1;
            });
        }
    }

    // Set number of ping sites
    response_hm.insert("pingSite.[]".to_string(), n_ping_sites.to_string());

    let mut min_sites_to_ping = 0;
    if let Some(min_sites_to_ping_str) =
        get_cfg_value("GetPingSites_minPingSitesToPing", &*prq.sstate.database).await
    {
        min_sites_to_ping = min_sites_to_ping_str.parse::<i32>().unwrap_or(0);
    }

    // Limit the number of ping sites to match the number of ping sites at max
    min_sites_to_ping = min_sites_to_ping.min(n_ping_sites);
    response_hm.insert(
        "minPingSitesToPing".to_string(),
        min_sites_to_ping.to_string(),
    );

    let response = DataPacket::new(
        DataMode::FESL_FSYS,
        PacketMode::FeslSinglePacketResponse,
        prq.packet.packet_id,
        response_hm,
    );

    // Enqueue the response
    submit_packet(response, &prq.con, &prq.sstate, 0).await;
    Ok(())
}
