use indexmap::IndexMap;
use sea_orm::entity::*;
use sea_orm::query::*;

use crate::handler::{submit_packet, to_error_packet};
use crate::orm::model::persona;
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_errors::EAError;
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::fesl::FeslHandler;


pub async fn acct_nugetpersona(
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

    // Get the namespace argument
    let namespace: Option<String> = prq.packet.data.get("namespace").cloned();

    let Some(db_session) = prq.get_active_session_model().await else {
        panic!("Session not found although authenticated earlier...");
    };
    let user_id = db_session.user_id as i64;

    // Load personas from the database
    let db_personas: Vec<persona::Model> = persona::Entity::find()
        .filter(persona::Column::UserId.eq(user_id))
        .order_by(persona::Column::CreatedAt, sea_orm::Order::Asc)
        .all(&*prq.sstate.database)
        .await
        .unwrap();

    // Collect persona names
    let personas_names = db_personas
        .iter()
        .map(|p| p.name.clone())
        .collect::<Vec<String>>();

    // Prepare response
    let mut response_hm = IndexMap::new();
    response_hm.insert("TXN".to_string(), "NuGetPersonas".to_string());
    response_hm.insert("personas.[]".to_string(), personas_names.len().to_string());

    for (idx, persona) in personas_names.iter().enumerate() {
        response_hm.insert(format!("personas.{}", idx), persona.to_string());
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
