use either::{Either, Left, Right};
use rocket::response::status::{BadRequest, NotFound};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use rocket_okapi::{openapi, JsonSchema};

pub mod persistence;

use crate::player_data::RandomGeneratorWrapper;
use crate::status_messages::{new_status, Status};

use rand::SeedableRng;
use rand_pcg::Lcg64Xsh32;

/// Player actions
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema, Hash)]
#[serde(crate = "rocket::serde", tag = "action_type")]
pub enum PlayerActions {
    NewGame { seed: Option<u64> },
    // Encounter actions (Step 7)
    EncounterPickEncounter { card_id: usize },
    EncounterPlayCard { card_id: u64 },
    EncounterApplyScouting { card_ids: Vec<usize> },
    EncounterAbort,
}

#[openapi]
#[post("/action", format = "json", data = "<player_action>")]
pub async fn play(
    player_data: &State<RandomGeneratorWrapper>,
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
        PlayerActions::NewGame { seed } => {
            use rand::RngCore;
            let s = seed.unwrap_or_else(|| {
                let mut rng = Lcg64Xsh32::from_entropy();
                rng.next_u64()
            });
            let mut seed_bytes: [u8; 16] = [0u8; 16];
            seed_bytes[0..8].copy_from_slice(&s.to_le_bytes());
            seed_bytes[8..16].copy_from_slice(&s.to_le_bytes());
            *player_data.seed.lock().await = seed_bytes;
            let new_rng = Lcg64Xsh32::from_seed(seed_bytes);
            *player_data.random_generator_state.lock().await = new_rng;

            // Reset game state (library is re-initialized with encounter cards in hand)
            let mut gs = game_state.lock().await;
            let new_gs = crate::library::GameState::new();
            gs.library = new_gs.library;
            gs.token_balances = new_gs.token_balances;
            gs.current_encounter = None;
            gs.encounter_phase = new_gs.encounter_phase;
            gs.last_encounter_result = None;
            gs.encounter_results.clear();

            let payload = crate::library::types::ActionPayload::SetSeed { seed: s };
            let entry = gs.append_action("NewGame", payload);
            Ok((rocket::http::Status::Created, Json(entry)))
        }
        // Step 7: Encounter action handlers
        PlayerActions::EncounterPickEncounter { card_id } => {
            let mut gs = game_state.lock().await;
            if !gs.library.encounter_contains(card_id) {
                return Err(Left(NotFound(new_status(format!(
                    "Encounter card {} not found in area hand",
                    card_id
                )))));
            }
            // Move encounter from hand → discard
            if let Err(e) = gs.library.play(card_id) {
                return Err(Right(BadRequest(new_status(e))));
            }
            // Get the encounter kind to dispatch
            let lib_card = gs.library.get(card_id).ok_or_else(|| {
                Left(NotFound(new_status(format!(
                    "Card {} not found in Library",
                    card_id
                ))))
            })?;
            let encounter_kind = match &lib_card.kind {
                crate::library::types::CardKind::Encounter { encounter_kind } => {
                    encounter_kind.clone()
                }
                _ => {
                    return Err(Right(BadRequest(new_status(format!(
                        "Card {} is not an encounter",
                        card_id
                    )))));
                }
            };
            let mut rng = player_data.random_generator_state.lock().await;
            match encounter_kind {
                crate::library::types::EncounterKind::Combat { .. } => {
                    // Initialize player health if not set
                    if gs
                        .token_balances
                        .get(&crate::library::types::Token::persistent(
                            crate::library::types::TokenType::Health,
                        ))
                        .copied()
                        .unwrap_or(0)
                        == 0
                    {
                        gs.token_balances.insert(
                            crate::library::types::Token::persistent(
                                crate::library::types::TokenType::Health,
                            ),
                            20,
                        );
                    }
                    match gs.start_combat(card_id, &mut rng) {
                        Ok(()) => {}
                        Err(e) => return Err(Right(BadRequest(new_status(e)))),
                    }
                }
                crate::library::types::EncounterKind::Mining { .. } => {
                    match gs.start_mining_encounter(card_id, &mut rng) {
                        Ok(()) => {}
                        Err(e) => return Err(Right(BadRequest(new_status(e)))),
                    }
                }
                crate::library::types::EncounterKind::Herbalism { .. } => {
                    match gs.start_herbalism_encounter(card_id, &mut rng) {
                        Ok(()) => {}
                        Err(e) => return Err(Right(BadRequest(new_status(e)))),
                    }
                }
                crate::library::types::EncounterKind::Woodcutting { .. } => {
                    match gs.start_woodcutting_encounter(card_id, &mut rng) {
                        Ok(()) => {}
                        Err(e) => return Err(Right(BadRequest(new_status(e)))),
                    }
                }
                crate::library::types::EncounterKind::Fishing { .. } => {
                    match gs.start_fishing_encounter(card_id, &mut rng) {
                        Ok(()) => {}
                        Err(e) => return Err(Right(BadRequest(new_status(e)))),
                    }
                }
            }
            let payload = crate::library::types::ActionPayload::DrawEncounter {
                encounter_id: card_id.to_string(),
            };
            let entry = gs.append_action("EncounterPickEncounter", payload);
            Ok((rocket::http::Status::Created, Json(entry)))
        }
        PlayerActions::EncounterPlayCard { card_id } => {
            let mut gs = game_state.lock().await;
            if gs.current_encounter.is_none() {
                return Err(Right(BadRequest(new_status(
                    "No active encounter".to_string(),
                ))));
            }
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

            // Dispatch based on encounter type
            match &gs.current_encounter {
                Some(crate::library::types::EncounterState::Combat(_)) => {
                    // Validate card kind matches current combat phase
                    {
                        let combat = match &gs.current_encounter {
                            Some(crate::library::types::EncounterState::Combat(c)) => c,
                            _ => unreachable!(),
                        };
                        let is_allowed = combat.phase.allowed_card_kind();
                        if !is_allowed(&lib_card.kind) {
                            return Err(Right(BadRequest(new_status(format!(
                                "Card {} is not playable in current phase (expected {})",
                                card_id,
                                combat.phase.allowed_card_kind_name()
                            )))));
                        }
                    }
                    match gs.library.play(card_id as usize) {
                        Ok(()) => {
                            let mut rng = player_data.random_generator_state.lock().await;
                            let _ = gs.resolve_player_card(card_id as usize, &mut rng);
                            // Auto-advance: enemy plays and phase advances
                            if gs.current_encounter.is_some() {
                                let _ = gs.resolve_enemy_play(&mut rng);
                                if gs.current_encounter.is_some() {
                                    let _ = gs.advance_combat_phase();
                                }
                            }
                        }
                        Err(e) => return Err(Right(BadRequest(new_status(e)))),
                    }
                }
                Some(crate::library::types::EncounterState::Mining(_)) => {
                    // Validate card is a Mining card
                    if !matches!(
                        lib_card.kind,
                        crate::library::types::CardKind::Mining { .. }
                    ) {
                        return Err(Right(BadRequest(new_status(format!(
                            "Card {} is not a Mining card (required for mining encounter)",
                            card_id
                        )))));
                    }
                    match gs.library.play(card_id as usize) {
                        Ok(()) => {
                            let mut rng = player_data.random_generator_state.lock().await;
                            let _ = gs.resolve_player_mining_card(card_id as usize, &mut rng);
                        }
                        Err(e) => return Err(Right(BadRequest(new_status(e)))),
                    }
                }
                Some(crate::library::types::EncounterState::Herbalism(_)) => {
                    // Validate card is an Herbalism card
                    if !matches!(
                        lib_card.kind,
                        crate::library::types::CardKind::Herbalism { .. }
                    ) {
                        return Err(Right(BadRequest(new_status(format!(
                            "Card {} is not an Herbalism card (required for herbalism encounter)",
                            card_id
                        )))));
                    }
                    match gs.library.play(card_id as usize) {
                        Ok(()) => {
                            let mut rng = player_data.random_generator_state.lock().await;
                            let _ = gs.resolve_player_herbalism_card(card_id as usize, &mut rng);
                        }
                        Err(e) => return Err(Right(BadRequest(new_status(e)))),
                    }
                }
                Some(crate::library::types::EncounterState::Woodcutting(_)) => {
                    if !matches!(
                        lib_card.kind,
                        crate::library::types::CardKind::Woodcutting { .. }
                    ) {
                        return Err(Right(BadRequest(new_status(format!(
                            "Card {} is not a Woodcutting card (required for woodcutting encounter)",
                            card_id
                        )))));
                    }
                    match gs.library.play(card_id as usize) {
                        Ok(()) => {
                            let mut rng = player_data.random_generator_state.lock().await;
                            let _ = gs.resolve_player_woodcutting_card(card_id as usize, &mut rng);
                        }
                        Err(e) => return Err(Right(BadRequest(new_status(e)))),
                    }
                }
                Some(crate::library::types::EncounterState::Fishing(_)) => {
                    if !matches!(
                        lib_card.kind,
                        crate::library::types::CardKind::Fishing { .. }
                    ) {
                        return Err(Right(BadRequest(new_status(format!(
                            "Card {} is not a Fishing card (required for fishing encounter)",
                            card_id
                        )))));
                    }
                    match gs.library.play(card_id as usize) {
                        Ok(()) => {
                            let mut rng = player_data.random_generator_state.lock().await;
                            let _ = gs.resolve_player_fishing_card(card_id as usize, &mut rng);
                        }
                        Err(e) => return Err(Right(BadRequest(new_status(e)))),
                    }
                }
                None => {
                    return Err(Right(BadRequest(new_status(
                        "No active encounter".to_string(),
                    ))));
                }
            }

            let payload = crate::library::types::ActionPayload::PlayCard {
                card_id: card_id as usize,
            };
            let entry = gs.append_action("EncounterPlayCard", payload);
            Ok((rocket::http::Status::Created, Json(entry)))
        }
        PlayerActions::EncounterApplyScouting { card_ids } => {
            let mut gs = game_state.lock().await;
            if gs.encounter_phase != crate::library::types::EncounterPhase::Scouting {
                return Err(Right(BadRequest(new_status(
                    "Not in Scouting phase".to_string(),
                ))));
            }
            for cid in &card_ids {
                if !gs.library.encounter_contains(*cid) {
                    return Err(Left(NotFound(new_status(format!(
                        "Card {} not found in area hand",
                        cid
                    )))));
                }
            }

            // Recycle encounter back to deck and refill hand
            if let Some(ref enc) = gs.current_encounter {
                let enc_id = enc.encounter_card_id();
                let _ = gs.library.return_to_deck(enc_id);
            }
            let foresight = gs
                .token_balances
                .get(&crate::library::types::Token::persistent(
                    crate::library::types::TokenType::Foresight,
                ))
                .copied()
                .unwrap_or(3) as usize;
            gs.library.encounter_draw_to_hand(foresight);

            gs.encounter_phase = crate::library::types::EncounterPhase::NoEncounter;
            let payload = crate::library::types::ActionPayload::ApplyScouting {
                card_ids: card_ids.clone(),
            };
            let entry = gs.append_action("EncounterApplyScouting", payload);
            Ok((rocket::http::Status::Created, Json(entry)))
        }
        PlayerActions::EncounterAbort => {
            let mut gs = game_state.lock().await;
            match &gs.current_encounter {
                Some(crate::library::types::EncounterState::Combat(_)) => {
                    return Err(Right(BadRequest(new_status(
                        "Cannot abort a combat encounter".to_string(),
                    ))));
                }
                Some(_) => {
                    // Mark non-combat encounter as lost, go to scouting
                    gs.abort_encounter();
                }
                None => {
                    return Err(Right(BadRequest(new_status(
                        "No active encounter to abort".to_string(),
                    ))));
                }
            }
            let payload = crate::library::types::ActionPayload::AbortEncounter;
            let entry = gs.append_action("EncounterAbort", payload);
            Ok((rocket::http::Status::Created, Json(entry)))
        }
    }
}
