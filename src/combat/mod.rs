use rocket::response::status::NotFound;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;

use crate::library::types::{EncounterOutcome, EncounterState};
use crate::status_messages::{new_status, Status};

/// Current encounter state for the active encounter.
///
/// Returns the full encounter state including discipline-specific details (combat
/// health/phase, mining light level, fishing range, etc.). Use this to inspect
/// the board and make informed card-play decisions. Returns 404 when no encounter
/// is active (between encounters or before starting a game).
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

/// History of encounter outcomes (win/loss) for the current session.
///
/// Returns a chronological list of encounter results. Use this to track your
/// win/loss record across encounters. For richer statistics including per-discipline
/// breakdowns and token flow analysis, see `GET /metrics` instead.
#[openapi]
#[get("/encounter/results")]
pub async fn get_encounter_results(
    game_state: &State<std::sync::Arc<rocket::futures::lock::Mutex<crate::library::GameState>>>,
) -> Json<Vec<EncounterOutcome>> {
    let gs = game_state.lock().await;
    Json(gs.encounter_results.clone())
}
