use either::{Either, Left, Right};
use rocket::response::status::{BadRequest, NotFound};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use rocket_okapi::{openapi, JsonSchema};

pub mod persistence;

use crate::player_data::PlayerData;
use crate::status_messages::{new_status, Status};

use rand::SeedableRng;
use rand_pcg::Lcg64Xsh32;

/// Player actions
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema, Hash)]
#[serde(crate = "rocket::serde", tag = "action_type")]
pub enum PlayerActions {
    SetSeed { seed: u64 },
    // Encounter actions (Step 7)
    EncounterPickEncounter { card_id: usize },
    EncounterPlayCard { card_id: u64, effects: Vec<String> },
    EncounterApplyScouting { card_ids: Vec<usize> },
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
        // Step 7: Encounter action handlers
        PlayerActions::EncounterPickEncounter { card_id } => {
            let mut current_area = player_data.current_area_deck.lock().await;
            match current_area.as_mut() {
                Some(area_deck) => {
                    if !area_deck.contains(card_id) {
                        return Err(Left(NotFound(new_status(format!(
                            "Encounter card {} not found in area deck hand",
                            card_id
                        )))));
                    }
                    area_deck.pick_encounter(card_id);
                    let mut gs = game_state.lock().await;
                    // Initialize player health if not set
                    if gs
                        .token_balances
                        .get(&crate::library::types::TokenId::Health)
                        .copied()
                        .unwrap_or(0)
                        == 0
                    {
                        gs.token_balances
                            .insert(crate::library::types::TokenId::Health, 20);
                    }
                    match gs.start_combat(card_id) {
                        Ok(()) => {
                            let payload = crate::library::types::ActionPayload::DrawEncounter {
                                area_id: "current".to_string(),
                                encounter_id: card_id.to_string(),
                                reason: Some("Player picked encounter by card_id".to_string()),
                            };
                            let entry = gs.append_action("EncounterPickEncounter", payload);
                            Ok((rocket::http::Status::Created, Json(entry)))
                        }
                        Err(e) => Err(Right(BadRequest(new_status(e)))),
                    }
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
            if gs.current_combat.is_none() {
                return Err(Right(BadRequest(new_status(
                    "No active combat".to_string(),
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
            // Validate card kind matches current combat phase
            {
                let combat = gs.current_combat.as_ref().expect("checked above");
                let allowed_kind = combat.phase.allowed_card_kind();
                let card_kind_name = match &lib_card.kind {
                    crate::library::types::CardKind::Attack { .. } => "Attack",
                    crate::library::types::CardKind::Defence { .. } => "Defence",
                    crate::library::types::CardKind::Resource { .. } => "Resource",
                    crate::library::types::CardKind::CombatEncounter { .. } => "CombatEncounter",
                };
                if card_kind_name != allowed_kind {
                    return Err(Right(BadRequest(new_status(format!(
                        "Card {} is not playable in current phase (expected {})",
                        card_id, allowed_kind
                    )))));
                }
            }
            match gs.library.play(card_id as usize) {
                Ok(()) => {
                    let _ = gs.resolve_player_card(card_id as usize);
                    let payload = crate::library::types::ActionPayload::PlayCard {
                        card_id: card_id as usize,
                        deck_id: None,
                        reason: Some("Player played card during encounter".to_string()),
                    };
                    let entry = gs.append_action("EncounterPlayCard", payload);

                    // Auto-advance: enemy plays and phase advances
                    if gs.current_combat.is_some() {
                        let mut rng = player_data.random_generator_state.lock().await;
                        let _ = gs.resolve_enemy_play(&mut rng);
                        if gs.current_combat.is_some() {
                            let _ = gs.advance_combat_phase();
                        }
                    }

                    Ok((rocket::http::Status::Created, Json(entry)))
                }
                Err(e) => Err(Right(BadRequest(new_status(e)))),
            }
        }
        PlayerActions::EncounterApplyScouting { card_ids } => {
            let mut gs = game_state.lock().await;
            if gs.encounter_state.phase != crate::library::types::EncounterPhase::Scouting {
                return Err(Right(BadRequest(new_status(
                    "Not in Scouting phase".to_string(),
                ))));
            }
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

                    // Recycle encounter back to area deck and refill hand
                    {
                        let mut current_area = player_data.current_area_deck.lock().await;
                        if let Some(area_deck) = current_area.as_mut() {
                            if let Some(ref combat) = gs.current_combat {
                                if let Some(enc_id) = combat.encounter_card_id {
                                    area_deck.recycle_encounter(enc_id);
                                }
                            }
                            let foresight = gs
                                .token_balances
                                .get(&crate::library::types::TokenId::Foresight)
                                .copied()
                                .unwrap_or(3) as usize;
                            area_deck.draw_to_hand(foresight);
                        }
                    }

                    gs.encounter_state.phase = crate::library::types::EncounterPhase::Ready;
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
