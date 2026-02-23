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
    NewGame { seed: Option<u64> },
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
            gs.current_combat = None;
            gs.encounter_state = new_gs.encounter_state;
            gs.last_combat_result = None;

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
            // Initialize player health if not set
            if gs
                .token_balances
                .get(&crate::library::types::TokenType::Health)
                .copied()
                .unwrap_or(0)
                == 0
            {
                gs.token_balances
                    .insert(crate::library::types::TokenType::Health, 20);
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
                    crate::library::types::CardKind::Encounter { .. } => "Encounter",
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
            for cid in &card_ids {
                if !gs.library.encounter_contains(*cid) {
                    return Err(Left(NotFound(new_status(format!(
                        "Card {} not found in area hand",
                        cid
                    )))));
                }
            }

            let parameters = match serde_json::to_string(&card_ids) {
                Ok(s) => s,
                Err(e) => {
                    return Err(Left(NotFound(new_status(format!(
                        "Failed to serialize card_ids: {e}"
                    )))));
                }
            };

            // Recycle encounter back to deck and refill hand
            if let Some(ref combat) = gs.current_combat {
                if let Some(enc_id) = combat.encounter_card_id {
                    let _ = gs.library.return_to_deck(enc_id);
                }
            }
            let foresight = gs
                .token_balances
                .get(&crate::library::types::TokenType::Foresight)
                .copied()
                .unwrap_or(3) as usize;
            gs.library.encounter_draw_to_hand(foresight);

            gs.encounter_state.phase = crate::library::types::EncounterPhase::Ready;
            let payload = crate::library::types::ActionPayload::ApplyScouting {
                area_id: "current".to_string(),
                parameters,
                reason: Some("Player applied scouting with card_ids".to_string()),
            };
            let entry = gs.append_action("EncounterApplyScouting", payload);
            Ok((rocket::http::Status::Created, Json(entry)))
        }
    }
}
