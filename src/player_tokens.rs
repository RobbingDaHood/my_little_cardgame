use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;

use std::collections::HashMap;

#[openapi]
#[get("/player/tokens")]
pub async fn get_player_tokens(
    game_state: &State<std::sync::Arc<rocket::futures::lock::Mutex<crate::library::GameState>>>,
) -> Json<HashMap<String, i64>> {
    let gs = game_state.lock().await;
    Json(gs.token_balances.clone())
}
