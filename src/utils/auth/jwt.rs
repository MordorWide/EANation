use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::mordorwide_errors::MWErr;

#[derive(Debug, Clone)]
pub enum JWTErr {
    JWTDecodeError,
    JWTEncodeError,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    username: String,
    hashed_password: String,
    exp: usize,
}

pub fn get_credential_pair_from_jwt(
    token: &String,
    secret: &String,
) -> Result<(String, String), MWErr> {
    if let Ok(decoded_claim) = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    ) {
        Ok((
            decoded_claim.claims.username,
            decoded_claim.claims.hashed_password,
        ))
    } else {
        Err(MWErr::JWTError(JWTErr::JWTDecodeError))
    }
}

pub fn get_jwt_for_credentials(
    username: &String,
    password_hashdata: &String,
    secret: &String,
) -> Result<String, MWErr> {
    // Get the secret from the database
    let header = Header::new(Algorithm::HS256); // SHA2 is fine :)

    let max_exp: usize = i64::MAX as usize; // OMG, this is a long time
    let claim = Claims {
        username: username.to_string(),
        hashed_password: password_hashdata.to_string(),
        exp: max_exp,
    };
    let Ok(token) = encode(&header, &claim, &EncodingKey::from_secret(secret.as_ref())) else {
        return Err(MWErr::JWTError(JWTErr::JWTEncodeError));
    };
    Ok(token)
}
