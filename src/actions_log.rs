use rocket::serde::json::Json;
use rocket_okapi::openapi;

use crate::library::{GameState, types::ActionEntry};

#[openapi]
#[get("/actions/log")]
pub async fn list_actions_log(
    game_state: &rocket::State<std::sync::Arc<rocket::futures::lock::Mutex<GameState>>>,
) -> Json<Vec<ActionEntry>> {
    let gs = game_state.lock().await;
    Json(gs.action_log.entries())
}
