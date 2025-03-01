use indexmap::IndexMap;
use sea_orm::entity::*;
use sea_orm::query::*;
use sea_orm::sea_query::{Expr, Func};

use crate::handler::{submit_packet, to_error_packet};
use crate::orm::model::persona;
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_errors::EAError;
use crate::plasma_handle::PlasmaRequestBundle;
use crate::utils::config_values::get_cfg_value;
use crate::handler::fesl::FeslHandler;
use crate::utils::data_validation::persona::persona_validate;


pub async fn acct_nuaddpersona(
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

    let selected_persona_name: String = prq.packet.data.get("name").unwrap().to_string();

    let Some(db_session) = prq.get_active_session_model().await else {
        panic!("Session not found although authenticated earlier...");
    };

    let user_id = db_session.user_id;
    let Ok(n_personas) = persona::Entity::find()
        .filter(persona::Column::UserId.eq(user_id))
        .count(&*prq.sstate.database)
        .await
    else {
        return Err("Failed to retrieve persona data");
    };

    // Get the maximum number of personas from the database
    const DEFAULT_MAX_PERSONAS: u32 = 5;
    let mut max_personas = DEFAULT_MAX_PERSONAS;
    if let Some(cfg_max_personas) = get_cfg_value("MAX_PERSONAS", &*prq.sstate.database).await {
        max_personas = cfg_max_personas.parse().unwrap_or(DEFAULT_MAX_PERSONAS);
    }

    if n_personas as u32 >= max_personas {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_TooManyPersonas as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Maximum number of personas reached");
    }

    // Validate new persona name
    if let Err(_) = persona_validate(&selected_persona_name) {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_LoginErrorHeading as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Invalid persona name");
    }
    // Check if the persona already exists (case-insensitive)
    if persona::Entity::find()
        .filter(
            Expr::expr(Func::lower(Expr::col(persona::Column::Name)))
                .eq(selected_persona_name.to_lowercase()),
        )
        .count(&*prq.sstate.database)
        .await
        .unwrap()
        > 0
    {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_NameInUse as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Persona name already taken");
    }

    let creation_date = chrono::Utc::now();

    // Insert new persona
    let db_new_persona = persona::ActiveModel {
        user_id: Set(user_id),
        name: Set(selected_persona_name.to_string()),
        allow_insecure_login: Set(false),
        created_at: Set(creation_date),
        ..Default::default()
    };
    let db_new_persona = db_new_persona.insert(&*prq.sstate.database).await.unwrap();

    // Prepare response
    let mut response_hm = IndexMap::new();
    response_hm.insert("TXN".to_string(), "NuAddPersona".to_string());

    let response = DataPacket::new(
        DataMode::FESL_ACCT,
        PacketMode::FeslSinglePacketResponse,
        prq.packet.packet_id,
        response_hm,
    );

    submit_packet(response, &prq.con, &prq.sstate, 0).await;
    Ok(())
}
