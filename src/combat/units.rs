use std::collections::HashMap;

use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::JsonSchema;

use crate::deck::card::CardType;
use crate::deck::token::{PermanentDefinition, Token, TokenPermanence, TokenType};
use crate::deck::CardState;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct Unit {
    pub attack_deck: Vec<UnitCard>,
    pub defence_deck: Vec<UnitCard>,
    pub resource_deck: Vec<UnitCard>,
    pub tokens: Vec<Token>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct UnitCard {
    /// Unique id of the card
    pub effects: Vec<Token>,
    pub costs: Vec<Token>,
    pub card_type: CardType,
    pub(crate) state: HashMap<CardState, u32>,
}

pub fn get_gnome() -> Unit {
    Unit {
        attack_deck: get_gnome_attack_deck(),
        defence_deck: get_gnome_defence_deck(),
        resource_deck: get_gnome_resource_deck(),
        tokens: get_gnome_tokens(),
    }
}

fn get_gnome_resource_deck() -> Vec<UnitCard> {
    vec![
        UnitCard {
            state: HashMap::from([(CardState::Deck, 20)]),
            card_type: CardType::Resource,
            effects: vec![Token {
                token_type: TokenType::Stamina,
                permanence: TokenPermanence::Permanent(PermanentDefinition { max_count: 20 }),
                count: 2,
            }],
            costs: vec![],
        },
        UnitCard {
            state: HashMap::from([(CardState::Deck, 10)]),
            card_type: CardType::Resource,
            effects: vec![
                Token {
                    token_type: TokenType::Health,
                    permanence: TokenPermanence::Permanent(PermanentDefinition { max_count: 10 }),
                    count: 1,
                },
                Token {
                    token_type: TokenType::Stamina,
                    permanence: TokenPermanence::Permanent(PermanentDefinition { max_count: 10 }),
                    count: 1,
                },
            ],
            costs: vec![],
        },
        UnitCard {
            state: HashMap::from([(CardState::Deck, 10)]),
            card_type: CardType::Resource,
            effects: vec![
                Token {
                    token_type: TokenType::Mana,
                    permanence: TokenPermanence::Permanent(PermanentDefinition { max_count: 10 }),
                    count: 1,
                },
                Token {
                    token_type: TokenType::Stamina,
                    permanence: TokenPermanence::Permanent(PermanentDefinition { max_count: 10 }),
                    count: 1,
                },
            ],
            costs: vec![],
        },
    ]
}

fn get_gnome_attack_deck() -> Vec<UnitCard> {
    vec![
        UnitCard {
            state: HashMap::from([(CardState::Deck, 5)]),
            card_type: CardType::Attack,
            effects: vec![Token {
                token_type: TokenType::Poison,
                permanence: TokenPermanence::Instant,
                count: 1,
            }],
            costs: vec![Token {
                token_type: TokenType::Mana,
                permanence: TokenPermanence::Instant,
                count: 1,
            }],
        },
        UnitCard {
            state: HashMap::from([(CardState::Deck, 5)]),
            card_type: CardType::Attack,
            effects: vec![Token {
                token_type: TokenType::Health,
                permanence: TokenPermanence::Instant,
                count: 3,
            }],
            costs: vec![Token {
                token_type: TokenType::Stamina,
                permanence: TokenPermanence::Instant,
                count: 2,
            }],
        },
        UnitCard {
            state: HashMap::from([(CardState::Deck, 10)]),
            card_type: CardType::Attack,
            effects: vec![Token {
                token_type: TokenType::Health,
                permanence: TokenPermanence::Instant,
                count: 2,
            }],
            costs: vec![Token {
                token_type: TokenType::Stamina,
                permanence: TokenPermanence::Instant,
                count: 1,
            }],
        },
        UnitCard {
            state: HashMap::from([(CardState::Deck, 20)]),
            card_type: CardType::Attack,
            effects: vec![Token {
                token_type: TokenType::Health,
                permanence: TokenPermanence::Instant,
                count: 1,
            }],
            costs: vec![],
        },
    ]
}

fn get_gnome_defence_deck() -> Vec<UnitCard> {
    vec![UnitCard {
        state: HashMap::from([(CardState::Deck, 40)]),
        card_type: CardType::Defence,
        effects: vec![Token {
            token_type: TokenType::Dodge,
            permanence: TokenPermanence::AllAtBeginningOfRound,
            count: 1,
        }],
        costs: vec![Token {
            token_type: TokenType::Stamina,
            permanence: TokenPermanence::Instant,
            count: 2,
        }],
    }]
}

fn get_gnome_tokens() -> Vec<Token> {
    vec![
        Token {
            token_type: TokenType::Health,
            permanence: TokenPermanence::UsedOnUnit,
            count: 20,
        },
        Token {
            token_type: TokenType::Stamina,
            permanence: TokenPermanence::UsedOnUnit,
            count: 20,
        },
        Token {
            token_type: TokenType::Mana,
            permanence: TokenPermanence::UsedOnUnit,
            count: 1,
        },
    ]
}
