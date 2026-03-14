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
            Some("Research") => matches!(c.kind, CardKind::Research { .. }),
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

/// Returns the list of currently valid player actions based on game state.
/// Each entry is a `PlayerActions` variant with placeholder field values
/// to illustrate the expected payload shape.
#[openapi]
#[get("/actions/possible")]
pub async fn get_possible_actions(
    game_state: &rocket::State<std::sync::Arc<rocket::futures::lock::Mutex<GameState>>>,
) -> Json<Vec<crate::action::PlayerActions>> {
    let gs = game_state.lock().await;
    let mut actions: Vec<crate::action::PlayerActions> = Vec::new();

    actions.push(crate::action::PlayerActions::NewGame { seed: None });

    match gs.encounter_phase {
        EncounterPhase::NoEncounter => {
            let has_encounter = !gs.library.encounter_hand().is_empty();
            if has_encounter {
                actions.push(crate::action::PlayerActions::EncounterPickEncounter { card_id: 0 });
            }
            let has_milestone = !gs.library.milestone_hand().is_empty();
            if has_milestone {
                actions.push(crate::action::PlayerActions::EncounterPickEncounter { card_id: 0 });
            }
        }
        EncounterPhase::InEncounter => {
            if let Some(ref enc) = gs.current_encounter {
                let has_playable = !playable_card_ids_for_encounter(enc, &gs).is_empty();
                if has_playable {
                    actions.push(crate::action::PlayerActions::EncounterPlayCard { card_id: 0 });
                }

                match enc {
                    EncounterState::Combat(_) => {
                        // Combat cannot be aborted
                    }
                    EncounterState::Research(_) => {
                        actions.push(crate::action::PlayerActions::ResearchChooseProject {
                            discipline: super::types::Discipline::Combat,
                            tier_count: 0,
                        });
                        actions.push(crate::action::PlayerActions::ResearchSelectCandidate {
                            candidate_index: 0,
                        });
                        actions.push(crate::action::PlayerActions::ResearchProgress { amount: 0 });
                        actions.push(crate::action::PlayerActions::EncounterAbort);
                        actions.push(crate::action::PlayerActions::EncounterConcludeEncounter);
                    }
                    EncounterState::Crafting(_) => {
                        actions.push(crate::action::PlayerActions::EncounterCraftSwap {
                            from_id: 0,
                            to_id: 0,
                        });
                        actions.push(crate::action::PlayerActions::EncounterCraftCard {
                            target_card_id: 0,
                        });
                        actions.push(crate::action::PlayerActions::EncounterCraftDurability {
                            discipline: String::new(),
                        });
                        actions.push(crate::action::PlayerActions::EncounterAbort);
                        actions.push(crate::action::PlayerActions::EncounterConcludeEncounter);
                    }
                    EncounterState::Milestone(_) => {
                        actions.push(crate::action::PlayerActions::EncounterAbort);
                    }
                    _ => {
                        actions.push(crate::action::PlayerActions::EncounterAbort);
                        actions.push(crate::action::PlayerActions::EncounterConcludeEncounter);
                    }
                }
            }
        }
        EncounterPhase::Scouting => {
            actions.push(crate::action::PlayerActions::EncounterApplyScouting { card_ids: vec![] });
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
            EncounterState::Research(r) => {
                r.experiment_active && matches!(c.kind, CardKind::Research { .. })
            }
            EncounterState::Milestone(m) => match m.inner_state.as_ref() {
                EncounterState::Combat(combat) => (combat.phase.allowed_card_kind())(&c.kind),
                EncounterState::Mining(_) => matches!(c.kind, CardKind::Mining { .. }),
                EncounterState::Herbalism(_) => matches!(c.kind, CardKind::Herbalism { .. }),
                EncounterState::Woodcutting(_) => matches!(c.kind, CardKind::Woodcutting { .. }),
                EncounterState::Fishing(_) => matches!(c.kind, CardKind::Fishing { .. }),
                _ => false,
            },
        })
        .map(|(id, _)| id)
        .collect()
}
