use super::game_state::GameState;
use super::types::CardKind;
use rocket::serde::json::Json;
use rocket_okapi::openapi;

/// A library card with its ID (index in the library).
#[derive(
    Debug, Clone, rocket::serde::Serialize, rocket::serde::Deserialize, rocket_okapi::JsonSchema,
)]
#[serde(crate = "rocket::serde")]
pub struct LibraryCardWithId {
    pub id: usize,
    #[serde(flatten)]
    pub card: super::types::LibraryCard,
}

/// Library cards endpoint: returns all cards from the canonical Library.
/// Optionally filter by ?location= (Library, Deck, Hand, Discard)
/// and ?card_kind= (Attack, Defence, Resource, Mining, Encounter, PlayerCardEffect, EnemyCardEffect).
#[openapi]
#[get("/library/cards?<location>&<card_kind>")]
pub async fn list_library_cards(
    location: Option<String>,
    card_kind: Option<String>,
    game_state: &rocket::State<std::sync::Arc<rocket::futures::lock::Mutex<GameState>>>,
) -> Json<Vec<LibraryCardWithId>> {
    let gs = game_state.lock().await;
    let cards: Vec<LibraryCardWithId> = gs
        .library
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| match location.as_deref() {
            Some("Library") => c.counts.library > 0,
            Some("Deck") => c.counts.deck > 0,
            Some("Hand") => c.counts.hand > 0,
            Some("Discard") => c.counts.discard > 0,
            _ => true,
        })
        .filter(|(_, c)| match card_kind.as_deref() {
            Some("Attack") => matches!(c.kind, CardKind::Attack { .. }),
            Some("Defence") => matches!(c.kind, CardKind::Defence { .. }),
            Some("Resource") => matches!(c.kind, CardKind::Resource { .. }),
            Some("Mining") => matches!(c.kind, CardKind::Mining { .. }),
            Some("Herbalism") => matches!(c.kind, CardKind::Herbalism { .. }),
            Some("Encounter") => matches!(c.kind, CardKind::Encounter { .. }),
            Some("PlayerCardEffect") => matches!(c.kind, CardKind::PlayerCardEffect { .. }),
            Some("EnemyCardEffect") => matches!(c.kind, CardKind::EnemyCardEffect { .. }),
            _ => true,
        })
        .map(|(id, c)| LibraryCardWithId {
            id,
            card: c.clone(),
        })
        .collect();
    Json(cards)
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
