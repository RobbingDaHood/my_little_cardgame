use either::{Either, Left, Right};
use rocket::response::status::{BadRequest, Created, NotFound};
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Json;
use rocket::State;

pub use crate::deck::card::Card;
use crate::deck::card::get_card;
use crate::deck::card_state::CardState;
use crate::player_data::PLayerData;
use crate::status_messages::{new_status, Status};

pub mod card;
pub mod card_state;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Deck {
    cards: Vec<DeckCard>,
    id: usize,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct DeckCard {
    id: usize,
    state: CardState,
}

impl Deck {
    pub fn add_new_card(&mut self, new_card: DeckCard) {
        self.cards.push(new_card);
    }
}

#[get("/decks")]
pub async fn list_all_decks(player_data: &State<PLayerData>) -> Json<Vec<Deck>> {
    let mut result = vec![];
    for deck in player_data.decks.lock().await.iter() {
        result.push(deck.clone())
    }
    Json(result)
}

#[get("/decks/<id>")]
pub async fn get_deck(id: usize, player_data: &State<PLayerData>) -> Result<Json<Deck>, NotFound<Json<Status>>> {
    player_data.decks.lock().await.iter()
        .find(|existing| existing.id == id)
        .map(|existing| Json(existing.clone()))
        .ok_or(NotFound(new_status(format!("Deck with id {} does not exist!", id))))
}

#[get("/decks/<deck_id>/cards/<card_id>")]
pub async fn get_card_in_deck(deck_id: usize, card_id: usize, player_data: &State<PLayerData>) -> Result<Json<DeckCard>, NotFound<Json<Status>>> {
    player_data.decks.lock().await.iter()
        .filter(|existing| existing.id == deck_id)
        .flat_map(|existing| existing.cards.iter())
        .find(|existing| existing.id == card_id)
        .map(|existing| Json(existing.clone()))
        .ok_or(NotFound(new_status(format!("Either Deck with id {} or Card with id {} does not exist!", deck_id, card_id))))
}

#[post("/decks/<id>/cards", format = "json", data = "<new_card>")]
pub async fn add_card_to_deck(id: usize, new_card: Json<DeckCard>, player_data: &State<PLayerData>) -> Result<Created<&str>, Either<NotFound<Json<Status>>, BadRequest<Json<Status>>>> {
    match get_card(new_card.id, player_data).await {
        None => Err(Left(NotFound(new_status(format!("Card with id {} does not exist!", new_card.id))))),
        Some(_) => {
            match player_data.decks.lock().await.iter_mut()
                .find(|existing| existing.id == id) {
                None => Err(Left(NotFound(new_status(format!("Deck with id {} does not exist!", id))))),
                Some(existing_deck) => {
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

#[post("/decks", format = "json")]
pub async fn create_deck(player_data: &State<PLayerData>) -> Created<&str> {
    let unused_id = *player_data.decks.lock().await.iter()
        .map(|existing| existing.id)
        .max()
        .map(|existing_id| existing_id + 1)
        .get_or_insert(0);
    player_data.decks.lock().await.push(
        Deck {
            cards: vec![],
            id: unused_id,
        }
    );
    let location = uri!(get_deck(unused_id));
    Created::new(location.to_string())
}
