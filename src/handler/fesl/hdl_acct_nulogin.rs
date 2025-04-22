use indexmap::IndexMap;
use sea_orm::entity::*;
use sea_orm::query::*;
use tracing::{debug, info};

use crate::handler::{submit_packet, to_error_packet};
use crate::mordorwide_errors::MWErr;
use crate::orm::model::config;
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_errors::EAError;
use crate::plasma_handle::PlasmaRequestBundle;
use crate::utils::auth::jwt::{get_jwt_for_credentials, JWTErr};
use crate::utils::auth::user::UserAuthErr;
use crate::utils::config_values::get_cfg_value;
use crate::handler::fesl::FeslHandler;


pub async fn acct_nulogin(
    fh: &FeslHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    let return_jwt_credentials: bool = prq
        .packet
        .data
        .get("returnEncryptedInfo")
        .unwrap_or(&"0".to_string())
        == "1";
    // let jwt_credentials: Option<&String> = prq.packet.data.get("encryptedInfo");
    // let nuid: Option<&String> = prq.packet.data.get("nuid");
    // let password: Option<&String> = prq.packet.data.get("password");
    // let macAddr: Option<&String> = prq.packet.data.get("macAddr");
    let tos_version: Option<String> = prq.packet.data.get("tosVersion").cloned();

    match prq.auth_by_packet().await {
        Ok(()) => {}
        Err(mw_err) => {
            let error_id: i32 = match mw_err {
                MWErr::UserAuthError(UserAuthErr::UserNotFound) => EAError::EA_EmailNotFound as i32,
                MWErr::UserAuthError(UserAuthErr::InvalidPassword) => {
                    EAError::EA_InvalidPassword as i32
                }
                MWErr::UserAuthError(UserAuthErr::UserBanned) => EAError::EA_Banned as i32,
                _ => EAError::EA_AuthFail as i32,
            };
            let err_pkt = to_error_packet(&prq.packet, error_id, None);
            submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
            debug!(target: "fesl", "ACCT/NuLogin - Error occurred: {:?}", mw_err);
            return Err("Authentication failed.");
        }
    };

    let Some(db_session) = prq.get_active_session_model().await else {
        panic!("Session not found although authenticated earlier...");
    };

    // Prepare response
    let mut response_hm = IndexMap::new();
    response_hm.insert("TXN".to_string(), "NuLogin".to_string());
    response_hm.insert("lkey".to_string(), db_session.lobby_key.to_string());
    // Just reuse the user/owner/nuid identifier
    response_hm.insert("profileId".to_string(), db_session.user_id.to_string());
    response_hm.insert("userId".to_string(), db_session.user_id.to_string());

    let Some(db_account) = prq.get_active_user_model().await else {
        panic!("User not found although authenticated earlier...");
    };

    // Report the login
    info!(target: "auth", "Login successful for user: {}", &db_account.email);

    if return_jwt_credentials {
        match get_jwt_for_credentials(
            &db_account.email,
            &db_account.password_hashed,
            &prq.sstate.server_secret,
        ) {
            Ok(encrypted_info) => {
                response_hm.insert("encryptedLoginInfo".to_string(), encrypted_info);
            }
            Err(mw_err) => {
                let error_id: i32 = match mw_err {
                    MWErr::JWTError(JWTErr::JWTEncodeError) => EAError::EA_AuthFail as i32,
                    _ => EAError::EA_AuthFail as i32,
                };
                let err_pkt = to_error_packet(&prq.packet, error_id, None);
                submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
                debug!(target: "fesl", "ACCT/NuLogin - Error occurred: {:?}", mw_err);
                return Err("JWT encoding failed.");
            }
        };
    }

    // Check more recent ToS stuff and show it.
    const DEFAULT_TOS_CHECK: bool = true;
    let mut enable_tos_check = DEFAULT_TOS_CHECK;
    if let Some(cfg_tos_check) = get_cfg_value("ENABLE_TOS_CHECK", &*prq.sstate.database).await {
        if let Ok(nb_cfg_tos_check) = cfg_tos_check.parse::<u32>() {
            enable_tos_check = nb_cfg_tos_check == 1;
        }
    }

    if enable_tos_check {
        // Check the most recent ToS version
        let most_recent_tos_version;
        if let Ok(Some(tos_version)) = config::Entity::find()
            .filter(config::Column::Key.eq("TOS_VERSION"))
            .one(&*prq.sstate.database)
            .await
        {
            most_recent_tos_version = tos_version.value;
        } else {
            most_recent_tos_version = "1.0".to_string();
        };

        // Check if the player has accepted the latest ToS version
        if &db_account.accepted_tos != &most_recent_tos_version {
            // The players last ToS version is outdated
            let mut should_raise_tos = true;
            // Check the ToS version from the request
            if let Some(request_tos) = tos_version {
                if request_tos == most_recent_tos_version {
                    // Don't raise ToS if the player as this request confirms the
                    // latest ToS version
                    should_raise_tos = false;

                    // Update account instead...
                    let mut db_account_active = db_account.into_active_model();
                    db_account_active.accepted_tos = Set(most_recent_tos_version);
                    db_account_active
                        .update(&*prq.sstate.database)
                        .await
                        .unwrap();
                    prq.flush();
                }
            }

            if should_raise_tos {
                let raise_tos_pkt = to_error_packet(&prq.packet, EAError::EA_NewToS as i32, None);
                submit_packet(raise_tos_pkt, &prq.con, &prq.sstate, 0).await;
                return Ok(());
            }
        }
    }

    // Re-read the user model again
    let Some(db_account) = prq.get_active_user_model().await else {
        panic!("User not found although authenticated earlier...");
    };

    // Check Entitlements (=license key)
    const DEFAULT_ENTITLEMENT_ENABLED: bool = true;
    let mut enable_entitlement_check = DEFAULT_ENTITLEMENT_ENABLED;
    if let Some(cfg_entitlement_check) =
        get_cfg_value("ENABLE_ENTITLEMENT", &*prq.sstate.database).await
    {
        if let Ok(nb_cfg_entitlement_check) = cfg_entitlement_check.parse::<u32>() {
            enable_entitlement_check = nb_cfg_entitlement_check == 1;
        }
    }

    // Perform the entitlement check if required
    if enable_entitlement_check {
        // If the entitlement key is not set, request the entitlement key.
        if &db_account.entitlement_key == "" {
            let err_pkt = to_error_packet(&prq.packet, EAError::EA_NotEntitled as i32, None);
            submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
            return Ok(());
        }
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
