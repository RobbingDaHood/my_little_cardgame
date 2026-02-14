use rocket::State;
use rocket::serde::json::Json;
use rocket_okapi::{openapi, JsonSchema};

use crate::player_data::PlayerData;
use crate::deck::token::Token;

#[openapi]
#[get("/player/tokens")]
pub async fn get_player_tokens(player_data: &State<PlayerData>) -> Json<Vec<Token>> {
    Json(player_data.tokens.lock().await.clone())
}
