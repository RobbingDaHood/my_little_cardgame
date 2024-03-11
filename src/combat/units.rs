use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::JsonSchema;
use crate::deck::Card;
use crate::deck::token::Token;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct Unit {
    attack_deck: Vec<Card>,
    defence_deck: Vec<Card>,
    resource_deck: Vec<Card>,
    effect_deck: Vec<Card>,
    tokens: Vec<Token>,
}
