use crate::mordorwide_errors::MWErr;

#[derive(Debug, Clone)]
pub enum PersonaErr {
    PersonaNameTooShortLessThanThreeChars,
    PersonaNameTooLongMoreThanThirtyOneChars,
    PersonaNameContainsInvalidChars,
    PersonaNameStartsOrEndsWithWhitespace,
}

pub fn persona_validate(persona_name: &String) -> Result<(), MWErr> {
    // Check length
    if persona_name.as_bytes().len() < 3 {
        return Err(MWErr::ValidationPersonaError(
            PersonaErr::PersonaNameTooShortLessThanThreeChars,
        ));
    }

    if persona_name.as_bytes().len() > 31 {
        return Err(MWErr::ValidationPersonaError(
            PersonaErr::PersonaNameTooLongMoreThanThirtyOneChars,
        ));
    }

    // Allow only alphanumeric characters, '-', '_' and space
    if !persona_name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ' ')
    {
        return Err(MWErr::ValidationPersonaError(
            PersonaErr::PersonaNameContainsInvalidChars,
        ));
    }

    // Check if first or last character is a space
    if persona_name.trim() != persona_name {
        return Err(MWErr::ValidationPersonaError(
            PersonaErr::PersonaNameStartsOrEndsWithWhitespace,
        ));
    }

    return Ok(());
}
