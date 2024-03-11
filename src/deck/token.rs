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
    /// Each dodge token prevents a full attack
    Dodge,
    /// Poison token increases the damage taken by one pr. attack
    Poison,
}

/// Token permanence defines for how long time the token exists
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub enum TokenPermanence {
    Permanent(PermanentDefinition),
    OneAtEndOfRound,
    AllAtEndOfRound,
    OneAtBeginningOfRound,
    AllAtBeginningOfRound,
    EndOutCombat,
    /// The tokens on a unit or player is permanent but does not define a maximum
    UsedOnUnit,
    Instant,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct PermanentDefinition {
    pub(crate) max_count: u32
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
