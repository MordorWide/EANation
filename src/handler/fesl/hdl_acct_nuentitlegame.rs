use indexmap::IndexMap;
use sea_orm::entity::*;
use sea_orm::query::*;

use crate::handler::{submit_packet, to_error_packet};
use crate::orm::model::account;
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_errors::EAError;
use crate::plasma_handle::PlasmaRequestBundle;
use crate::utils::config_values::get_cfg_value;
use crate::handler::fesl::FeslHandler;
use crate::utils::auth::user::{
    get_credentials_from_packet, validate_credentials,
};


pub async fn acct_nuentitlegame(
    fh: &FeslHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    /* {"TXN": "NuEntitleGame", "key": "<License Key>", "nuid": "test14", "password": "test14"} } */
    // Allegedly, there can also be the authentication via encryptedInfo

    // User should be authenticated
    if !prq.is_authenticated_user().await {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_AuthFail as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("User not authenticated.");
    }

    let provided_entitlement_key: String = prq.packet.data.get("key").unwrap().to_string();

    // Extract login credentials from the packet
    let credentials = get_credentials_from_packet(&prq.packet, &prq.sstate).await;
    if credentials.is_err() {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_AuthFail as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("User not authenticated.");
    }
    let credentials = credentials.unwrap();

    // Validate the credentials
    let validation = validate_credentials(&credentials, &prq.sstate).await;
    if validation.is_err() {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_AuthFail as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("User not authenticated.");
    }
    let user_id = validation.unwrap();

    // Compare the user_id with the one from the session
    let Some(db_session) = prq.get_active_session_model().await else {
        panic!("Session not found although checked for authentication earlier...");
    };

    if db_session.user_id != user_id {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_AuthFail as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("User not authenticated.");
    }

    let Some(db_account) = prq.get_active_user_model().await else {
        panic!("User not found although checked for authentication earlier...");
    };

    // Check if the entitlement key is already in use?
    let current_entitlement_key = db_account.entitlement_key.clone();
    if current_entitlement_key != "" {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_AccountAlreadyEntitled as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Account is already entitled.");
    }

    // Validate the new entitlement key...
    // > Check if the key is already in use

    // Check if shared entitlement is enabled

    const DEFAULT_SHARED_ENTITLEMENT: bool = true;
    let mut enable_shared_entitlement = DEFAULT_SHARED_ENTITLEMENT;
    if let Some(cfg_shared_entitlement_check) =
        get_cfg_value("ENABLE_SHARED_ENTITLEMENT", &*prq.sstate.database).await
    {
        if let Ok(nb_fg_shared_entitlement_check) = cfg_shared_entitlement_check.parse::<u32>() {
            enable_shared_entitlement = nb_fg_shared_entitlement_check == 1;
        }
    }

    if !enable_shared_entitlement {
        // Check if the key is already is use
        let Ok(n_hits) = account::Entity::find()
            .filter(account::Column::EntitlementKey.eq(&provided_entitlement_key))
            .count(&*prq.sstate.database)
            .await
        else {
            return Err("Failed to retrieve account data");
        };
        if n_hits > 0 {
            let err_pkt =
                to_error_packet(&prq.packet, EAError::EA_RegCodeAlreadyInuse as i32, None);
            submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
            return Err("License key is already in use.");
        }
    }

    // > Validate the structure -> ToDo: Implement a proper check
    if !&provided_entitlement_key.ends_with("-MORDORWIDE") {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_InvalidRegCode as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("License key is invalid.");
    }

    // Update the account with the new entitlement key
    let mut db_account_active = db_account.into_active_model();
    db_account_active.entitlement_key = Set(provided_entitlement_key.to_string());
    db_account_active
        .update(&*prq.sstate.database)
        .await
        .unwrap();
    prq.flush();

    // Prepare response
    let mut response_hm = IndexMap::new();
    response_hm.insert("TXN".to_string(), "NuEntitleGame".to_string());
    response_hm.insert("lkey".to_string(), provided_entitlement_key.to_string());
    response_hm.insert("profileId".to_string(), user_id.to_string());
    response_hm.insert("userId".to_string(), user_id.to_string());

    let response = DataPacket::new(
        DataMode::FESL_ACCT,
        PacketMode::FeslSinglePacketResponse,
        prq.packet.packet_id,
        response_hm,
    );

    submit_packet(response, &prq.con, &prq.sstate, 0).await;
    Ok(())
}
