use rocket::response::status::NotFound;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::{openapi, JsonSchema};

use crate::status_messages::Status;

/// Response wrapper for area encounter details
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct AreaEncountersResponse {
    pub hand: Vec<usize>,
    pub deck_count: usize,
    pub discard_count: usize,
}

/// Get the current area (encounter cards tracked in Library)
#[openapi]
#[get("/area")]
pub async fn get_area(
    game_state: &State<std::sync::Arc<rocket::futures::lock::Mutex<crate::library::GameState>>>,
) -> Result<Json<AreaEncountersResponse>, NotFound<Json<Status>>> {
    let gs = game_state.lock().await;
    let hand = gs.library.encounter_hand();
    let deck_count: usize = gs
        .library
        .cards_matching(|k| matches!(k, crate::library::types::CardKind::CombatEncounter { .. }))
        .iter()
        .map(|(_, c)| c.counts.deck as usize)
        .sum();
    let discard_count: usize = gs
        .library
        .cards_matching(|k| matches!(k, crate::library::types::CardKind::CombatEncounter { .. }))
        .iter()
        .map(|(_, c)| c.counts.discard as usize)
        .sum();
    Ok(Json(AreaEncountersResponse {
        hand,
        deck_count,
        discard_count,
    }))
}

/// List encounter card IDs in the current area hand
#[openapi]
#[get("/area/encounters")]
pub async fn get_area_encounters(
    game_state: &State<std::sync::Arc<rocket::futures::lock::Mutex<crate::library::GameState>>>,
) -> Result<Json<Vec<usize>>, NotFound<Json<Status>>> {
    let gs = game_state.lock().await;
    Ok(Json(gs.library.encounter_hand()))
}
