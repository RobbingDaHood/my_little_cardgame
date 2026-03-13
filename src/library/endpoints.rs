use super::game_state::GameState;
use super::types::{CardKind, EncounterPhase, EncounterState};
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
/// and ?card_kind= (Attack, Defence, Resource, Rest, Mining, Encounter, PlayerCardEffect, EnemyCardEffect).
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
            Some("Woodcutting") => matches!(c.kind, CardKind::Woodcutting { .. }),
            Some("Fishing") => matches!(c.kind, CardKind::Fishing { .. }),
            Some("Rest") => matches!(c.kind, CardKind::Rest { .. }),
            Some("Encounter") => matches!(c.kind, CardKind::Encounter { .. }),
            Some("PlayerCardEffect") => matches!(c.kind, CardKind::PlayerCardEffect { .. }),
            Some("EnemyCardEffect") => matches!(c.kind, CardKind::EnemyCardEffect { .. }),
            Some("Crafting") => matches!(c.kind, CardKind::Crafting { .. }),
            _ => true,
        })
        .map(|(id, c)| LibraryCardWithId {
            id,
            card: c.clone(),
        })
        .collect();
    Json(cards)
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

/// A possible player action with optional playable card IDs.
#[derive(
    Debug, Clone, rocket::serde::Serialize, rocket::serde::Deserialize, rocket_okapi::JsonSchema,
)]
#[serde(crate = "rocket::serde")]
pub struct PossibleAction {
    pub action_type: String,
    pub playable_card_ids: Vec<usize>,
}

/// Returns the list of currently valid player actions based on game state.
#[openapi]
#[get("/actions/possible")]
pub async fn get_possible_actions(
    game_state: &rocket::State<std::sync::Arc<rocket::futures::lock::Mutex<GameState>>>,
) -> Json<Vec<PossibleAction>> {
    let gs = game_state.lock().await;
    let mut actions = Vec::new();

    actions.push(PossibleAction {
        action_type: "NewGame".to_string(),
        playable_card_ids: vec![],
    });

    match gs.encounter_phase {
        EncounterPhase::NoEncounter => {
            let encounter_ids: Vec<usize> = gs
                .library
                .cards
                .iter()
                .enumerate()
                .filter(|(_, c)| matches!(c.kind, CardKind::Encounter { .. }) && c.counts.hand > 0)
                .map(|(id, _)| id)
                .collect();
            if !encounter_ids.is_empty() {
                actions.push(PossibleAction {
                    action_type: "EncounterPickEncounter".to_string(),
                    playable_card_ids: encounter_ids,
                });
            }
        }
        EncounterPhase::InEncounter => {
            if let Some(ref enc) = gs.current_encounter {
                let playable_ids = playable_card_ids_for_encounter(enc, &gs);
                if !playable_ids.is_empty() {
                    actions.push(PossibleAction {
                        action_type: "EncounterPlayCard".to_string(),
                        playable_card_ids: playable_ids,
                    });
                }

                match enc {
                    EncounterState::Combat(_) => {
                        // Combat cannot be aborted
                    }
                    EncounterState::Research(_) => {
                        actions.push(PossibleAction {
                            action_type: "ResearchChooseProject".to_string(),
                            playable_card_ids: vec![],
                        });
                        actions.push(PossibleAction {
                            action_type: "ResearchSelectCandidate".to_string(),
                            playable_card_ids: vec![],
                        });
                        actions.push(PossibleAction {
                            action_type: "ResearchProgress".to_string(),
                            playable_card_ids: vec![],
                        });
                        actions.push(PossibleAction {
                            action_type: "EncounterAbort".to_string(),
                            playable_card_ids: vec![],
                        });
                        actions.push(PossibleAction {
                            action_type: "EncounterConcludeEncounter".to_string(),
                            playable_card_ids: vec![],
                        });
                    }
                    EncounterState::Crafting(_) => {
                        actions.push(PossibleAction {
                            action_type: "EncounterCraftSwap".to_string(),
                            playable_card_ids: vec![],
                        });
                        actions.push(PossibleAction {
                            action_type: "EncounterCraftCard".to_string(),
                            playable_card_ids: vec![],
                        });
                        actions.push(PossibleAction {
                            action_type: "EncounterCraftDurability".to_string(),
                            playable_card_ids: vec![],
                        });
                        actions.push(PossibleAction {
                            action_type: "EncounterAbort".to_string(),
                            playable_card_ids: vec![],
                        });
                        actions.push(PossibleAction {
                            action_type: "EncounterConcludeEncounter".to_string(),
                            playable_card_ids: vec![],
                        });
                    }
                    _ => {
                        actions.push(PossibleAction {
                            action_type: "EncounterAbort".to_string(),
                            playable_card_ids: vec![],
                        });
                        actions.push(PossibleAction {
                            action_type: "EncounterConcludeEncounter".to_string(),
                            playable_card_ids: vec![],
                        });
                    }
                }
            }
        }
        EncounterPhase::Scouting => {
            let encounter_ids: Vec<usize> = gs
                .library
                .cards
                .iter()
                .enumerate()
                .filter(|(_, c)| matches!(c.kind, CardKind::Encounter { .. }) && c.counts.hand > 0)
                .map(|(id, _)| id)
                .collect();
            actions.push(PossibleAction {
                action_type: "EncounterApplyScouting".to_string(),
                playable_card_ids: encounter_ids,
            });
        }
    }

    Json(actions)
}

fn playable_card_ids_for_encounter(encounter: &EncounterState, gs: &GameState) -> Vec<usize> {
    gs.library
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| c.counts.hand > 0)
        .filter(|(_, c)| match encounter {
            EncounterState::Combat(combat) => (combat.phase.allowed_card_kind())(&c.kind),
            EncounterState::Mining(_) => matches!(c.kind, CardKind::Mining { .. }),
            EncounterState::Herbalism(_) => matches!(c.kind, CardKind::Herbalism { .. }),
            EncounterState::Woodcutting(_) => matches!(c.kind, CardKind::Woodcutting { .. }),
            EncounterState::Fishing(_) => matches!(c.kind, CardKind::Fishing { .. }),
            EncounterState::Rest(_) => matches!(c.kind, CardKind::Rest { .. }),
            EncounterState::Crafting(_) => matches!(c.kind, CardKind::Crafting { .. }),
            EncounterState::Research(_) => false,
        })
        .map(|(id, _)| id)
        .collect()
}
