use rocket::response::status::NotFound;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;

use crate::library::types::{EncounterOutcome, EncounterState};
use crate::status_messages::{new_status, Status};

#[openapi]
#[get("/encounter")]
pub async fn get_encounter(
    game_state: &State<std::sync::Arc<rocket::futures::lock::Mutex<crate::library::GameState>>>,
) -> Result<Json<EncounterState>, NotFound<Json<Status>>> {
    let gs = game_state.lock().await;
    match &gs.current_encounter {
        Some(c) => Ok(Json(c.clone())),
        None => Err(NotFound(new_status("No active encounter".to_string()))),
    }
}

#[openapi]
#[get("/encounter/results")]
pub async fn get_encounter_results(
    game_state: &State<std::sync::Arc<rocket::futures::lock::Mutex<crate::library::GameState>>>,
) -> Json<Vec<EncounterOutcome>> {
    let gs = game_state.lock().await;
    Json(gs.encounter_results.clone())
}
