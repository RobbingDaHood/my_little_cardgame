use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use rocket_okapi::openapi;
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct TokenBalance {
    pub token: crate::library::types::Token,
    pub value: i64,
}

#[openapi]
#[get("/player/tokens")]
pub async fn get_player_tokens(
    game_state: &State<std::sync::Arc<rocket::futures::lock::Mutex<crate::library::GameState>>>,
) -> Json<Vec<TokenBalance>> {
    let gs = game_state.lock().await;
    let balances: Vec<TokenBalance> = gs
        .token_balances
        .iter()
        .map(|(k, v)| TokenBalance {
            token: k.clone(),
            value: *v,
        })
        .collect();
    Json(balances)
}
