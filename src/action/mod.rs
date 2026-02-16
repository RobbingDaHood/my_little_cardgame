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
#[serde(crate = "rocket::serde")]
pub enum PlayerActions {
    PlayCard(usize),
    GrantToken { token_id: String, amount: i64 },
    SetSeed { seed: u64 },
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
            match gs.apply_grant(&token_id, amount) {
                Ok(entry) => Ok((rocket::http::Status::Created, Json(entry))),
                Err(e) => Err(Right(BadRequest(new_status(e)))),
            }
        }
        PlayerActions::SetSeed { seed } => {
            let gs = game_state.lock().await;
            // append to action log
            let payload = crate::library::types::ActionPayload::SetSeed { seed };
            let entry = gs.append_action("SetSeed", payload);
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
        PlayerActions::PlayCard(card_id) => {
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
                                    "Card with id {action:?} does not exist in deck!"
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
    }
}
