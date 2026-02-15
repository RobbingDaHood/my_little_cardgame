use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;

use crate::deck::token::Token;
use crate::player_data::PlayerData;

#[openapi]
#[get("/player/tokens")]
pub async fn get_player_tokens(player_data: &State<PlayerData>) -> Json<Vec<Token>> {
    Json(player_data.tokens.lock().await.clone())
}
