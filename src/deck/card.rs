use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Json;
use rocket::State;

use crate::deck::card_state::CardState;
use crate::player_data::PLayerData;
use crate::status_messages::StatusMessages;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Card {
    card_type_id: usize,
    state: CardState,
    id: usize,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CardCreate {
    card_type_id: usize,
    state: CardState,
}

pub fn new(card_type_id: usize, state: CardState, id: usize) -> Card {
    Card {
        card_type_id,
        state,
        id,
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
pub async fn create_card(new_card: Json<CardCreate>, player_data: &State<PLayerData>) -> Json<StatusMessages> {
    let the_card = new_card.0;
    let unused_id = *player_data.cards.lock().await.iter()
        .map(|existing| existing.id)
        .max()
        .map(|existing_id| existing_id + 1)
        .get_or_insert(0);
    player_data.cards.lock().await.push(
        Card {
            card_type_id: the_card.card_type_id,
            state: the_card.state,
            id: unused_id,
        }
    );
    Json(StatusMessages::CreatedStatusMessage(unused_id))
}
