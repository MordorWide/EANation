use indexmap::IndexMap;
use sea_orm::entity::*;
use sea_orm::query::*;
use tracing::{info};

use crate::handler::{submit_packet, to_error_packet};
use crate::orm::model::{account, persona};
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_errors::EAError;
use crate::plasma_handle::PlasmaRequestBundle;
use crate::utils::psn::{dec_hex_str, PSNTicket};
use crate::handler::fesl::FeslHandler;

pub async fn acct_nups3login(
    fh: &FeslHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    // User should not be authenticated
    if prq.is_authenticated_user().await {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_AuthFail as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("User already authenticated.");
    }

    // Login via RPCS3 (requires an active RPCN account to be logged in)
    let ticket_opt = prq.packet.data.get("ticket").cloned();
    //let mac_addr_opt = prq.packet.data.get("macAddr");
    //let console_id_opt = prq.packet.data.get("consoleId");

    // Get PSN name from ticket, so we need to extract the ticket first
    let Some(mut ticket) = ticket_opt else {
        return Err("No ticket provided");
    };

    // Remove the $ sign
    if ticket.starts_with("$") {
        ticket = ticket[1..].to_string();
    }
    // Decode the hex string into bytes
    let Ok(ticket_bytes) = dec_hex_str(&ticket) else {
        return Err("Invalid ticket format");
    };

    // Parse ticket
    let Ok(psn_ticket) = PSNTicket::from_bytes(&ticket_bytes) else {
        return Err("Failed to parse ticket");
    };

    // Extract PSN name (It should be the 6th entry from the first section)
    let psn_name_bytes = psn_ticket.sections[0].data_entries[5].payload.clone();

    // Remove trailing zeroes
    let trimmed_psn_name = psn_name_bytes
        .iter()
        .rposition(|&byte| byte != 0)
        .map(|last_non_zero| &psn_name_bytes[..=last_non_zero])
        .unwrap_or(&[]);
    let psn_name = String::from_utf8(trimmed_psn_name.to_vec()).unwrap();

    // Get persona data from the database
    let Ok(Some(db_persona)) = persona::Entity::find()
        .filter(
            Condition::all()
                .add(persona::Column::Name.eq(&psn_name))
                .add(persona::Column::AllowInsecureLogin.eq(true)),
        )
        .one(&*prq.sstate.database)
        .await
    else {
        // No persona found
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NotFound as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Failed to retrieve persona data");
    };

    // Get user data from the database
    let Ok(Some(db_account)) = account::Entity::find_by_id(db_persona.user_id)
        .one(&*prq.sstate.database)
        .await
    else {
        return Err("Failed to retrieve account data");
    };

    // Report the login
    info!(target: "auth", "Login via PS3 successful for user: {} (via {})", &db_account.email, &prq.con.to_string());

    let user_id = db_account.id;
    let lobby_key = db_account.lobby_key.clone();
    let persona_name = db_persona.name.clone();

    prq.set_active_user_session(&lobby_key, user_id, None).await;
    prq.set_active_persona_session(db_persona.id).await;

    let mut response_hm = IndexMap::new();
    response_hm.insert("TXN".to_string(), "NuPS3Login".to_string());
    response_hm.insert("lkey".to_string(), lobby_key);
    response_hm.insert("profileId".to_string(), user_id.to_string());
    response_hm.insert("userId".to_string(), user_id.to_string());
    response_hm.insert("personaName".to_string(), persona_name);

    let response = DataPacket::new(
        DataMode::FESL_ACCT,
        PacketMode::FeslSinglePacketResponse,
        prq.packet.packet_id,
        response_hm,
    );

    submit_packet(response, &prq.con, &prq.sstate, 0).await;
    Ok(())
}
