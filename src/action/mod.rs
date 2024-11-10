use rocket::futures::StreamExt;
use rocket::response::status::{Created, NotFound};
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::{JsonSchema, openapi};

use crate::combat::{Combat, CombatStates};
use crate::combat::units::get_gnome;
use crate::deck::{CardState, Deck};
use crate::player_data::PLayerData;
use crate::status_messages::{new_status, Status};

/// Player actions
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema, Hash, Copy)]
#[serde(crate = "rocket::serde")]
pub enum PlayerActions {
    /// Id of the card in the relevant deck
    PlayCard(usize)
}

#[openapi]
#[post("/action", format = "json", data = "<player_action>")]
pub async fn play_action(player_data: &State<PLayerData>, player_action: Json<PlayerActions>) -> Result<Created<&str>, NotFound<Json<Status>>> {
    let action = player_action.0;

    match action {
        PlayerActions::PlayCard(card_id) => {
            let mut combat_optional: Option<Combat> = *player_data.current_combat.lock().await.clone();
            match combat_optional {
                Some(combat) => {
                    let deck_id = match combat.state {
                        CombatStates::PlayerDefending => { player_data.defence_deck_id.lock().await.clone() }
                        CombatStates::PlayerAttacking => { player_data.attack_deck_id.lock().await.clone() }
                        CombatStates::PlayerRessourcing => { player_data.resource_deck_id.lock().await.clone() }
                    };
                    return player_data.decks.lock().await
                        .iter_mut()
                        .find(|deck| deck.id == deck_id)
                        .map_or(
                            Err(NotFound(new_status(format!("Card with id {:?} does not exist in deck!", action)))),
                            |deck| deck.change_card_state(card_id, CardState::Hand, CardState::Deck)
                                .map(|_| Created::new("ALL OKAY")),
                        );
                }
                None => {}
            }
        }
    };
    Err(NotFound(new_status(format!("Card with id {:?} does not exist in deck!", action))))
}
