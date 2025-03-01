use crate::utils::auth::jwt::JWTErr;
use crate::utils::data_validation::email::EmailErr;
use crate::utils::data_validation::game_name::GameNameErr;
use crate::utils::data_validation::password::PasswordErr;
use crate::utils::data_validation::persona::PersonaErr;

use crate::utils::auth::user::UserAuthErr;

#[derive(Debug, Clone)]
pub enum MWErr {
    JWTError(JWTErr),

    ValidationEmailError(EmailErr),
    ValidationPasswordError(PasswordErr),
    ValidationGameNameError(GameNameErr),
    ValidationPersonaError(PersonaErr),

    UserAuthError(UserAuthErr),

    DBError,
}
