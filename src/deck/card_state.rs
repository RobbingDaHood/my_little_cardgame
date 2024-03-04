use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::JsonSchema;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub enum CardState {
    Deck,
    Hand,
    Discard,
}