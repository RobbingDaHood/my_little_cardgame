use std::collections::HashMap;

use rand::Rng;
use rand_pcg::Lcg64Xsh32;
use rocket::response::status::NotFound;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::JsonSchema;

use crate::status_messages::{new_status, Status};

/// `CardState` represents the cards state in a deck.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema, Hash, Copy)]
#[serde(crate = "rocket::serde")]
pub enum CardState {
    /// The card is in the deck.
    Deck,
    /// The card is in your hand.
    Hand,
    /// The card is in the discard pile.
    Discard,
    /// The card is marked as deleted. This is both used for a possible undo option, documentation and performance.
    Deleted,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct Deck {
    pub cards: Vec<DeckCard>,
    pub id: usize,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct DeckCard {
    pub(crate) id: usize,
    pub(crate) state: HashMap<CardState, u32>,
}

impl Deck {
    pub fn add_new_card(&mut self, new_card: DeckCard) {
        self.cards.push(new_card);
    }

    pub fn draw_cards(
        &mut self,
        number_of_cards: usize,
        random_generator_state: &mut Lcg64Xsh32,
    ) -> Result<(), NotFound<Json<Status>>> {
        self.change_random_cards_state(
            number_of_cards,
            CardState::Hand,
            CardState::Deck,
            random_generator_state,
        )
    }

    pub fn change_random_cards_state(
        &mut self,
        number_of_cards: usize,
        new_state: CardState,
        old_state: CardState,
        random_generator_state: &mut Lcg64Xsh32,
    ) -> Result<(), NotFound<Json<Status>>> {
        if self.cards.is_empty() {
            return Err(NotFound(new_status(format!(
                "Deck with id {} has no cards!",
                self.id
            ))));
        }
        for _ in 0..number_of_cards {
            let random_card_index = random_generator_state.gen_range(0..self.cards.len());
            let random_card_id = self
                .cards
                .get(random_card_index)
                .ok_or_else(|| {
                    NotFound(new_status(format!(
                        "Card at index {} not found in deck",
                        random_card_index
                    )))
                })?
                .id;
            self.change_card_state(random_card_id, new_state, old_state)?;
        }
        Ok(())
    }

    pub fn change_card_state(
        &mut self,
        card_id: usize,
        new_state: CardState,
        old_state: CardState,
    ) -> Result<(), NotFound<Json<Status>>> {
        match self.cards.iter_mut().find(|card| card.id == card_id) {
            None => Err(NotFound(new_status(format!(
                "Card {:?} does not exist on deck {:?}!",
                card_id, self.id
            )))),
            Some(card) => match card.state.get(&old_state) {
                None => Err(NotFound(new_status(format!(
                    "State {:?} does not exist for card {:?} on deck {:?}!",
                    old_state, card_id, self.id
                )))),
                Some(old_state_count) => {
                    // Guard against underflow: ensure the source state's count is > 0 before subtracting
                    if *old_state_count == 0 {
                        return Err(NotFound(new_status(format!(
                            "State {:?} for card {:?} on deck {:?} has zero count!",
                            old_state, card_id, self.id
                        ))));
                    }
                    card.state.insert(old_state, old_state_count - 1);
                    match card.state.get(&new_state) {
                        None => {
                            card.state.insert(new_state, 1);
                            Ok(())
                        }
                        Some(new_state_count) => {
                            card.state.insert(new_state, new_state_count + 1);
                            Ok(())
                        }
                    }
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_pcg::Lcg64Xsh32;
    use std::collections::HashMap;

    #[test]
    fn change_card_state_nonexistent_card_returns_error() {
        let mut deck = Deck {
            cards: vec![],
            id: 42,
        };
        assert!(deck
            .change_card_state(99, CardState::Hand, CardState::Deck)
            .is_err());
    }

    #[test]
    fn change_card_state_missing_state_returns_error() {
        let mut deck = Deck {
            cards: vec![DeckCard {
                id: 1,
                state: HashMap::from([(CardState::Deck, 1)]),
            }],
            id: 1,
        };
        assert!(deck
            .change_card_state(1, CardState::Hand, CardState::Discard)
            .is_err());
    }

    #[test]
    fn change_random_cards_state_calls_change_card_state_error_when_old_state_missing() {
        let mut deck = Deck {
            cards: vec![DeckCard {
                id: 7,
                state: HashMap::from([(CardState::Deck, 1)]),
            }],
            id: 3,
        };
        let mut rng = Lcg64Xsh32::from_seed([0u8; 16]);
        // old_state Discard does not exist, so change_card_state should return Err and propagate
        assert!(deck
            .change_random_cards_state(1, CardState::Hand, CardState::Discard, &mut rng)
            .is_err());
    }

    #[test]
    fn draw_cards_moves_to_hand() {
        let mut deck = Deck {
            cards: vec![
                DeckCard {
                    id: 1,
                    state: HashMap::from([(CardState::Deck, 2)]),
                },
                DeckCard {
                    id: 2,
                    state: HashMap::from([(CardState::Deck, 2)]),
                },
            ],
            id: 4,
        };
        let mut rng = Lcg64Xsh32::from_seed([1u8; 16]);
        let _ = deck.draw_cards(1, &mut rng);
        let has_hand = deck
            .cards
            .iter()
            .any(|c| c.state.contains_key(&CardState::Hand));
        assert!(has_hand);
    }

    #[test]
    fn change_random_cards_state_on_empty_deck_returns_error() {
        let mut deck = Deck {
            cards: vec![],
            id: 99,
        };
        let mut rng = Lcg64Xsh32::from_seed([0u8; 16]);
        assert!(deck
            .change_random_cards_state(1, CardState::Hand, CardState::Deck, &mut rng)
            .is_err());
    }

    #[test]
    fn change_card_state_zero_count_returns_error() {
        let mut deck = Deck {
            cards: vec![DeckCard {
                id: 5,
                state: HashMap::from([(CardState::Hand, 0u32), (CardState::Deck, 1u32)]),
            }],
            id: 7,
        };
        assert!(deck
            .change_card_state(5, CardState::Discard, CardState::Hand)
            .is_err());
    }

    #[test]
    fn create_deck_unused_id_empty_and_nonempty_behaviour() {
        // empty decks -> unused id should be 0
        let decks_empty: Vec<Deck> = vec![];
        let unused_empty = decks_empty
            .iter()
            .map(|d| d.id)
            .max()
            .map(|id| id + 1)
            .unwrap_or(0);
        assert_eq!(unused_empty, 0);

        // existing decks -> unused id is max + 1
        let decks = [
            Deck {
                cards: vec![],
                id: 0,
            },
            Deck {
                cards: vec![],
                id: 2,
            },
        ];
        let unused = decks
            .iter()
            .map(|d| d.id)
            .max()
            .map(|id| id + 1)
            .unwrap_or(0);
        assert_eq!(unused, 3);
    }
}
