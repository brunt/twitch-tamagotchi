use serde::{Deserialize, Serialize};
use std::fmt::Display;

// verbs that owner tells pet to do or does to pet
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PetCommand {
    Feed,
    Play,
    Sleep,
    Clean,
    Kill,
    New(String),
}

impl Display for PetCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PetCommand::Feed => write!(f, "Feed"),
            PetCommand::Play => write!(f, "Pet"),
            PetCommand::Sleep => write!(f, "Sleep"),
            PetCommand::Clean => write!(f, "Clean"),
            PetCommand::Kill => write!(f, "Kill"),
            PetCommand::New(..) => write!(f, "New"),
        }
    }
}
