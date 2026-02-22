use rocket::response::status::NotFound;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::{openapi, JsonSchema};

use crate::area_deck::AreaDeck;
use crate::player_data::PlayerData;
use crate::status_messages::{new_status, Status};

/// Response wrapper for area deck details
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct AreaDeckResponse {
    pub area_deck: AreaDeck,
}

/// Get the current area deck (player's current location)
#[openapi]
#[get("/area")]
pub async fn get_area(
    player_data: &State<PlayerData>,
) -> Result<Json<AreaDeckResponse>, NotFound<Json<Status>>> {
    let area_deck = player_data.current_area_deck.lock().await;
    match area_deck.clone() {
        Some(deck) => Ok(Json(AreaDeckResponse { area_deck: deck })),
        None => Err(NotFound(new_status("No current area set".to_string()))),
    }
}

/// List encounter card IDs in the current area deck
#[openapi]
#[get("/area/encounters")]
pub async fn get_area_encounters(
    player_data: &State<PlayerData>,
) -> Result<Json<Vec<usize>>, NotFound<Json<Status>>> {
    let area_deck = player_data.current_area_deck.lock().await;
    match area_deck.as_ref() {
        Some(deck) => Ok(Json(deck.encounter_card_ids.clone())),
        None => Err(NotFound(new_status("No current area set".to_string()))),
    }
}
