use chrono::NaiveDate;
use indexmap::IndexMap;
use uuid::Uuid;
use tracing::debug;

use crate::handler::{submit_packet, to_error_packet};
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_errors::EAError;
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::fesl::FeslHandler;
use crate::utils::auth::user::register_new_user;
use crate::utils::data_validation::email::{email_normalize, email_validate};
use crate::utils::data_validation::password::password_validate;


pub async fn acct_nuaddaccount(
    fh: &FeslHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    /*
        "TXN": "NuAddAccount",
        "nuid": "asdf",
        "password": "asdf",
        "globalOptin": "0", // EA may notify you about EA events/news/products.
        "thirdPartyOptin": "1", // EA may forward contact information to third parties
        "parentalEmail": "<blub>",
        "DOBDay": "3", "DOBMonth": "3", "DOBYear": "1995", "zipCode": "",
        "country": "CA",
        "language": "",
        "tosVersion": "1.0",
    */

    let nuid: &String = prq.packet.data.get("nuid").unwrap();
    let plain_password: &String = prq.packet.data.get("password").unwrap();
    let optin_global: &String = prq.packet.data.get("globalOptin").unwrap();
    let optin_thirdparty: &String = prq.packet.data.get("thirdPartyOptin").unwrap();
    let email_parental: &String = prq.packet.data.get("parentalEmail").unwrap();
    let date_of_birth_day: &String = prq.packet.data.get("DOBDay").unwrap();
    let date_of_birth_month: &String = prq.packet.data.get("DOBMonth").unwrap();
    let date_of_birth_year: &String = prq.packet.data.get("DOBYear").unwrap();
    let zip_code: &String = prq.packet.data.get("zipCode").unwrap();
    let country: &String = prq.packet.data.get("country").unwrap();
    let language: &String = prq.packet.data.get("language").unwrap();
    let tos_version: &String = prq.packet.data.get("tosVersion").unwrap();

    // Convert "1" to true, "0" to false
    let optin_global = optin_global == "1";
    let optin_thirdparty = optin_thirdparty == "1";

    // Normalize the email address first
    let normalized_nuid = email_normalize(nuid);

    // Validate the email address
    if let Err(mw_err) = email_validate(&normalized_nuid) {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_LoginErrorHeading as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Invalid email address");
    }

    // Validate the password
    if let Err(mw_err) = password_validate(&plain_password) {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_InvalidPassword as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Invalid password");
    }
    // Parse the birthdate (Format: YYYY-MM-DD)
    let birthdate: NaiveDate = chrono::NaiveDate::parse_from_str(
        format!(
            "{}-{}-{}",
            date_of_birth_year.to_string(),
            date_of_birth_month.to_string(),
            date_of_birth_day.to_string()
        )
        .as_str(),
        "%Y-%m-%d",
    )
    .unwrap();

    // ToDo: Make this more general

    // Generate random lobby key
    let uuid = Uuid::new_v4();
    let lobby_key: &str = &uuid.to_string();
    // Set empty entitlement key
    let entitlement_key = String::new();

    let reg_result = register_new_user(
        &normalized_nuid,
        plain_password,
        &lobby_key.to_string(),
        birthdate,
        optin_global,
        optin_thirdparty,
        email_parental,
        zip_code,
        country,
        language,
        tos_version,
        &entitlement_key,
        &prq.sstate,
    )
    .await;

    if let Err(mw_err) = reg_result {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_LoginErrorHeading as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        debug!(target: "fesl", "ACCT/NuAddAccount - Error occurred: {:?}", mw_err);
        return Err("Registration failed.");
    }
    let user_id = reg_result.unwrap();

    let mut response_hm = IndexMap::new();
    response_hm.insert("TXN".to_string(), "NuAddAccount".to_string());

    let response = DataPacket::new(
        DataMode::FESL_ACCT,
        PacketMode::FeslSinglePacketResponse,
        prq.packet.packet_id,
        response_hm,
    );

    submit_packet(response, &prq.con, &prq.sstate, 0).await;
    Ok(())
}
