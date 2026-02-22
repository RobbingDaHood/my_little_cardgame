use rocket::response::status::{Created, NotFound};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use rocket_okapi::{openapi, JsonSchema};

use crate::library::types::{CombatResult, CombatSnapshot};
use crate::player_data::PlayerData;
use crate::status_messages::{new_status, Status};

pub mod resolve;

#[openapi]
#[get("/combat")]
pub async fn get_combat(
    game_state: &State<std::sync::Arc<rocket::futures::lock::Mutex<crate::library::GameState>>>,
) -> Result<Json<CombatSnapshot>, NotFound<Json<Status>>> {
    let gs = game_state.lock().await;
    match &gs.current_combat {
        Some(c) => Ok(Json(c.clone())),
        None => Err(NotFound(new_status(
            "Combat has not been initialized".to_string(),
        ))),
    }
}

/// Initialize combat for testing purposes.
///
/// **TESTING ENDPOINT ONLY** - Creates combat from the first CombatEncounter
/// card found in the Library (gnome at index 3).
#[openapi]
#[post("/tests/combat")]
pub async fn initialize_combat(
    game_state: &State<std::sync::Arc<rocket::futures::lock::Mutex<crate::library::GameState>>>,
) -> Created<&str> {
    let mut gs = game_state.lock().await;
    // Initialize player health token if not set
    if gs
        .token_balances
        .get(&crate::library::types::TokenId::Health)
        .copied()
        .unwrap_or(0)
        == 0
    {
        gs.token_balances
            .insert(crate::library::types::TokenId::Health, 20);
    }
    let _ = gs.start_combat(3);
    Created::new("/tests/combat")
}

#[openapi]
#[post("/combat/enemy_play")]
pub async fn enemy_play(
    player_data: &State<PlayerData>,
    game_state: &State<std::sync::Arc<rocket::futures::lock::Mutex<crate::library::GameState>>>,
) -> Created<&'static str> {
    let mut gs = game_state.lock().await;
    if gs.current_combat.is_none() {
        return Created::new("/combat/enemy_play");
    }
    let mut rng = player_data.random_generator_state.lock().await;
    let _ = gs.resolve_enemy_play(&mut rng);
    Created::new("/combat/enemy_play")
}

#[openapi]
#[post("/combat/advance")]
pub async fn advance_phase(
    game_state: &State<std::sync::Arc<rocket::futures::lock::Mutex<crate::library::GameState>>>,
) -> Created<&'static str> {
    let mut gs = game_state.lock().await;
    let _ = gs.advance_combat_phase();
    Created::new("/combat/advance")
}

#[openapi]
#[get("/combat/result")]
pub async fn get_combat_result(
    game_state: &State<std::sync::Arc<rocket::futures::lock::Mutex<crate::library::GameState>>>,
) -> Result<Json<CombatResult>, NotFound<Json<Status>>> {
    let gs = game_state.lock().await;
    match &gs.last_combat_result {
        Some(r) => Ok(Json(r.clone())),
        None => Err(NotFound(new_status(
            "No combat result available".to_string(),
        ))),
    }
}

/// Simulate a deterministic combat encounter from a seed and initial state.
///
/// **TESTING ENDPOINT ONLY** — This endpoint is temporary and should not be
/// used in production. It bypasses the single mutator action endpoint.
#[openapi]
#[post("/tests/combat/simulate", format = "json", data = "<request>")]
pub async fn simulate_combat_endpoint(
    request: Json<SimulateCombatRequest>,
) -> Json<CombatSnapshot> {
    let result = crate::library::combat::simulate_combat(
        request.initial_state.clone(),
        request.seed,
        request.actions.clone(),
        &request.card_defs,
    );
    Json(result)
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct SimulateCombatRequest {
    pub initial_state: CombatSnapshot,
    pub seed: u64,
    pub actions: Vec<crate::library::types::CombatAction>,
    pub card_defs: std::collections::HashMap<u64, crate::library::types::CardDef>,
}
