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

/// Current token balances for the player.
///
/// Returns all token types and their values. Tokens include persistent resources
/// (Health, Stamina, materials like Ore/Plant/Lumber/Fish), combat tokens (Shield,
/// Dodge, Mana), durability tokens, Insight tokens, and hand size limits. Use this
/// to check resource levels before choosing encounters and to monitor the impact
/// of card plays. Token balances reset partially on death (materials lost, Health/
/// Stamina restored) and fully on NewGame.
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
