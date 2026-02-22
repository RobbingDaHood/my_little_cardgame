use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod endpoints;
pub mod scouting;

pub use scouting::ScoutingParams;

/// Area Deck: encounter card indices into the Library with deck/hand/discard tracking.
///
/// Each entry is a `usize` referencing a `LibraryCard` with `CardKind::CombatEncounter`.
/// The hand represents encounters currently visible/pickable by the player.
/// Hand size is controlled by the Foresight token.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct AreaDeck {
    pub id: String,
    pub deck: Vec<usize>,
    pub hand: Vec<usize>,
    pub discard: Vec<usize>,
}

impl AreaDeck {
    pub fn new(id: String) -> Self {
        AreaDeck {
            id,
            deck: Vec::new(),
            hand: Vec::new(),
            discard: Vec::new(),
        }
    }

    /// Add an encounter to the deck (not yet visible).
    pub fn add_encounter(&mut self, library_card_id: usize) {
        self.deck.push(library_card_id);
    }

    /// Draw encounters from deck to hand until hand reaches `target_count`.
    pub fn draw_to_hand(&mut self, target_count: usize) {
        while self.hand.len() < target_count {
            if let Some(card_id) = self.deck.pop() {
                self.hand.push(card_id);
            } else {
                break;
            }
        }
    }

    /// Pick an encounter from hand, moving it to discard. Returns true if found.
    pub fn pick_encounter(&mut self, library_card_id: usize) -> bool {
        if let Some(pos) = self.hand.iter().position(|&id| id == library_card_id) {
            let card_id = self.hand.remove(pos);
            self.discard.push(card_id);
            true
        } else {
            false
        }
    }

    /// Recycle a finished encounter: move from discard back to deck.
    pub fn recycle_encounter(&mut self, library_card_id: usize) -> bool {
        if let Some(pos) = self.discard.iter().position(|&id| id == library_card_id) {
            let card_id = self.discard.remove(pos);
            self.deck.push(card_id);
            true
        } else {
            false
        }
    }

    /// Check if an encounter is in the hand (available to pick).
    pub fn contains(&self, library_card_id: usize) -> bool {
        self.hand.contains(&library_card_id)
    }

    /// All encounter card IDs across deck, hand, and discard.
    pub fn encounter_card_ids(&self) -> Vec<usize> {
        let mut all = self.deck.clone();
        all.extend(&self.hand);
        all.extend(&self.discard);
        all
    }
}
