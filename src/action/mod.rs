use either::{Either, Left, Right};
use rocket::response::status::{BadRequest, NotFound};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use rocket_okapi::{openapi, JsonSchema};

pub mod persistence;

use crate::combat::{Combat, States};
use crate::deck::card::{get_card, CardType};
use crate::deck::CardState;
use crate::player_data::PlayerData;
use crate::status_messages::{new_status, Status};

use rand::SeedableRng;
use rand_pcg::Lcg64Xsh32;

/// Player actions
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema, Hash)]
#[serde(crate = "rocket::serde", tag = "action_type")]
pub enum PlayerActions {
    PlayCard {
        card_id: usize,
    },
    GrantToken {
        token_id: String,
        amount: i64,
    },
    SetSeed {
        seed: u64,
    },
    DrawEncounter {
        area_id: String,
        encounter_id: String,
    },
    ReplaceEncounter {
        area_id: String,
        old_encounter_id: String,
        new_encounter_id: String,
    },
    ApplyScouting {
        area_id: String,
        parameters: String,
    },
    // Encounter actions (Step 7)
    EncounterPickEncounter {
        card_id: String,
    },
    EncounterPlayCard {
        card_id: u64,
        effects: Vec<String>,
    },
    EncounterApplyScouting {
        choice_id: String,
        parameters: serde_json::Value,
    },
    EncounterFinish,
}

#[openapi]
#[post("/action", format = "json", data = "<player_action>")]
pub async fn play(
    player_data: &State<PlayerData>,
    game_state: &State<std::sync::Arc<rocket::futures::lock::Mutex<crate::library::GameState>>>,
    player_action: Json<PlayerActions>,
) -> Result<
    (
        rocket::http::Status,
        Json<crate::library::types::ActionEntry>,
    ),
    Either<NotFound<Json<Status>>, BadRequest<Json<Status>>>,
