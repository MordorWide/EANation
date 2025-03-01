use crate::mordorwide_errors::MWErr;

#[derive(Debug, Clone)]
pub enum PasswordErr {
    PasswordTooShortLessThanSixChars,
    PasswordTooLongMoreThanFiftyChars,
}

// Check password just for length
pub fn password_validate(password: &String) -> Result<(), MWErr> {
    // Check length
    if password.len() < 6 {
        return Err(MWErr::ValidationPasswordError(
            PasswordErr::PasswordTooShortLessThanSixChars,
        ));
    }

    if password.len() > 50 {
        return Err(MWErr::ValidationPasswordError(
            PasswordErr::PasswordTooLongMoreThanFiftyChars,
        ));
    }

    return Ok(());
}
