use either::{Either, Left, Right};
use rocket::response::status::{BadRequest, Created, NotFound};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use rocket_okapi::{openapi, JsonSchema};

use crate::combat::{Combat, States};
use crate::deck::CardState;
use crate::deck::card::{get_card, CardType};
use crate::combat;
use crate::player_data::PlayerData;
use crate::status_messages::{new_status, Status};

/// Player actions
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema, Hash, Copy)]
#[serde(crate = "rocket::serde")]
pub enum PlayerActions {
    /// Id of the card in the relevant deck
    PlayCard(usize),
}

#[openapi]
#[post("/action", format = "json", data = "<player_action>")]
pub async fn play(
    player_data: &State<PlayerData>,
    player_action: Json<PlayerActions>,
) -> Result<Created<&str>, Either<NotFound<Json<Status>>, BadRequest<Json<Status>>>> {
    let action = player_action.0;

    match action {
        PlayerActions::PlayCard(card_id) => {
            let combat_optional: Option<Combat> = *player_data.current_combat.lock().await.clone();
            match combat_optional {
                Some(combat) => {
                    // check card exists and is of allowed type for the phase
                    match get_card(card_id, player_data).await {
                        None => Err(Left(NotFound(new_status(format!("Card {:?} does not exist!", card_id))))),
                        Some(card) => {
                            let allowed = match combat.state {
                                States::Defending => CardType::Defence,
                                States::Attacking => CardType::Attack,
                                States::Resourcing => CardType::Resource,
                            };
                            if card.card_type != allowed {
                                return Err(Right(BadRequest(new_status(format!("Card with id {} is not playable in current phase", card_id)))));
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
                                    let res = deck.change_card_state(card_id, CardState::Discard, CardState::Hand).map_err(Left).map(|()| Created::new("ALL OKAY"));
                                    if res.is_ok() {
                                        crate::combat::resolve::resolve_card_effects(card_id, true, player_data).await;
                                    }
                                    res
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
