use std::collections::HashMap;
use std::sync::Arc;

use rocket::futures::lock::Mutex;

use crate::combat::Combat;
use crate::deck::{Card, CardState, DeckCard};
use crate::deck::card::CardType;
use crate::deck::Deck;
use crate::deck::token::{PermanentDefinition, Token, TokenPermanence, TokenType};

pub struct PLayerData {
    pub(crate) decks: Arc<Mutex<Vec<Deck>>>,
    pub(crate) cards: Arc<Mutex<Vec<Card>>>,
    pub(crate) attack_deck_id: Arc<Mutex<usize>>,
    pub(crate) defence_deck_id: Arc<Mutex<usize>>,
    pub(crate) resource_deck_id: Arc<Mutex<usize>>,
    pub(crate) tokens: Arc<Mutex<Vec<Token>>>,
    pub(crate) current_combat: Arc<Mutex<Option<Combat>>>,
}

pub fn new() -> PLayerData {
    PLayerData {
        cards: Arc::new(
            Mutex::new(
                vec![
                    Card {
                        id: 0,
                        card_type: CardType::Attack,
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
                    Card {
                        id: 1,
                        card_type: CardType::Defence,
                        effects: vec![
                            Token {
                                token_type: TokenType::Dodge,
                                permanence: TokenPermanence::AllAtBeginningOfRound,
                                count: 1,
                            }
                        ],
                        costs: vec![
                            Token {
                                token_type: TokenType::Stamina,
                                permanence: TokenPermanence::Instant,
                                count: 2,
                            }
                        ],
                        count: 40,
                    },
                    Card {
                        id: 2,
                        card_type: CardType::Ressource,
                        effects: vec![
                            Token {
                                token_type: TokenType::Stamina,
                                permanence: TokenPermanence::Permanent(PermanentDefinition { max_count: 20 }),
                                count: 2,
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
                        contains_card_types: vec![CardType::Attack],
                        cards: vec![
                            DeckCard {
                                id: 0,
                                state: HashMap::from([
                                    (CardState::Deck, 40)
                                ]),
                            }
                        ],
                        id: 0,
                    },
                    Deck {
                        contains_card_types: vec![CardType::Defence],
                        cards: vec![
                            DeckCard {
                                id: 1,
                                state: HashMap::from([
                                    (CardState::Deck, 40)
                                ]),
                            }
                        ],
                        id: 1,
                    },
                    Deck {
                        contains_card_types: vec![CardType::Ressource],
                        cards: vec![
                            DeckCard {
                                id: 2,
                                state: HashMap::from([
                                    (CardState::Deck, 40)
                                ]),
                            }
                        ],
                        id: 2,
                    },
                ]
            )
        ),
        attack_deck_id: Arc::new(Mutex::new(0)),
        defence_deck_id: Arc::new(Mutex::new(1)),
        resource_deck_id: Arc::new(Mutex::new(2)),
        tokens: Arc::new(
            Mutex::new(vec![
                Token {
                    token_type: TokenType::Health,
                    permanence: TokenPermanence::UsedOnUnit,
                    count: 20,
                },
            ],
            )
        ),
        current_combat: Arc::new(Mutex::new(None)),
    }
}
