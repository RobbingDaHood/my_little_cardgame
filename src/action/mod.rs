use either::{Either, Left, Right};
use rocket::response::status::{BadRequest, NotFound};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use rocket_okapi::{openapi, JsonSchema};

pub mod persistence;

use crate::combat::{Combat, States};
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
        encounter_id: usize,
    },
    ReplaceEncounter {
        area_id: String,
        old_encounter_id: usize,
        new_encounter_id: usize,
    },
    ApplyScouting {
        area_id: String,
        parameters: String,
    },
    // Encounter actions (Step 7)
    EncounterPickEncounter {
        card_id: usize,
    },
    EncounterPlayCard {
        card_id: u64,
        effects: Vec<String>,
    },
    EncounterApplyScouting {
        card_ids: Vec<usize>,
    },
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
                    let mut gs = game_state.lock().await;
                    // Look up card in Library
                    let lib_card = match gs.library.get(card_id) {
                        Some(c) => c.clone(),
                        None => {
                            return Err(Left(NotFound(new_status(format!(
                                "Card {:?} does not exist in Library!",
                                card_id
                            )))));
                        }
                    };
                    // Validate card kind matches combat phase
                    let allowed_kind = match combat.state {
                        States::Defending => "Defence",
                        States::Attacking => "Attack",
                        States::Resourcing => "Resource",
                    };
                    let card_kind_name = match &lib_card.kind {
                        crate::library::types::CardKind::Attack { .. } => "Attack",
                        crate::library::types::CardKind::Defence { .. } => "Defence",
                        crate::library::types::CardKind::Resource { .. } => "Resource",
                        crate::library::types::CardKind::CombatEncounter { .. } => {
                            "CombatEncounter"
                        }
                    };
                    if card_kind_name != allowed_kind {
                        return Err(Right(BadRequest(new_status(format!(
                            "Card with id {} is not playable in current phase",
                            card_id
                        )))));
                    }
                    // Move card from hand to discard via Library
                    match gs.library.play(card_id) {
                        Ok(()) => {
                            drop(gs);
                            crate::combat::resolve::resolve_card_effects(
                                card_id,
                                true,
                                player_data,
                            )
                            .await;
                            let gs = game_state.lock().await;
                            let payload = crate::library::types::ActionPayload::PlayCard {
                                card_id,
                                deck_id: None,
                                reason: None,
                            };
                            let entry = gs.append_action("PlayCard", payload);
                            Ok((rocket::http::Status::Created, Json(entry)))
                        }
                        Err(e) => Err(Right(BadRequest(new_status(e)))),
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
            let current_area = player_data.current_area_deck.lock().await;
            match current_area.as_ref() {
                Some(area_deck) => {
                    if !area_deck.contains(encounter_id) {
                        return Err(Left(NotFound(new_status(format!(
                            "Encounter card {} not in area deck",
                            encounter_id
                        )))));
                    }
                    let gs = game_state.lock().await;
                    let payload = crate::library::types::ActionPayload::DrawEncounter {
                        area_id: "current".to_string(),
                        encounter_id: encounter_id.to_string(),
                        reason: Some("Player drew encounter".to_string()),
                    };
                    let entry = gs.append_action("DrawEncounter", payload);
                    Ok((rocket::http::Status::Created, Json(entry)))
                }
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
            let current_area = player_data.current_area_deck.lock().await;
            match current_area.as_ref() {
                Some(area_deck) => {
                    if !area_deck.contains(old_encounter_id) {
                        return Err(Left(NotFound(new_status(format!(
                            "Old encounter card {} not in area deck",
                            old_encounter_id
                        )))));
                    }
                    let gs = game_state.lock().await;
                    let payload = crate::library::types::ActionPayload::ReplaceEncounter {
                        area_id: "current".to_string(),
                        old_encounter_id: old_encounter_id.to_string(),
                        new_encounter_id: new_encounter_id.to_string(),
                        affixes_applied: vec![],
                        reason: Some("Player replaced resolved encounter".to_string()),
                    };
                    let entry = gs.append_action("ReplaceEncounter", payload);
                    Ok((rocket::http::Status::Created, Json(entry)))
                }
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
            let current_area = player_data.current_area_deck.lock().await;
            match current_area.as_ref() {
                Some(area_deck) => {
                    if !area_deck.contains(card_id) {
                        return Err(Left(NotFound(new_status(format!(
                            "Encounter card {} not found in area deck",
                            card_id
                        )))));
                    }
                    let gs = game_state.lock().await;
                    let payload = crate::library::types::ActionPayload::DrawEncounter {
                        area_id: "current".to_string(),
                        encounter_id: card_id.to_string(),
                        reason: Some("Player picked encounter by card_id".to_string()),
                    };
                    let entry = gs.append_action("EncounterPickEncounter", payload);
                    Ok((rocket::http::Status::Created, Json(entry)))
                }
                None => Err(Left(NotFound(new_status(
                    "No current area set".to_string(),
                )))),
            }
        }
        PlayerActions::EncounterPlayCard {
            card_id,
            effects: _,
        } => {
            let mut gs = game_state.lock().await;
            let lib_card = match gs.library.get(card_id as usize) {
                Some(c) => c.clone(),
                None => {
                    return Err(Left(NotFound(new_status(format!(
                        "Card {} does not exist in Library",
                        card_id
                    )))));
                }
            };
            if lib_card.counts.hand == 0 {
                return Err(Right(BadRequest(new_status(format!(
                    "Card {} is not on hand",
                    card_id
                )))));
            }
            match gs.library.play(card_id as usize) {
                Ok(()) => {
                    let payload = crate::library::types::ActionPayload::PlayCard {
                        card_id: card_id as usize,
                        deck_id: None,
                        reason: Some("Player played card during encounter".to_string()),
                    };
                    let entry = gs.append_action("EncounterPlayCard", payload);
                    Ok((rocket::http::Status::Created, Json(entry)))
                }
                Err(e) => Err(Right(BadRequest(new_status(e)))),
            }
        }
        PlayerActions::EncounterApplyScouting { card_ids } => {
            let current_area = player_data.current_area_deck.lock().await;
            match current_area.as_ref() {
                Some(area_deck) => {
                    for cid in &card_ids {
                        if !area_deck.contains(*cid) {
                            return Err(Left(NotFound(new_status(format!(
                                "Card {} not found in area deck",
                                cid
                            )))));
                        }
                    }
                    drop(current_area);

                    let parameters = match serde_json::to_string(&card_ids) {
                        Ok(s) => s,
                        Err(e) => {
                            return Err(Left(NotFound(new_status(format!(
                                "Failed to serialize card_ids: {e}"
                            )))));
                        }
                    };
                    let gs = game_state.lock().await;
                    let payload = crate::library::types::ActionPayload::ApplyScouting {
                        area_id: "current".to_string(),
                        parameters,
                        reason: Some("Player applied scouting with card_ids".to_string()),
                    };
                    let entry = gs.append_action("EncounterApplyScouting", payload);
                    Ok((rocket::http::Status::Created, Json(entry)))
                }
                None => Err(Left(NotFound(new_status(
                    "No current area set".to_string(),
                )))),
            }
        }
    }
}
