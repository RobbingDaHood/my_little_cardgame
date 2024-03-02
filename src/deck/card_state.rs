use rocket::serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub enum CardState {
    Deck,
    Hand,
    Discard,
}