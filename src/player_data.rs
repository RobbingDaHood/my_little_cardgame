use std::sync::Arc;

use rocket::futures::lock::Mutex;

use crate::deck::Deck;

pub struct PLayerData {
    pub(crate) decks: Vec<Arc<Mutex<Deck>>>,
}

pub fn new(decks: Vec<Arc<Mutex<Deck>>>) -> PLayerData {
    PLayerData {
        decks
    }
}
