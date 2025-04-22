use indexmap::IndexMap;
use sea_orm::entity::*;
use sea_orm::query::*;
use tracing::info;

use crate::handler::{submit_packet, to_error_packet};
use crate::orm::model::persona;
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_errors::EAError;
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::fesl::FeslHandler;


pub async fn acct_nuloginpersona(
    fh: &FeslHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    // User should be authenticated
    if !prq.is_authenticated_user().await {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_AuthFail as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("User not authenticated.");
    }
    // User should not have selected a persona yet
    if prq.get_active_persona_model().await.is_some() {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_AuthFail as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Persona already selected.");
    }
    let persona_name: String = prq.packet.data.get("name").unwrap().to_string();

    // Check if the persona exists
    let Ok(Some(db_persona)) = persona::Entity::find()
        .filter(persona::Column::Name.eq(persona_name))
        .one(&*prq.sstate.database)
        .await
    else {
        return Err("Failed to retrieve persona data");
    };

    let persona_id = db_persona.id;
    let persona_name: String = db_persona.name;

    let Some(db_session) = prq.get_active_session_model().await else {
        panic!("Session not found although authenticated earlier...");
    };

    let lobby_key = db_session.lobby_key.to_string();
    let owner_id = db_session.user_id;

    let Some(db_account) = prq.get_active_user_model().await else {
        panic!("User not found although authenticated earlier...");
    };

    let owner_name = db_account.email;

    // Report the persona login
    info!(target: "auth", "Login successful for persona: {} (by user: {})", &persona_name, &owner_name);

    let mut db_session_active = db_session.into_active_model();
    db_session_active.persona_id = Set(persona_id);
    db_session_active
        .update(&*prq.sstate.database)
        .await
        .unwrap();
    prq.flush();

    // Select the persona
    let set_success = prq.set_active_persona_session(persona_id).await;
    if !set_success {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_AuthFail as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Persona selection failed.");
    }

    // Prepare response
    let mut response_hm = IndexMap::new();
    response_hm.insert("TXN".to_string(), "NuLoginPersona".to_string());
    response_hm.insert("lkey".to_string(), lobby_key);
    response_hm.insert("profileId".to_string(), owner_id.to_string());
    response_hm.insert("userId".to_string(), owner_id.to_string());

    let response = DataPacket::new(
        DataMode::FESL_ACCT,
        PacketMode::FeslSinglePacketResponse,
        prq.packet.packet_id,
        response_hm,
    );

    submit_packet(response, &prq.con, &prq.sstate, 0).await;
    Ok(())
}
