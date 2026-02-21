use rocket::response::status::{Created, NotFound};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use rocket_okapi::{openapi, JsonSchema};

use crate::combat::units::{get_gnome, Unit};
use crate::deck::card::CardType;
use crate::player_data::PlayerData;
use crate::status_messages::{new_status, Status};
use rand::RngCore;

pub mod resolve;
pub mod units;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct Combat {
    pub allies: Vec<Unit>,
    pub enemies: Vec<Unit>,
    pub state: States,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct CombatResult {
    pub winner: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema, Hash, Copy)]
#[serde(crate = "rocket::serde")]
pub enum States {
    Defending,
    Attacking,
    Resourcing,
}

#[openapi]
#[get("/combat")]
pub async fn get_combat(
    player_data: &State<PlayerData>,
) -> Result<Json<Combat>, NotFound<Json<Status>>> {
    let combat = player_data.current_combat.lock().await.clone();
    match *combat {
        Some(c) => Ok(Json(c)),
        None => Err(NotFound(new_status(
            "Combat has not been initialized".to_string(),
        ))),
    }
}

/// Initialize combat for testing purposes.
///
/// **TESTING ENDPOINT ONLY** - This endpoint is provided for testing purposes only
/// and may be replaced with an action-based initialization mechanism in the future.
#[openapi]
#[post("/tests/combat")]
pub async fn initialize_combat(player_data: &State<PlayerData>) -> Created<&str> {
    player_data.current_combat.lock().await.replace(Combat {
        allies: vec![],
        enemies: vec![get_gnome()],
        state: States::Defending,
    });
    // Ensure enemy unit decks have some cards in Hand so enemy_play can select from them in tests
    {
        let mut combat_lock = player_data.current_combat.lock().await;
        if let Some(combat_box) = combat_lock.as_mut() {
            for enemy in combat_box.enemies.iter_mut() {
                use crate::deck::CardState;
                for uc in enemy
                    .defence_deck
                    .iter_mut()
                    .chain(enemy.attack_deck.iter_mut())
                    .chain(enemy.resource_deck.iter_mut())
                {
                    uc.state.insert(CardState::Hand, 1);
                }
            }
        }
    }
    Created::new("/tests/combat")
}

#[openapi]
#[post("/combat/enemy_play")]
pub async fn enemy_play(player_data: &State<PlayerData>) -> Created<&'static str> {
    // Determine current phase
    let combat_opt = player_data.current_combat.lock().await.clone();
    if combat_opt.is_none() {
        return Created::new("/combat/enemy_play");
    }
    let combat = match *combat_opt {
        Some(c) => c,
        None => return Created::new("/combat/enemy_play"),
    };
    let phase = combat.state;

    // Map phase to card type
    let desired_card_type = match phase {
        States::Defending => CardType::Defence,
        States::Attacking => CardType::Attack,
        States::Resourcing => CardType::Resource,
    };

    // If the phase is Defending, attempt to pick a UnitCard from enemy defence_deck, otherwise pick from the matching deck
    // Deterministic enemy play: if Defending, add dodge to first enemy; otherwise resolve a global card of matching type
    // Multi-enemy unit selection: collect candidate pairs across enemies and pick one at random
    let mut effects_to_resolve: Option<Vec<crate::deck::token::Token>> = None;
    {
        let mut combat_lock = player_data.current_combat.lock().await;
        if let Some(combat_box) = combat_lock.as_mut() {
            // Build candidate list of (enemy_idx, unit_card_idx)
            use crate::deck::CardState;
            let mut candidates: Vec<(usize, usize)> = Vec::new();
            for (ei, enemy) in combat_box.enemies.iter().enumerate() {
                let deck_vec = match desired_card_type {
                    CardType::Defence => &enemy.defence_deck,
                    CardType::Attack => &enemy.attack_deck,
                    CardType::Resource => &enemy.resource_deck,
                };
                for (uci, uc) in deck_vec.iter().enumerate() {
                    if let Some(count) = uc.state.get(&CardState::Hand) {
                        if *count > 0 {
                            candidates.push((ei, uci));
                        }
                    }
                }
            }

            if !candidates.is_empty() {
                // pick one deterministically using PlayerData RNG
                let mut rng = player_data.random_generator_state.lock().await;
                let pick = (rng.next_u64() as usize) % candidates.len();
                let (enemy_idx, uc_idx) = candidates[pick];
                // mutate the selected unit card counts
                let enemy = &mut combat_box.enemies[enemy_idx];
                let deck_vec = match desired_card_type {
                    CardType::Defence => &mut enemy.defence_deck,
                    CardType::Attack => &mut enemy.attack_deck,
                    CardType::Resource => &mut enemy.resource_deck,
                };
                // decrement Hand and increment Discard
                if let Some(hand_count) = deck_vec[uc_idx].state.get_mut(&CardState::Hand) {
                    *hand_count = hand_count.saturating_sub(1);
                }
                let discard_entry = deck_vec[uc_idx]
                    .state
                    .entry(CardState::Discard)
                    .or_insert(0);
                *discard_entry += 1;
                // clone effects and resolve after lock release
                effects_to_resolve = Some(deck_vec[uc_idx].effects.clone());
            } else {
                // no unit candidates: fallback behavior
                if desired_card_type == CardType::Defence {
                    use crate::deck::token::{Token, TokenPermanence, TokenType};
                    if !combat_box.enemies.is_empty() {
                        let enemy = &mut combat_box.enemies[0];
                        let existing = enemy
                            .tokens
                            .iter_mut()
                            .find(|t| t.token_type == TokenType::Dodge);
                        if let Some(t) = existing {
                            t.count += 1;
                        } else {
                            enemy.tokens.push(Token {
                                token_type: TokenType::Dodge,
                                permanence: TokenPermanence::UsedOnUnit,
                                count: 1,
                            });
                        }
                    }
                } else {
                    // mark to fallback to global outside lock
                }
            }
        }
    }

    if let Some(effects) = effects_to_resolve {
        crate::combat::resolve::apply_effects(&effects, false, player_data).await;
    } else if desired_card_type != CardType::Defence {
        if let Some(card) = player_data
            .cards
            .lock()
            .await
            .iter()
            .find(|c| c.card_type == desired_card_type)
            .cloned()
        {
            crate::combat::resolve::resolve_card_effects(card.id, false, player_data).await;
        }
    }

    Created::new("/combat/enemy_play")
}

#[openapi]
#[post("/combat/advance")]
pub async fn advance_phase(player_data: &State<PlayerData>) -> Created<&'static str> {
    let mut lock = player_data.current_combat.lock().await;
    if let Some(combat_box) = lock.as_mut() {
        combat_box.state = match combat_box.state {
            States::Defending => States::Attacking,
            States::Attacking => States::Resourcing,
            States::Resourcing => States::Defending,
        };
    }
    Created::new("/combat/advance")
}

#[openapi]
#[get("/combat/result")]
pub async fn get_combat_result(
    player_data: &State<PlayerData>,
) -> Result<Json<CombatResult>, NotFound<Json<Status>>> {
    let result = player_data.last_combat_result.lock().await.clone();
    match result {
        Some(r) => Ok(Json(r)),
        None => Err(NotFound(new_status(
            "No combat result available".to_string(),
        ))),
    }
}
