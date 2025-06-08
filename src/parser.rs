use crate::commands::PetCommand;
use winnow::Parser;
use winnow::ascii::{Caseless, alpha1, space1};
use winnow::combinator::{alt, preceded};
use winnow::error::EmptyError;

pub fn get_command(input: &mut &str) -> Option<PetCommand> {
    preceded::<&str, &str, PetCommand, EmptyError, &str, _>(
        "!",
        alt((
            Caseless("feed").value(PetCommand::Feed),
            Caseless("play").value(PetCommand::Play),
            Caseless("sleep").value(PetCommand::Sleep),
            Caseless("clean").value(PetCommand::Clean),
            Caseless("kill").value(PetCommand::Kill),
            preceded(
                "new",
                preceded(
                    space1,
                    alpha1.map(|s: &str| PetCommand::New(s.trim().to_string())),
                ),
            ),
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

    #[test]
    fn test_new_name() {
        let input = "!new bob";
        assert_eq!(
            get_command(&mut &*input),
            Some(PetCommand::New("bob".to_string()))
        );
    }

    #[test]
    fn test_new_name_spaces() {
        let input = "!new   bob";
        assert_eq!(
            get_command(&mut &*input),
            Some(PetCommand::New("bob".to_string()))
        );
    }
}
