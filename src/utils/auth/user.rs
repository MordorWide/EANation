use crate::orm::model::{account, ban};
use crate::packet::DataPacket;
use crate::sharedstate::SharedState;
use chrono::NaiveDate;
use sea_orm::entity::*;
use sea_orm::query::*;
use std::sync::Arc;

use crate::mordorwide_errors::MWErr;

// Hashing
use crate::utils::auth::hashing::{plain_string_to_hash, verify_plain_string_for_hash};

// JWT
use crate::utils::auth::jwt::{get_credential_pair_from_jwt, JWTErr};

// User (EMail) Validation / Normalization
use crate::utils::data_validation::email::email_normalize;

#[derive(Debug, Clone)]
pub enum UserAuthErr {
    NoCredentials,
    UserNotFound,
    InvalidPassword,
    AlreadyAuthenticated,
    NewUserAlreadyRegistered,
    UserBanned,
}

#[derive(Debug, Clone)]
pub enum CredentialType {
    PlainText(String, String),
    EncryptedHashed(String, String),
}

pub async fn get_credentials_from_packet(
    packet: &DataPacket,
    sstate: &Arc<SharedState>,
) -> Result<CredentialType, MWErr> {
    // Extract the username and (hashed/plain) password from the packet

    if packet.data.get("nuid").is_some() && packet.data.get("password").is_some() {
        // Plain text credentials -> Password is plain text -> Hash password first
        let username = packet.data.get("nuid").unwrap();
        let raw_password = packet.data.get("password").unwrap().to_string();

        let username = email_normalize(username);

        Ok(CredentialType::PlainText(username, raw_password))
    } else if packet.data.get("encryptedInfo").is_some() {
        // Encrypted JWT-encoded credentials
        let encrypted_info = packet.data.get("encryptedInfo").unwrap();
        let decoded_info = get_credential_pair_from_jwt(encrypted_info, &sstate.server_secret);

        if let Err(_) = decoded_info {
            return Err(MWErr::JWTError(JWTErr::JWTDecodeError));
        }

        let (username, password) = decoded_info.unwrap();

        let username = email_normalize(&username);

        Ok(CredentialType::EncryptedHashed(username, password))
    } else {
        Err(MWErr::UserAuthError(UserAuthErr::NoCredentials))
    }
}

pub async fn validate_credentials(
    credentials: &CredentialType,
    sstate: &Arc<SharedState>,
) -> Result<i64, MWErr> {
    // Extract username first
    let username = match credentials {
        CredentialType::PlainText(username, _) => username,
        CredentialType::EncryptedHashed(username, _) => username,
    };

    // Check if the user exists
    let Ok(Some(db_user)) = account::Entity::find()
        .filter(account::Column::Email.eq(username))
        .one(&*sstate.database)
        .await
    else {
        return Err(MWErr::UserAuthError(UserAuthErr::UserNotFound));
    };

    // Verify password (hashes)
    let credentials_valid = match credentials {
        CredentialType::PlainText(username, plain_password) => {
            verify_plain_string_for_hash(plain_password, &db_user.password_hashed)
        }
        CredentialType::EncryptedHashed(username, hashed_password) => {
            &db_user.password_hashed == hashed_password
        }
    };

    if !credentials_valid {
        return Err(MWErr::UserAuthError(UserAuthErr::InvalidPassword));
    }

    // Check if the user is banned
    let lowercase_username = username.to_lowercase();
    let email_sha256hash = sha256::digest(&lowercase_username);
    match ban::Entity::find()
        .filter(ban::Column::EmailHash.eq(email_sha256hash))
        .count(&*sstate.database)
        .await
    {
        Ok(n_hits) if n_hits > 0 => {
            return Err(MWErr::UserAuthError(UserAuthErr::UserBanned));
        }
        Ok(n_hits) => {
            // User is not banned
        }
        Err(_) => {
            return Err(MWErr::DBError);
        }
    }

    Ok(db_user.id)
}

pub async fn register_new_user(
    email: &String,
    plain_password: &String,
    lobby_key: &String,

    birthdate: NaiveDate,
    optin_global: bool,
    optin_thirdparty: bool,
    email_parental: &String,

    zipcode: &String,
    country: &String,
    language: &String,
    accepted_tos: &String,
    entitlement_key: &String,

    sstate: &Arc<SharedState>,
) -> Result<i64, MWErr> {
    let normalized_email = email_normalize(email);
    // Check if the user exists
    match account::Entity::find()
        .filter(account::Column::Email.eq(&normalized_email))
        .count(&*sstate.database)
        .await
    {
        Ok(count) if count == 0 => {}
        Ok(_) => {
            return Err(MWErr::UserAuthError(UserAuthErr::NewUserAlreadyRegistered));
        }
        Err(_) => {
            return Err(MWErr::DBError);
        }
    }

    // Hash the password
    let hashed_password = plain_string_to_hash(plain_password);

    // Get the current time
    let current_time = chrono::Utc::now();

    // Register the new user
    // Create account in the database
    let db_account = account::ActiveModel {
        email: Set(normalized_email.to_string()),
        password_hashed: Set(hashed_password.to_string()),
        lobby_key: Set(lobby_key.to_string()),

        is_staff: Set(false),
        is_superuser: Set(false),
        is_verified: Set(false),

        created_at: Set(current_time),
        last_login: Set(current_time),

        force_client_turn: Set(false),
        force_server_turn: Set(false),

        optin_global: Set(optin_global),
        optin_thirdparty: Set(optin_thirdparty),
        parental_email: Set(email_parental.to_string()),
        birthdate: Set(birthdate),

        zipcode: Set(zipcode.to_string()),
        country: Set(country.to_string()),
        language: Set(language.to_string()),
        accepted_tos: Set(accepted_tos.to_string()),
        entitlement_key: Set("".to_string()),

        ..Default::default()
    };

    let Ok(db_account) = db_account.insert(&*sstate.database).await else {
        return Err(MWErr::DBError);
    };

    // Return the user id (account id)
    Ok(db_account.id)
}
