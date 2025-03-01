use crate::mordorwide_errors::MWErr;

#[derive(Debug, Clone)]
pub enum EmailErr {
    EmailTooShortLessThanFourChars,
    EmailTooLongMoreThanFiftyChars,
    EmailInvalidChars,
    EmailNoAt,
}

pub fn email_normalize(email: &String) -> String {
    // Remove trailing whitespaces
    let stripped_mail = email.trim();

    if !stripped_mail.contains("@") {
        return stripped_mail.to_string();
    }

    // Split into atmost 2 parts
    let mut parts = stripped_mail.rsplitn(2, "@");
    let domain_part = parts.next().unwrap().to_lowercase();
    let email_name = parts.next().unwrap();
    return format!("{}@{}", email_name, domain_part);
}

// Check mail for length and basic sanity
pub fn email_validate(email: &String) -> Result<(), MWErr> {
    // Check for @
    if !email.contains("@") {
        return Err(MWErr::ValidationEmailError(EmailErr::EmailNoAt));
    }
    // Check length (Django checks regex '^[a-zA-Z0-9_.+-]+@[a-z0-9-]+\.[a-z0-9-.]+$'
    // which is at least 4 characters long)
    if email.len() < 4 {
        return Err(MWErr::ValidationEmailError(
            EmailErr::EmailTooShortLessThanFourChars,
        ));
    }

    if email.len() > 50 {
        return Err(MWErr::ValidationEmailError(
            EmailErr::EmailTooLongMoreThanFiftyChars,
        ));
    }

    // Check chars (might be too strict)
    if !email.chars().all(|c| {
        c.is_alphanumeric() || c == '.' || c == '_' || c == ',' || c == '-' || c == '@' || c == '+'
    }) {
        return Err(MWErr::ValidationEmailError(EmailErr::EmailInvalidChars));
    }

    return Ok(());
}
