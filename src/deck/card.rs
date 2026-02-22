use crate::deck::token::Token;
use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::JsonSchema;

/// Represents a card in the game.
///
/// Cards have effects, costs, and can exist in multiple decks simultaneously.
/// Each card has a unique ID and belongs to one of three types: Attack, Defence, or Resource.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct Card {
    /// Unique id of the card
    pub id: usize,
    /// Effects that activate when the card is played
    pub effects: Vec<Token>,
    /// Resources required to play this card
    pub costs: Vec<Token>,
    /// Number of copies of this card
    pub count: u32,
    /// The type of card (Attack, Defence, or Resource)
    pub card_type: CardType,
}

/// The three types of cards available in the game.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub enum CardType {
    Attack,
    Defence,
    Resource,
}
