use indexmap::IndexMap;

use crate::handler::{submit_packet, to_error_packet};
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_errors::EAError;
use crate::plasma_handle::PlasmaRequestBundle;
use crate::utils::config_values::get_cfg_value;
use crate::handler::fesl::FeslHandler;


pub async fn acct_nugettos(
    fh: &FeslHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    let mut country_code = prq.packet.data.get("countryCode").cloned();

    // let mut user_tos_version: Option<String> = None;

    // Determine the country code if not provided (e.g. the logged-in user has only seen an outdated ToS)
    if country_code.is_none() {
        if let Some(db_user) = prq.get_active_user_model().await {
            country_code = Some(db_user.country);

            // The tos version that the user has accepted so far.
            // user_tos_version = Some(db_user.accepted_tos);
        };
    }

    let TOS_VERSION_KEY = "TOS_VERSION";
    let TOS_TEXT_KEY = format!("TOS_TEXT_{}", country_code.unwrap_or("US".to_string()));
    let DEFAULT_TOS_TEXT_KEY = "TOS_TEXT_US";

    let mut tos_text = "No text for the Terms of Service yet. Stay tuned!".to_string();
    let mut tos_version = "1.0".to_string();

    // Try to get the ToS text
    match get_cfg_value(&TOS_TEXT_KEY, &*prq.sstate.database).await {
        Some(tos_data) => {
            tos_text = tos_data;
        }
        None => {
            // Try to find the default (fallback) TOS
            if let Some(tos_default_data) =
                get_cfg_value(&DEFAULT_TOS_TEXT_KEY, &*prq.sstate.database).await
            {
                tos_text = tos_default_data;
            } else {
                // No default TOS found
                let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
                submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
                return Err("Unable to query ToS data.");
            }
        }
    }

    // Try to get the ToS version
    if let Some(db_tos_version) = get_cfg_value(&TOS_VERSION_KEY, &*prq.sstate.database).await {
        tos_version = db_tos_version;
    } else {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NoData as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Unable to query ToS version data.");
    }

    // Prepare response
    let mut response_hm = IndexMap::new();
    response_hm.insert("TXN".to_string(), "NuGetTos".to_string());
    response_hm.insert("tos".to_string(), tos_text.to_string());
    response_hm.insert("version".to_string(), tos_version.to_string());

    let response = DataPacket::new(
        DataMode::FESL_ACCT,
        PacketMode::FeslSinglePacketResponse,
        prq.packet.packet_id,
        response_hm,
    );

    submit_packet(response, &prq.con, &prq.sstate, 0).await;
    Ok(())
}
