use crate::mordorwide_errors::MWErr;

#[derive(Debug, Clone)]
pub enum GameNameErr {
    GameNameTooShortLessThanThreeChars,
    GameNameTooLongMoreThanThirtyOneChars,
    GameNameContainsInvalidChars,
    GameNameStartsOrEndsWithWhitespace,
}

pub fn game_name_validate(game_name: &String) -> Result<(), MWErr> {
    // Check length
    if game_name.as_bytes().len() < 3 {
        return Err(MWErr::ValidationGameNameError(
            GameNameErr::GameNameTooShortLessThanThreeChars,
        ));
    }

    if game_name.as_bytes().len() > 31 {
        return Err(MWErr::ValidationGameNameError(
            GameNameErr::GameNameTooLongMoreThanThirtyOneChars,
        ));
    }

    // Allow only alphanumeric characters, '-', '_' and space
    if !game_name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ' ')
    {
        return Err(MWErr::ValidationGameNameError(
            GameNameErr::GameNameContainsInvalidChars,
        ));
    }

    // Check if first or last character is a space
    if game_name.trim() != game_name {
        return Err(MWErr::ValidationGameNameError(
            GameNameErr::GameNameStartsOrEndsWithWhitespace,
        ));
    }

    return Ok(());
}
