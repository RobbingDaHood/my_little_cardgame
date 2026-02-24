use super::game_state::GameState;
use super::registry::TokenRegistry;
use super::types::CardKind;
use rocket::serde::json::Json;
use rocket_okapi::openapi;

/// Get canonical token registry with full token details
#[openapi]
#[get("/tokens")]
pub async fn list_library_tokens() -> Json<Vec<super::types::TokenRegistryEntry>> {
    let reg = TokenRegistry::with_canonical();
    Json(reg.tokens.into_values().collect())
}

/// Library cards endpoint: returns all cards from the canonical Library.
#[openapi]
#[get("/library/cards")]
pub async fn list_library_cards(
    game_state: &rocket::State<std::sync::Arc<rocket::futures::lock::Mutex<GameState>>>,
) -> Json<Vec<super::types::LibraryCard>> {
    let gs = game_state.lock().await;
    Json(gs.library.cards.clone())
}

/// Test endpoint: add a card to the Library with specified kind and counts.
#[openapi]
#[post("/tests/library/cards", data = "<card>")]
pub async fn add_test_library_card(
    card: Json<super::types::LibraryCard>,
    game_state: &rocket::State<std::sync::Arc<rocket::futures::lock::Mutex<GameState>>>,
) -> rocket::response::status::Created<String> {
    let mut gs = game_state.lock().await;
    let id = gs.library.add_card(card.0.kind, card.0.counts);
    rocket::response::status::Created::new(format!("/library/cards/{}", id))
}

/// A single card effect entry with its library ID.
#[derive(
    Debug, Clone, rocket::serde::Serialize, rocket::serde::Deserialize, rocket_okapi::JsonSchema,
)]
#[serde(crate = "rocket::serde")]
pub struct CardEffectEntry {
    pub id: usize,
    pub card: super::types::LibraryCard,
}

/// Response for the card effects endpoint.
#[derive(
    Debug, Clone, rocket::serde::Serialize, rocket::serde::Deserialize, rocket_okapi::JsonSchema,
)]
#[serde(crate = "rocket::serde")]
pub struct CardEffectsResponse {
    pub player_effects: Vec<CardEffectEntry>,
    pub enemy_effects: Vec<CardEffectEntry>,
}

/// List all CardEffect deck entries (player and enemy).
#[openapi]
#[get("/library/card-effects")]
pub async fn list_card_effects(
    game_state: &rocket::State<std::sync::Arc<rocket::futures::lock::Mutex<GameState>>>,
) -> Json<CardEffectsResponse> {
    let gs = game_state.lock().await;
    let player_effects: Vec<CardEffectEntry> = gs
        .library
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| matches!(c.kind, CardKind::PlayerCardEffect { .. }))
        .map(|(i, c)| CardEffectEntry {
            id: i,
            card: c.clone(),
        })
        .collect();
    let enemy_effects: Vec<CardEffectEntry> = gs
        .library
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| matches!(c.kind, CardKind::EnemyCardEffect { .. }))
        .map(|(i, c)| CardEffectEntry {
            id: i,
            card: c.clone(),
        })
        .collect();
    Json(CardEffectsResponse {
        player_effects,
        enemy_effects,
    })
}
