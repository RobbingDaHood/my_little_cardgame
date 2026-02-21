use std::sync::Arc;

use rand::{RngCore, SeedableRng};
use rand_pcg::Lcg64Xsh32;
use rocket::futures::lock::Mutex;

use crate::area_deck::AreaDeck;
use crate::combat::Combat;
use crate::deck::card::CardType;
use crate::deck::token::{PermanentDefinition, Token, TokenPermanence, TokenType};
use crate::deck::Card;

pub struct PlayerData {
    pub(crate) cards: Arc<Mutex<Vec<Card>>>,
    #[allow(dead_code)]
    pub(crate) tokens: Arc<Mutex<Vec<Token>>>,
    pub(crate) current_combat: Arc<Mutex<Box<Option<Combat>>>>,
    pub(crate) last_combat_result: Arc<Mutex<Option<crate::combat::CombatResult>>>,
    #[allow(dead_code)]
    pub(crate) seed: Arc<Mutex<[u8; 16]>>,
    #[allow(dead_code)]
    pub(crate) random_generator_state: Arc<Mutex<Lcg64Xsh32>>,
    /// Represents the player's current location/area
    pub(crate) current_area_deck: Arc<Mutex<Option<AreaDeck>>>,
}

pub fn new() -> PlayerData {
    let mut new_seed: [u8; 16] = [1; 16];
    Lcg64Xsh32::from_entropy().fill_bytes(&mut new_seed);
    let random_generator = Lcg64Xsh32::from_seed(new_seed);

    PlayerData {
        seed: Arc::new(Mutex::new(new_seed)),
        random_generator_state: Arc::new(Mutex::new(random_generator)),
        cards: initialize_player_cards(),
        tokens: Arc::new(Mutex::new(vec![Token {
            token_type: TokenType::Health,
            permanence: TokenPermanence::UsedOnUnit,
            count: 20,
        }])),
        current_combat: Arc::new(Mutex::new(Box::new(None))),
        last_combat_result: Arc::new(Mutex::new(None)),
        current_area_deck: Arc::new(Mutex::new(Some(initialize_area_deck()))),
    }
}

fn initialize_area_deck() -> AreaDeck {
    let mut deck = AreaDeck::new("starter_area".to_string());
    // Reference the gnome CombatEncounter card at Library index 3
    deck.add_encounter(3);
    deck.add_encounter(3);
    deck.add_encounter(3);
    deck
}

fn initialize_player_cards() -> Arc<Mutex<Vec<Card>>> {
    Arc::new(Mutex::new(vec![
        Card {
            id: 0,
            card_type: CardType::Attack,
            effects: vec![Token {
                token_type: TokenType::Health,
                permanence: TokenPermanence::Instant,
                count: 20,
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
            card_type: CardType::Resource,
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
