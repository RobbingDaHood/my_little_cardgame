use rocket::serde::{Deserialize, Serialize};

use crate::deck::card_state::CardState;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Card {
    card_type_id: u32,
    state: CardState,
    deck_index: u32,
}

pub fn new(card_type_id: u32, state: CardState, deck_index: u32) -> Card {
    Card {
        card_type_id,
        state,
        deck_index,
    }
}
