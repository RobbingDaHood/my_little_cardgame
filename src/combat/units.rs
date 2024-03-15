use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::JsonSchema;

use crate::deck::Card;
use crate::deck::card::CardType;
use crate::deck::token::{PermanentDefinition, Token, TokenPermanence, TokenType};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct Unit {
    attack_deck: Vec<Card>,
    defence_deck: Vec<Card>,
    resource_deck: Vec<Card>,
    tokens: Vec<Token>,
}


pub fn get_gnome() -> Unit {
    Unit {
        attack_deck: vec![
            Card {
                id: 5,
                card_type: CardType::Attack,
                effects: vec![
                    Token {
                        token_type: TokenType::Poison,
                        permanence: TokenPermanence::Instant,
                        count: 1,
                    }
                ],
                costs: vec![
                    Token {
                        token_type: TokenType::Mana,
                        permanence: TokenPermanence::Instant,
                        count: 1,
                    }
                ],
                count: 5,
            },
            Card {
                id: 5,
                card_type: CardType::Attack,
                effects: vec![
                    Token {
                        token_type: TokenType::Health,
                        permanence: TokenPermanence::Instant,
                        count: 3,
                    }
                ],
                costs: vec![
                    Token {
                        token_type: TokenType::Stamina,
                        permanence: TokenPermanence::Instant,
                        count: 2,
                    }
                ],
                count: 5,
            },
            Card {
                id: 5,
                card_type: CardType::Attack,
                effects: vec![
                    Token {
                        token_type: TokenType::Health,
                        permanence: TokenPermanence::Instant,
                        count: 2,
                    }
                ],
                costs: vec![
                    Token {
                        token_type: TokenType::Stamina,
                        permanence: TokenPermanence::Instant,
                        count: 1,
                    }
                ],
                count: 10,
            },
            Card {
                id: 5,
                card_type: CardType::Attack,
                effects: vec![
                    Token {
                        token_type: TokenType::Health,
                        permanence: TokenPermanence::Instant,
                        count: 1,
                    }
                ],
                costs: vec![],
                count: 20,
            },
        ],
        defence_deck: vec![
            Card {
                id: 4,
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
            }
        ],
        resource_deck: vec![
            Card {
                id: 1,
                card_type: CardType::Ressource,
                effects: vec![
                    Token {
                        token_type: TokenType::Stamina,
                        permanence: TokenPermanence::Permanent(PermanentDefinition { max_count: 20 }),
                        count: 2,
                    }
                ],
                costs: vec![],
                count: 20,
            },
            Card {
                id: 2,
                card_type: CardType::Ressource,
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
                count: 10,
            },
            Card {
                id: 3,
                card_type: CardType::Ressource,
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
                count: 10,
            },
        ],
        tokens: vec![
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
        ],
    }
}