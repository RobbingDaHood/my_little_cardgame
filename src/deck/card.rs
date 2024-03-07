use rocket::response::status::{Created, NotFound};
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::{JsonSchema, openapi};

use crate::player_data::PLayerData;
use crate::status_messages::{new_status, Status};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct Card {
    /// Refers to the type of card
    pub card_type_id: usize,
    /// Unique id of the card
    pub id: usize,
}

/// Used in the request body to create a card
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct CardCreate {
    /// Refers to the type of card
    pub(crate) card_type_id: usize,
}

#[openapi]
#[get("/cards")]
pub async fn list_all_cards(player_data: &State<PLayerData>) -> Json<Vec<Card>> {
    Json(player_data.cards.lock().await.clone())
}

#[openapi]
#[get("/cards/<id>")]
pub async fn get_card_json(id: usize, player_data: &State<PLayerData>) -> Result<Json<Card>, NotFound<Json<Status>>> {
    get_card(id, player_data)
        .await
        .map(|existing| Json(existing.clone()))
        .ok_or(NotFound(new_status(format!("Card with id {} does not exist!", id))))
}

pub async fn get_card(id: usize, player_data: &State<PLayerData>) -> Option<Card> {
    player_data.cards.lock().await.iter()
        .find(|existing| existing.id == id)
        .cloned()
}

#[openapi]
#[post("/cards", format = "json", data = "<new_card>")]
pub async fn create_card(new_card: Json<CardCreate>, player_data: &State<PLayerData>) -> Created<&str> {
    let the_card = new_card.0;
    let unused_id = *player_data.cards.lock().await.iter()
        .map(|existing| existing.id)
        .max()
        .map(|existing_id| existing_id + 1)
        .get_or_insert(0);
    player_data.cards.lock().await.push(
        Card {
            card_type_id: the_card.card_type_id,
            id: unused_id,
        }
    );
    let location = uri!(get_card_json(unused_id));
    Created::new(location.to_string())
}
