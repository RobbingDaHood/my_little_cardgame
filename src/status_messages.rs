use rocket::serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub enum StatusMessages {
    CardAddedToDeck(),
    CardAlreadyInDeck(usize, usize),
    CreatedStatusMessage(usize),
    CardNotFound(usize),
    DeckNotFound(usize),
}
