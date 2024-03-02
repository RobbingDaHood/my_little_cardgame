use rocket::serde::{Deserialize, Serialize};

use crate::deck::Deck;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PLayerData {
    pub(crate) decks: Vec<Deck>,
}

pub fn new(decks: Vec<Deck>) -> PLayerData {
    PLayerData {
        decks
    }
}
