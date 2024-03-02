use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Json;
use rocket::State;

use crate::deck::card_state::CardState;
use crate::player_data::PLayerData;
use crate::status_messages::StatusMessages;

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

#[get("/")]
pub async fn list_all_cards(player_data: &State<PLayerData>) -> Json<Vec<Card>> {
    Json(player_data.cards.lock().await.clone())
}

#[get("/<id>")]
pub async fn get_card(id: usize, player_data: &State<PLayerData>) -> Option<Json<Card>> {
    player_data.cards.lock().await.get(id)
        .map(|existing| Json(existing.clone()))
}

#[post("/", format = "json", data = "<new_card>")]
pub async fn create_card(new_card: Json<Card>, player_data: &State<PLayerData>) -> Json<StatusMessages> {
    player_data.cards.lock().await.push(new_card.0);
    Json(StatusMessages::CreatedStatusMessage(1))
}
