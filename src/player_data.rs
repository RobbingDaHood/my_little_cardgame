use std::collections::HashMap;
use std::sync::Arc;

use rand::{RngCore, SeedableRng};
use rand_pcg::{Lcg64Xsh32, Pcg32};
use rocket::futures::lock::Mutex;

use crate::combat::Combat;
use crate::deck::card::CardType;
use crate::deck::token::{PermanentDefinition, Token, TokenPermanence, TokenType};
use crate::deck::Deck;
use crate::deck::{Card, CardState, DeckCard};

pub struct PlayerData {
    pub(crate) decks: Arc<Mutex<Vec<Deck>>>,
    pub(crate) cards: Arc<Mutex<Vec<Card>>>,
    pub(crate) attack_deck_id: Arc<Mutex<usize>>,
    pub(crate) defence_deck_id: Arc<Mutex<usize>>,
    pub(crate) resource_deck_id: Arc<Mutex<usize>>,
    #[allow(dead_code)]
    pub(crate) tokens: Arc<Mutex<Vec<Token>>>,
    pub(crate) current_combat: Arc<Mutex<Box<Option<Combat>>>>,
    pub(crate) last_combat_result: Arc<Mutex<Option<crate::combat::CombatResult>>>,
    #[allow(dead_code)]
    pub(crate) seed: Arc<Mutex<[u8; 16]>>,
    #[allow(dead_code)]
    pub(crate) random_generator_state: Arc<Mutex<Lcg64Xsh32>>,
}

pub fn new() -> PlayerData {
    let mut attack_deck = intialize_player_attack_deck();
    let mut defence_deck = initialize_player_defence_deck();
    let mut ressource_deck = initialize_player_resource_deck();
    let mut new_seed: [u8; 16] = [1; 16];

    // Draw some cards
    Pcg32::from_entropy().fill_bytes(&mut new_seed);
    let mut random_generator = Pcg32::from_seed(new_seed);
    let _ = attack_deck.draw_cards(5, &mut random_generator);
    let _ = defence_deck.draw_cards(5, &mut random_generator);
    let _ = ressource_deck.draw_cards(5, &mut random_generator);

    PlayerData {
        seed: Arc::new(Mutex::new(new_seed)),
        random_generator_state: Arc::new(Mutex::new(random_generator)),
        cards: initialize_player_cards(),
        decks: Arc::new(Mutex::new(vec![attack_deck, defence_deck, ressource_deck])),
        attack_deck_id: Arc::new(Mutex::new(0)),
        defence_deck_id: Arc::new(Mutex::new(1)),
        resource_deck_id: Arc::new(Mutex::new(2)),
        tokens: Arc::new(Mutex::new(vec![Token {
            token_type: TokenType::Health,
            permanence: TokenPermanence::UsedOnUnit,
            count: 20,
        }])),
        current_combat: Arc::new(Mutex::new(Box::new(None))),
        last_combat_result: Arc::new(Mutex::new(None)),
    }
}

fn initialize_player_cards() -> Arc<Mutex<Vec<Card>>> {
    Arc::new(Mutex::new(vec![
        Card {
            id: 0,
            card_type: CardType::Attack,
            effects: vec![Token {
                token_type: TokenType::Health,
                permanence: TokenPermanence::Instant,
                count: 1,
            }],
            costs: vec![],
            count: 40,
        },
        Card {
            id: 1,
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
            count: 40,
        },
        Card {
            id: 2,
            card_type: CardType::Ressource,
            effects: vec![Token {
                token_type: TokenType::Stamina,
                permanence: TokenPermanence::Permanent(PermanentDefinition { max_count: 20 }),
                count: 2,
            }],
            costs: vec![],
            count: 40,
        },
    ]))
}

fn initialize_player_resource_deck() -> Deck {
    Deck {
        contains_card_types: vec![CardType::Ressource],
        cards: vec![DeckCard {
            id: 2,
            state: HashMap::from([(CardState::Deck, 40)]),
        }],
        id: 2,
    }
}

fn initialize_player_defence_deck() -> Deck {
    Deck {
        contains_card_types: vec![CardType::Defence],
        cards: vec![DeckCard {
            id: 1,
            state: HashMap::from([(CardState::Deck, 40)]),
        }],
        id: 1,
    }
}

fn intialize_player_attack_deck() -> Deck {
    Deck {
        contains_card_types: vec![CardType::Attack],
        cards: vec![DeckCard {
            id: 0,
            state: HashMap::from([(CardState::Deck, 40)]),
        }],
        id: 0,
    }
}
