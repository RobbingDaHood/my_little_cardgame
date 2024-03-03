use std::sync::Arc;

use rocket::futures::lock::Mutex;

use crate::deck::Card;
use crate::deck::Deck;

pub struct PLayerData {
    pub(crate) decks: Arc<Mutex<Vec<Deck>>>,
    pub(crate) cards: Arc<Mutex<Vec<Card>>>,
}

pub fn new(decks: Arc<Mutex<Vec<Deck>>>, cards: Arc<Mutex<Vec<Card>>>) -> PLayerData {
    PLayerData {
        decks,
        cards,
    }
}
