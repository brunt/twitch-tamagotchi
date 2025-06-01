use crate::commands::PetCommand;
use std::io::Empty;
use winnow::Parser;
use winnow::ascii::Caseless;
use winnow::combinator::{alt, preceded};
use winnow::error::EmptyError;

pub fn get_command(input: &mut &str) -> Option<PetCommand> {
    preceded::<&str, &str, PetCommand, EmptyError, &str, _>(
        "!",
        alt((
            Caseless("feed").value(PetCommand::Feed),
            // Caseless("discipline").value(PetCommand::Discipline),
            Caseless("pet").value(PetCommand::Play),
            Caseless("sleep").value(PetCommand::Sleep),
            Caseless("clean").value(PetCommand::Clean),
            // Caseless("medicine").value(PetCommand::Medicine),
            // Caseless("walk").value(PetCommand::Walk),
        )),
    )
    .parse_next(input)
    .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_command() {
        let input = "!play";
        assert_eq!(get_command(&mut &*input), Some(PetCommand::Play));
    }
}