> {
    let action = player_action.0;

    match action {
        PlayerActions::GrantToken { token_id, amount } => {
            let mut gs = game_state.lock().await;
            match gs.apply_grant(&token_id, amount, None) {
                Ok(entry) => Ok((rocket::http::Status::Created, Json(entry))),
                Err(e) => Err(Right(BadRequest(new_status(e)))),
            }
        }
        PlayerActions::SetSeed { seed } => {
            let gs = game_state.lock().await;
            // append to action log
            let payload = crate::library::types::ActionPayload::SetSeed { seed };
            let log_arc = std::sync::Arc::clone(&gs.action_log);
            let entry = log_arc.append_async("SetSeed", payload).await;
            // apply to PlayerData RNG/seed
            let s = seed;
            let mut seed_bytes: [u8; 16] = [0u8; 16];
            seed_bytes[0..8].copy_from_slice(&s.to_le_bytes());
            seed_bytes[8..16].copy_from_slice(&s.to_le_bytes());
            *player_data.seed.lock().await = seed_bytes;
            let new_rng = Lcg64Xsh32::from_seed(seed_bytes);
            *player_data.random_generator_state.lock().await = new_rng;
            Ok((rocket::http::Status::Created, Json(entry)))
        }
        PlayerActions::PlayCard { card_id } => {
            let combat_optional: Option<Combat> = *player_data.current_combat.lock().await.clone();
            match combat_optional {
                Some(combat) => {
                    // check card exists and is of allowed type for the phase
                    match get_card(card_id, player_data).await {
                        None => Err(Left(NotFound(new_status(format!(
                            "Card {:?} does not exist!",
                            card_id
                        ))))),
                        Some(card) => {
                            let allowed = match combat.state {
                                States::Defending => CardType::Defence,
                                States::Attacking => CardType::Attack,
                                States::Resourcing => CardType::Resource,
                            };
                            if card.card_type != allowed {
                                return Err(Right(BadRequest(new_status(format!(
                                    "Card with id {} is not playable in current phase",
                                    card_id
                                )))));
                            }
                            let deck_id = match combat.state {
                                States::Defending => *player_data.defence_deck_id.lock().await,
                                States::Attacking => *player_data.attack_deck_id.lock().await,
                                States::Resourcing => *player_data.resource_deck_id.lock().await,
                            };
                            // find deck and change card state Hand -> Discard
                            let mut decks = player_data.decks.lock().await;
                            match decks.iter_mut().find(|deck| deck.id == deck_id) {
                                None => Err(Left(NotFound(new_status(format!(
                                    "Card with id {} does not exist in deck!",
                                    card_id
                                ))))),
                                Some(deck) => {
                                    match deck.change_card_state(
                                        card_id,
                                        CardState::Discard,
                                        CardState::Hand,
                                    ) {
                                        Ok(()) => {
                                            crate::combat::resolve::resolve_card_effects(
                                                card_id,
                                                true,
                                                player_data,
                                            )
                                            .await;
                                            // append PlayCard action
                                            let gs = game_state.lock().await;
                                            let payload =
                                                crate::library::types::ActionPayload::PlayCard {
                                                    card_id,
                                                    deck_id: Some(deck_id.to_string()),
                                                    reason: None,
                                                };
                                            let entry = gs.append_action("PlayCard", payload);
                                            Ok((rocket::http::Status::Created, Json(entry)))
                                        }
                                        Err(e) => Err(Left(e)),
                                    }
                                }
                            }
                        }
                    }
                }
                None => Err(Right(BadRequest(new_status(
                    "Cannot play a card if there are no active combat!".to_string(),
                )))),
            }
        }
        PlayerActions::DrawEncounter {
            area_id: _,
            encounter_id,
        } => {
            let mut current_area = player_data.current_area_deck.lock().await;
            match current_area.as_mut() {
                Some(area_deck) => match area_deck.draw_encounter(&encounter_id) {
                    Ok(_) => {
                        let gs = game_state.lock().await;
                        let payload = crate::library::types::ActionPayload::DrawEncounter {
                            area_id: "current".to_string(),
                            encounter_id,
                            reason: Some("Player drew encounter".to_string()),
                        };
                        let entry = gs.append_action("DrawEncounter", payload);
                        Ok((rocket::http::Status::Created, Json(entry)))
                    }
                    Err(e) => Err(Left(NotFound(new_status(e)))),
                },
                None => Err(Left(NotFound(new_status(
                    "No current area set".to_string(),
                )))),
            }
        }
        PlayerActions::ReplaceEncounter {
            area_id: _,
            old_encounter_id,
            new_encounter_id,
        } => {
            let mut current_area = player_data.current_area_deck.lock().await;
            match current_area.as_mut() {
                Some(area_deck) => match area_deck.get_encounter(&new_encounter_id) {
                    Some(new_enc) => {
                        let affixes = new_enc.affixes.clone();
                        match area_deck.replace_encounter(&old_encounter_id, new_enc) {
                            Ok(_) => {
                                let gs = game_state.lock().await;
                                let payload =
                                    crate::library::types::ActionPayload::ReplaceEncounter {
                                        area_id: "current".to_string(),
                                        old_encounter_id,
                                        new_encounter_id,
                                        affixes_applied: affixes,
                                        reason: Some(
                                            "Player replaced resolved encounter".to_string(),
                                        ),
                                    };
                                let entry = gs.append_action("ReplaceEncounter", payload);
                                Ok((rocket::http::Status::Created, Json(entry)))
                            }
                            Err(e) => Err(Left(NotFound(new_status(e)))),
                        }
                    }
                    None => Err(Left(NotFound(new_status(format!(
                        "New encounter {} not found",
                        new_encounter_id
                    ))))),
                },
                None => Err(Left(NotFound(new_status(
                    "No current area set".to_string(),
                )))),
            }
        }
        PlayerActions::ApplyScouting {
            area_id: _,
            parameters,
        } => {
            let current_area = player_data.current_area_deck.lock().await;
            match current_area.as_ref() {
                Some(_) => {
                    let gs = game_state.lock().await;
                    let payload = crate::library::types::ActionPayload::ApplyScouting {
                        area_id: "current".to_string(),
                        parameters,
                        reason: Some("Player applied scouting".to_string()),
                    };
                    let entry = gs.append_action("ApplyScouting", payload);
                    Ok((rocket::http::Status::Created, Json(entry)))
                }
                None => Err(Left(NotFound(new_status(
                    "No current area set".to_string(),
                )))),
            }
        }
        // Step 7: Encounter action handlers
        PlayerActions::EncounterPickEncounter { card_id } => {
            let mut current_area = player_data.current_area_deck.lock().await;
            match current_area.as_mut() {
                Some(area_deck) => match area_deck.get_encounter(&card_id) {
                    Some(enc) => {
                        if enc.state != crate::area_deck::EncounterState::Available {
                            return Err(Right(BadRequest(new_status(format!(
                                "Encounter {} is not available (on hand)",
                                card_id
                            )))));
                        }
                        let gs = game_state.lock().await;
                        let payload = crate::library::types::ActionPayload::DrawEncounter {
                            area_id: "current".to_string(),
                            encounter_id: card_id,
                            reason: Some("Player picked encounter by card_id".to_string()),
                        };
                        let entry = gs.append_action("EncounterPickEncounter", payload);
                        Ok((rocket::http::Status::Created, Json(entry)))
                    }
                    None => Err(Left(NotFound(new_status(format!(
                        "Encounter card {} not found in area deck",
                        card_id
                    ))))),
                },
                None => Err(Left(NotFound(new_status(
                    "No current area set".to_string(),
                )))),
            }
        }
        PlayerActions::EncounterPlayCard {
            card_id,
            effects: _,
        } => {
            // Validate card exists and is on hand
            match get_card(card_id as usize, player_data).await {
                None => Err(Left(NotFound(new_status(format!(
                    "Card {} does not exist",
                    card_id
                ))))),
                Some(card) => {
                    // Derive deck from card type (implicit deck semantics)
                    let deck_id = match card.card_type {
                        CardType::Defence => *player_data.defence_deck_id.lock().await,
                        CardType::Attack => *player_data.attack_deck_id.lock().await,
                        CardType::Resource => *player_data.resource_deck_id.lock().await,
                    };
                    let decks = player_data.decks.lock().await;
                    let on_hand = decks
                        .iter()
                        .find(|d| d.id == deck_id)
                        .and_then(|d| {
                            d.cards
                                .iter()
                                .find(|c| c.id == card_id as usize)
                                .and_then(|c| c.state.get(&CardState::Hand).copied())
                        })
                        .unwrap_or(0);
                    if on_hand == 0 {
                        return Err(Right(BadRequest(new_status(format!(
                            "Card {} is not on hand",
                            card_id
                        )))));
                    }
                    drop(decks);

                    let gs = game_state.lock().await;
                    let payload = crate::library::types::ActionPayload::PlayCard {
                        card_id: card_id as usize,
                        deck_id: None,
                        reason: Some("Player played card during encounter".to_string()),
                    };
                    let entry = gs.append_action("EncounterPlayCard", payload);
                    Ok((rocket::http::Status::Created, Json(entry)))
                }
            }
        }
        PlayerActions::EncounterApplyScouting {
            choice_id: _,
            parameters,
        } => {
            let gs = game_state.lock().await;
            let payload = crate::library::types::ActionPayload::ApplyScouting {
                area_id: "encounter".to_string(),
                parameters: serde_json::to_string(&parameters).unwrap_or_default(),
                reason: Some("Player applied scouting after encounter".to_string()),
            };
            let entry = gs.append_action("EncounterApplyScouting", payload);
            Ok((rocket::http::Status::Created, Json(entry)))
        }
        PlayerActions::EncounterFinish => {
            let gs = game_state.lock().await;
            let payload = crate::library::types::ActionPayload::DrawEncounter {
                area_id: "encounter".to_string(),
                encounter_id: "finished".to_string(),
                reason: Some("Player finished encounter".to_string()),
            };
            let entry = gs.append_action("EncounterFinish", payload);
            Ok((rocket::http::Status::Created, Json(entry)))
        }
    }
}
