use std::sync::Arc;

use rocket::futures::lock::Mutex;

use crate::deck::Card;
use crate::deck::Deck;

pub struct PLayerData {
    pub(crate) decks: Vec<Arc<Mutex<Deck>>>,
    pub(crate) cards: Arc<Mutex<Vec<Card>>>,
}

pub fn new(decks: Vec<Arc<Mutex<Deck>>>, cards: Arc<Mutex<Vec<Card>>>) -> PLayerData {
    PLayerData {
        decks,
        cards,
    }
}
