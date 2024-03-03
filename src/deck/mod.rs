use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Json;
use rocket::State;

pub use crate::deck::card::Card;
use crate::deck::card::get_card;
use crate::deck::card_state::CardState;
use crate::player_data::PLayerData;
use crate::status_messages::StatusMessages;
use crate::status_messages::StatusMessages::{CardNotFound, DeckNotFound};

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

#[get("/")]
pub async fn list_all_decks(player_data: &State<PLayerData>) -> Json<Vec<Deck>> {
    let mut result = vec![];
    for deck in player_data.decks.lock().await.iter() {
        result.push(deck.clone())
    }
    Json(result)
}

#[get("/<id>")]
pub async fn get_deck_json(id: usize, player_data: &State<PLayerData>) -> Option<Json<Deck>> {
    player_data.decks.lock().await.iter()
        .find(|existing| existing.id == id)
        .map(|existing| Json(existing.clone()))
}

#[post("/<id>", format = "json", data = "<new_card>")]
pub async fn add_card_to_deck(id: usize, new_card: Json<DeckCard>, player_data: &State<PLayerData>) -> Option<Json<StatusMessages>> {
    match get_card(new_card.id, player_data).await {
        None => Some(Json(CardNotFound(new_card.id))),
        Some(_) => {
            match player_data.decks.lock().await.iter_mut()
                .find(|existing| existing.id == id) {
                None => Some(Json(DeckNotFound(id))),
                Some(existing_deck) => {
                    existing_deck.add_new_card(new_card.0);
                    Some(Json(StatusMessages::CardAddedToDeck()))
                }
            }
        }
    }
}

#[post("/", format = "json")]
pub async fn create_deck(player_data: &State<PLayerData>) -> Json<StatusMessages> {
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
    Json(StatusMessages::CreatedStatusMessage(unused_id))
}
