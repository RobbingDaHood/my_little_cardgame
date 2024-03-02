use std::ops::DerefMut;

use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Json;
use rocket::State;

use crate::deck::card::Card;
pub use crate::deck::card::new as new_card;
use crate::player_data::PLayerData;
use crate::status_messages::StatusMessages;

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

impl Deck {
    pub fn add_new_card(&mut self, new_card: Card) {
        self.cards.push(new_card);
    }
}

#[get("/")]
pub async fn list_all_decks(player_data: &State<PLayerData>) -> Json<Vec<Deck>> {
    let mut result = vec![];
    for deck in &player_data.decks {
        result.push(deck.lock().await.clone())
    }
    Json(result)
}

#[get("/<id>")]
pub async fn get_deck(id: usize, player_data: &State<PLayerData>) -> Json<Deck> {
    Json(player_data.decks[id].lock().await.clone())
}

#[post("/<id>", format = "json", data = "<new_card>")]
pub async fn add_card_to_deck(id: usize, new_card: Json<Card>, player_data: &State<PLayerData>) -> Json<StatusMessages> {
    player_data.decks[id].lock().await.deref_mut().add_new_card(new_card.0);
    Json(StatusMessages::CreatedStatusMessage(1))
}
