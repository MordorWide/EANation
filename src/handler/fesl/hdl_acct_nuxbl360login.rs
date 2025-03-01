use indexmap::IndexMap;
use sea_orm::entity::*;
use sea_orm::query::*;

use crate::handler::{submit_packet, to_error_packet};
use crate::orm::model::{account, persona};
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_errors::EAError;
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::fesl::FeslHandler;


pub async fn acct_nuxbl360login(
    fh: &FeslHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    // The user should not be authenticated
    if prq.is_authenticated_user().await {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_AuthFail as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("User already authenticated.");
    }

    // Login via Xenia Xbox Live network (requires a Xenia WebService account to be logged in)
    let gamertag_opt = prq.packet.data.get("gamertag").cloned();
    let xuid_opt = prq.packet.data.get("xuid").cloned();
    let mac_addr_opt = prq.packet.data.get("macAddr").cloned();
    let console_id_opt = prq.packet.data.get("consoleId").cloned();

    let Some(gamertag) = gamertag_opt else {
        return Err("No gamertag provided");
    };

    // Get persona data from the database
    let Ok(Some(db_persona)) = persona::Entity::find()
        .filter(
            Condition::all()
                .add(persona::Column::Name.eq(&gamertag))
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
    let user_id = db_account.id;
    let persona_id = db_persona.id;

    let lobby_key = db_account.lobby_key.clone();
    let persona_name = db_persona.name.clone();

    prq.set_active_user_session(&lobby_key, user_id, None).await;
    prq.set_active_persona_session(persona_id).await;

    let mut response_hm = IndexMap::new();
    response_hm.insert("TXN".to_string(), "NuXBL360Login".to_string());
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
