use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod endpoints;
pub mod scouting;

pub use scouting::ScoutingParams;

/// Area Deck: a list of encounter card indices into the Library.
///
/// Each entry is a `usize` referencing a `LibraryCard` with `CardKind::CombatEncounter`.
/// Encounter lifecycle (available/active/resolved) is tracked by the Library card's
/// `CardCounts` (library=available, hand=active, discard=resolved).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct AreaDeck {
    pub id: String,
    pub encounter_card_ids: Vec<usize>,
}

impl AreaDeck {
    pub fn new(id: String) -> Self {
        AreaDeck {
            id,
            encounter_card_ids: Vec::new(),
        }
    }

    pub fn add_encounter(&mut self, library_card_id: usize) {
        self.encounter_card_ids.push(library_card_id);
    }

    pub fn contains(&self, library_card_id: usize) -> bool {
        self.encounter_card_ids.contains(&library_card_id)
    }
}
