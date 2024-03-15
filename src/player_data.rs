use std::collections::HashMap;
use std::sync::Arc;

use rocket::futures::lock::Mutex;

use crate::deck::{Card, CardState, DeckCard};
use crate::deck::Deck;
use crate::deck::token::{Token, TokenPermanence, TokenType};

pub struct PLayerData {
    pub(crate) decks: Arc<Mutex<Vec<Deck>>>,
    pub(crate) cards: Arc<Mutex<Vec<Card>>>,
}

pub fn new() -> PLayerData {
    PLayerData {
        cards: Arc::new(
            Mutex::new(
                vec![
                    Card {
                        id: 5,
                        effects: vec![
                            Token {
                                token_type: TokenType::Health,
                                permanence: TokenPermanence::Instant,
                                count: 1,
                            }
                        ],
                        costs: vec![],
                        count: 40,
                    },
                ]
            )
        ),
        decks: Arc::new(
            Mutex::new(
                vec![
                    Deck {
                        cards: vec![
                            DeckCard {
                                id: 5,
                                state: HashMap::from([
                                    (CardState::Deck, 40)
                                ]),
                            }
                        ],
                        id: 0,
                    }
                ]
            )
        ),
    }
}
