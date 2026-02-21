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

/// Response wrapper for encounter list
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct EncountersResponse {
    pub encounters: Vec<crate::area_deck::Encounter>,
}

/// Get area deck by ID
#[openapi]
#[get("/area/<area_id>")]
pub async fn get_area(
    player_data: &State<PlayerData>,
    area_id: &str,
) -> Result<Json<AreaDeckResponse>, NotFound<Json<Status>>> {
    let area_decks = player_data.area_decks.lock().await;
    match area_decks.get(area_id) {
        Some(area_deck) => Ok(Json(AreaDeckResponse {
            area_deck: area_deck.clone(),
        })),
        None => Err(NotFound(new_status(format!("Area {} not found", area_id)))),
    }
}

/// List all encounters in an area deck
#[openapi]
#[get("/area/<area_id>/encounters")]
pub async fn get_area_encounters(
    player_data: &State<PlayerData>,
    area_id: &str,
) -> Result<Json<EncountersResponse>, NotFound<Json<Status>>> {
    let area_decks = player_data.area_decks.lock().await;
    match area_decks.get(area_id) {
        Some(area_deck) => Ok(Json(EncountersResponse {
            encounters: area_deck.encounters.clone(),
        })),
        None => Err(NotFound(new_status(format!("Area {} not found", area_id)))),
    }
}
