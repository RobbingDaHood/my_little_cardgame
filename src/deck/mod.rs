use std::collections::HashMap;

use either::{Either, Left, Right};
use rocket::response::status::{BadRequest, Created, NotFound};
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::{JsonSchema, openapi};

use crate::deck::card::{CardType, get_card};
pub use crate::deck::card::Card;
use crate::player_data::PLayerData;
use crate::status_messages::{new_status, Status};

pub mod card;
pub mod token;

/// CardState represents the cards state in a deck.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema, Hash)]
#[serde(crate = "rocket::serde")]
pub enum CardState {
    /// The card is in the deck.
    Deck,
    /// The card is in your hand.
    Hand,
    /// The card is in the discard pile.
    Discard,
    /// The card is marked as deleted. This is both used for a possible undo option, documentation and performance.
    Deleted,
}


#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct Deck {
    pub cards: Vec<DeckCard>,
    pub id: usize,
    pub contains_card_types: Vec<CardType>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct DeckCard {
    pub(crate) id: usize,
    pub(crate) state: HashMap<CardState, u32>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct CreateDeck {
    pub contains_card_types: Vec<CardType>,
}


impl Deck {
    pub fn add_new_card(&mut self, new_card: DeckCard) {
        self.cards.push(new_card);
    }

    pub fn change_card_state(&mut self, card_id: usize, new_state: CardState, old_state: CardState) -> Result<(), ()> {
        match self.cards.iter_mut()
            .find(|card| card.id == card_id) {
            None => Err(()),
            Some(card) => {
                match card.state.get(&old_state) {
                    None => Err(()),
                    Some(old_state_count) => {
                        card.state.insert(old_state, old_state_count - 1);
                        match card.state.get(&new_state) {
                            None => {
                                card.state.insert(new_state, 1);
                                Ok(())
                            }
                            Some(new_state_count) => {
                                card.state.insert(new_state, new_state_count + 1);
                                Ok(())
                            }
                        }
                    }
                }
            }
        }
    }
}

#[openapi]
#[get("/decks")]
pub async fn list_all_decks(player_data: &State<PLayerData>) -> Json<Vec<Deck>> {
    let mut result = vec![];
    for deck in player_data.decks.lock().await.iter() {
        result.push(deck.clone())
    }
    Json(result)
}

#[openapi]
#[get("/decks/<id>")]
pub async fn get_deck(id: usize, player_data: &State<PLayerData>) -> Result<Json<Deck>, NotFound<Json<Status>>> {
    player_data.decks.lock().await.iter()
        .find(|existing| existing.id == id)
        .map(|existing| Json(existing.clone()))
        .ok_or(NotFound(new_status(format!("Deck with id {} does not exist!", id))))
}

#[openapi]
#[get("/decks/<deck_id>/cards/<card_id>")]
pub async fn get_card_in_deck(deck_id: usize, card_id: usize, player_data: &State<PLayerData>) -> Result<Json<DeckCard>, NotFound<Json<Status>>> {
    player_data.decks.lock().await.iter()
        .filter(|existing| existing.id == deck_id)
        .flat_map(|existing| existing.cards.iter())
        .find(|existing| existing.id == card_id)
        .map(|existing| Json(existing.clone()))
        .ok_or(NotFound(new_status(format!("Either Deck with id {} or Card with id {} does not exist!", deck_id, card_id))))
}

/// Add a card to the deck. A card can exist in multiple decks, but they cannot be multiple times in the same deck
#[openapi]
#[post("/decks/<id>/cards", format = "json", data = "<new_card>")]
pub async fn add_card_to_deck(id: usize, new_card: Json<DeckCard>, player_data: &State<PLayerData>) -> Result<Created<&str>, Either<NotFound<Json<Status>>, BadRequest<Json<Status>>>> {
    match get_card(new_card.id, player_data).await {
        None => Err(Left(NotFound(new_status(format!("Card with id {} does not exist!", new_card.id))))),
        Some(existing_card) => {
            match player_data.decks.lock().await.iter_mut()
                .find(|existing| existing.id == id) {
                None => Err(Left(NotFound(new_status(format!("Deck with id {} does not exist!", id))))),
                Some(existing_deck) => {
                    if !existing_deck.contains_card_types.contains(&existing_card.card_type) {
                        Err(Right(BadRequest(new_status(format!("Card with id {} is of type {:?} and that is not part of the types '{:?}' allowed in deck with id {}", new_card.id, existing_card.card_type, existing_deck.contains_card_types, existing_deck.id)))))
                    } else {
                        match existing_deck.cards.iter()
                            .find(|existing_card| existing_card.id == new_card.id) {
                            Some(_) => Err(Right(BadRequest(new_status(format!("Deck with id {} does already contain a card with id {}!", id, new_card.id))))),
                            None => {
                                existing_deck.add_new_card(new_card.0.clone());
                                let location = uri!(get_card_in_deck(id, new_card.id));
                                Ok(Created::new(location.to_string()))
                            }
                        }
                    }
                }
            }
        }
    }
}

#[openapi]
#[delete("/decks/<deck_id>/cards/<card_id>")]
pub async fn delete_card_in_deck(deck_id: usize, card_id: usize, player_data: &State<PLayerData>) -> Result<(), NotFound<Json<Status>>> {
    match player_data.decks.lock().await.iter_mut()
        .find(|existing| existing.id == deck_id) {
        None => Err(NotFound(new_status(format!("Deck with id {} does not exist!", deck_id)))),
        Some(deck) => deck
            .change_card_state(card_id, CardState::Deleted, CardState::Deck)
            .map_err(
                |_| NotFound(new_status(format!("Card with id {} does not exist in deck!", card_id)))
            )
    }
}

#[openapi]
#[post("/decks", format = "json", data = "<new_deck>")]
pub async fn create_deck(new_deck: Json<CreateDeck>, player_data: &State<PLayerData>) -> Created<&str> {
    let unused_id = *player_data.decks.lock().await.iter()
        .map(|existing| existing.id)
        .max()
        .map(|existing_id| existing_id + 1)
        .get_or_insert(0);
    player_data.decks.lock().await.push(
        Deck {
            cards: vec![],
            id: unused_id,
            contains_card_types: new_deck.0.contains_card_types,
        }
    );
    let location = uri!(get_deck(unused_id));
    Created::new(location.to_string())
}
