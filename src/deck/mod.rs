use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Json;
use rocket::State;

use crate::deck::card::Card;
pub use crate::deck::card::new as new_card;
use crate::player_data::PLayerData;

mod card;
pub mod card_state;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Deck {
    cards: Vec<Card>,
}

pub fn new(cards: Vec<Card>) -> Deck {
    Deck { cards }
}

#[get("/")]
pub fn list_all_decks(player_data: &State<PLayerData>) -> Json<Vec<Deck>> {
    Json(player_data.decks.clone())
}
