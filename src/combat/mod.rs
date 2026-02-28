use rocket::response::status::{Created, NotFound};
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;

use crate::library::types::{EncounterOutcome, EncounterState};
use crate::player_data::RandomGeneratorWrapper;
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

/// Initialize combat for testing purposes.
///
/// **TESTING ENDPOINT ONLY** - Creates combat from the first CombatEncounter
/// card found in the Library (gnome at index 3).
#[openapi]
#[post("/tests/combat")]
pub async fn initialize_combat(
    player_data: &State<RandomGeneratorWrapper>,
    game_state: &State<std::sync::Arc<rocket::futures::lock::Mutex<crate::library::GameState>>>,
) -> Created<&'static str> {
    let mut gs = game_state.lock().await;
    // Initialize player health token if not set
    if gs
        .token_balances
        .get(&crate::library::types::Token::persistent(
            crate::library::types::TokenType::Health,
        ))
        .copied()
        .unwrap_or(0)
        == 0
    {
        gs.token_balances.insert(
            crate::library::types::Token::persistent(crate::library::types::TokenType::Health),
            20,
        );
    }
    let mut rng = player_data.random_generator_state.lock().await;
    let _ = gs.start_combat(11, &mut rng);
    Created::new("/tests/combat")
}

/// Enemy play for testing purposes.
///
/// **TESTING ENDPOINT ONLY**
#[post("/tests/combat/enemy_play")]
pub async fn enemy_play(
    player_data: &State<RandomGeneratorWrapper>,
    game_state: &State<std::sync::Arc<rocket::futures::lock::Mutex<crate::library::GameState>>>,
) -> Created<&'static str> {
    let mut gs = game_state.lock().await;
    if gs.current_encounter.is_none() {
        return Created::new("/tests/combat/enemy_play");
    }
    let mut rng = player_data.random_generator_state.lock().await;
    let _ = gs.resolve_enemy_play(&mut rng);
    Created::new("/tests/combat/enemy_play")
}

/// Advance combat phase for testing purposes.
///
/// **TESTING ENDPOINT ONLY**
#[post("/tests/combat/advance")]
pub async fn advance_phase(
    game_state: &State<std::sync::Arc<rocket::futures::lock::Mutex<crate::library::GameState>>>,
) -> Created<&'static str> {
    let mut gs = game_state.lock().await;
    let _ = gs.advance_combat_phase();
    Created::new("/tests/combat/advance")
}

#[openapi]
#[get("/encounter/results")]
pub async fn get_encounter_results(
    game_state: &State<std::sync::Arc<rocket::futures::lock::Mutex<crate::library::GameState>>>,
) -> Json<Vec<EncounterOutcome>> {
    let gs = game_state.lock().await;
    Json(gs.encounter_results.clone())
}
