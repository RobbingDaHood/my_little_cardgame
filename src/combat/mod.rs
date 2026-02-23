use rocket::response::status::{Created, NotFound};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use rocket_okapi::{openapi, JsonSchema};

use crate::library::types::{CombatOutcome, CombatSnapshot};
use crate::player_data::PlayerData;
use crate::status_messages::{new_status, Status};

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
    let _ = gs.start_combat(3);
    Created::new("/tests/combat")
}

/// Enemy play for testing purposes.
///
/// **TESTING ENDPOINT ONLY**
#[post("/tests/combat/enemy_play")]
pub async fn enemy_play(
    player_data: &State<PlayerData>,
    game_state: &State<std::sync::Arc<rocket::futures::lock::Mutex<crate::library::GameState>>>,
) -> Created<&'static str> {
    let mut gs = game_state.lock().await;
    if gs.current_combat.is_none() {
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
#[get("/combat/result")]
pub async fn get_combat_result(
    game_state: &State<std::sync::Arc<rocket::futures::lock::Mutex<crate::library::GameState>>>,
) -> Result<Json<CombatOutcome>, NotFound<Json<Status>>> {
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
) -> Json<SimulateCombatResponse> {
    let (snapshot, player_tokens) = crate::library::combat::simulate_combat(
        request.initial_state.clone(),
        request.player_tokens.clone(),
        request.seed,
        request.actions.clone(),
        &request.card_defs,
    );
    Json(SimulateCombatResponse {
        snapshot,
        player_tokens,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct SimulateCombatRequest {
    pub initial_state: CombatSnapshot,
    #[serde(with = "crate::library::types::token_map_serde")]
    #[schemars(with = "crate::library::types::token_map_serde::SchemaHelper")]
    pub player_tokens: std::collections::HashMap<crate::library::types::Token, i64>,
    pub seed: u64,
    pub actions: Vec<crate::library::types::CombatAction>,
    pub card_defs: std::collections::HashMap<u64, crate::library::types::CardDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct SimulateCombatResponse {
    pub snapshot: CombatSnapshot,
    #[serde(with = "crate::library::types::token_map_serde")]
    #[schemars(with = "crate::library::types::token_map_serde::SchemaHelper")]
    pub player_tokens: std::collections::HashMap<crate::library::types::Token, i64>,
}
