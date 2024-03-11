use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::JsonSchema;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub enum TokenType {
    /// Health tokens are representing how far the entity is from dying
    Health,
    /// Many cards requires a price to be paid in mana
    Mana,
    /// Many cards requires a price to be pain in stamina
    Stamina,
}

/// Token permanence defines for how long time the token exists
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub enum TokenPermanence {
    Permanent,
    OneAtEndOfRound,
    AllAtEndOfRound,
    OneAtBeginningOfRound,
    AllAtBeginningOfRound,
    EndOutCombat,
}

/// Token defines state. Like how much health is accumulated or if the entity haveving the token is poisoned.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct Token {
    pub(crate) token_type: TokenType,
    pub(crate) permanence: TokenPermanence,
    /// The numer of tokens
    pub(crate) count: u32
}
