use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod endpoints;
pub mod scouting;

pub use scouting::ScoutingParams;

/// Encounter lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub enum EncounterState {
    Available,
    Active,
    Resolved,
}

/// Represents a single encounter card in an area
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct Encounter {
    pub id: String,
    pub name: String,
    pub base_type: String,
    pub state: EncounterState,
    pub affixes: Vec<String>,
    pub entry_cost: Option<i64>,
    pub reward_deck_id: Option<String>,
}

impl Encounter {
    pub fn new(id: String, name: String, base_type: String) -> Self {
        Encounter {
            id,
            name,
            base_type,
            state: EncounterState::Available,
            affixes: Vec::new(),
            entry_cost: None,
            reward_deck_id: None,
        }
    }

    pub fn with_affixes(mut self, affixes: Vec<String>) -> Self {
        self.affixes = affixes;
        self
    }

    pub fn with_entry_cost(mut self, cost: i64) -> Self {
        self.entry_cost = Some(cost);
        self
    }

    pub fn with_reward_deck(mut self, deck_id: String) -> Self {
        self.reward_deck_id = Some(deck_id);
        self
    }
}

/// Affix definition
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct Affix {
    pub name: String,
    pub description: Option<String>,
}

/// Pipeline for generating affixes deterministically
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct AffixPipeline {
    pub available_affixes: Vec<Affix>,
}

impl AffixPipeline {
    pub fn new() -> Self {
        AffixPipeline {
            available_affixes: vec![
                Affix {
                    name: "fireresistant".to_string(),
                    description: Some("Reduces fire damage".to_string()),
                },
                Affix {
                    name: "poisoned".to_string(),
                    description: Some("Applies poison effect".to_string()),
                },
                Affix {
                    name: "cursed".to_string(),
                    description: Some("Applies curse effect".to_string()),
                },
                Affix {
                    name: "blessed".to_string(),
                    description: Some("Grants blessing".to_string()),
                },
            ],
        }
    }

    pub fn select_affixes(&self, seed: u64, count: usize) -> Vec<String> {
        use rand::seq::SliceRandom;
        use rand::SeedableRng;
        use rand_pcg::Lcg64Xsh32;

        let mut rng = Lcg64Xsh32::seed_from_u64(seed);
        let max_count = count.min(self.available_affixes.len());

        self.available_affixes
            .choose_multiple(&mut rng, max_count)
            .map(|a| a.name.clone())
            .collect()
    }
}

impl Default for AffixPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Area Deck containing encounter cards
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct AreaDeck {
    pub id: String,
    pub name: String,
    pub encounters: Vec<Encounter>,
    pub affix_pipeline: AffixPipeline,
    pub next_encounter_id: u64,
}

impl AreaDeck {
    pub fn new(id: String, name: String) -> Self {
        AreaDeck {
            id,
            name,
            encounters: Vec::new(),
            affix_pipeline: AffixPipeline::new(),
            next_encounter_id: 0,
        }
    }

    /// Add an encounter to the area deck
    pub fn add_encounter(&mut self, encounter: Encounter) {
        self.encounters.push(encounter);
    }

    /// Draw (activate) an encounter from the area deck
    pub fn draw_encounter(&mut self, encounter_id: &str) -> Result<Encounter, String> {
        let mut encounter = self
            .encounters
            .iter_mut()
            .find(|e| e.id == encounter_id)
            .ok_or_else(|| format!("Encounter {} not found", encounter_id))?
            .clone();

        encounter.state = EncounterState::Active;
        self.encounters
            .iter_mut()
            .find(|e| e.id == encounter_id)
            .ok_or_else(|| format!("Encounter {} not found", encounter_id))?
            .state = EncounterState::Active;

        Ok(encounter)
    }

    /// Mark an encounter as resolved
    pub fn resolve_encounter(&mut self, encounter_id: &str) -> Result<(), String> {
        self.encounters
            .iter_mut()
            .find(|e| e.id == encounter_id)
            .ok_or_else(|| format!("Encounter {} not found", encounter_id))?
            .state = EncounterState::Resolved;

        Ok(())
    }

    /// Replace a resolved encounter with a new one
    pub fn replace_encounter(
        &mut self,
        encounter_id: &str,
        new_encounter: Encounter,
    ) -> Result<Encounter, String> {
        let position = self
            .encounters
            .iter()
            .position(|e| e.id == encounter_id)
            .ok_or_else(|| format!("Encounter {} not found", encounter_id))?;

        let old = self.encounters.remove(position);
        if old.state != EncounterState::Resolved {
            self.encounters.insert(position, old.clone());
            return Err(format!(
                "Cannot replace encounter {} in state {:?}",
                encounter_id, old.state
            ));
        }

        self.encounters.insert(position, new_encounter.clone());
        Ok(new_encounter)
    }

    /// Generate a new encounter deterministically
    pub fn generate_encounter(&mut self, base_type: String, seed: u64) -> Encounter {
        let id = format!("encounter_{}", self.next_encounter_id);
        self.next_encounter_id += 1;

        let affix_count = ((seed >> 32) % 3) as usize;
        let affixes = self.affix_pipeline.select_affixes(seed, affix_count);

        Encounter::new(id, format!("{} Encounter", base_type), base_type).with_affixes(affixes)
    }

    /// Get all available (non-resolved) encounters
    pub fn get_available_encounters(&self) -> Vec<Encounter> {
        self.encounters
            .iter()
            .filter(|e| e.state != EncounterState::Resolved)
            .cloned()
            .collect()
    }

    /// Get an encounter by id
    pub fn get_encounter(&self, encounter_id: &str) -> Option<Encounter> {
        self.encounters
            .iter()
            .find(|e| e.id == encounter_id)
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_area_deck_creation() {
        let deck = AreaDeck::new("area_1".to_string(), "Forest".to_string());
        assert_eq!(deck.id, "area_1");
        assert_eq!(deck.name, "Forest");
        assert!(deck.encounters.is_empty());
    }

    #[test]
    fn test_add_and_draw_encounter() {
        let mut deck = AreaDeck::new("area_1".to_string(), "Forest".to_string());
        let encounter = Encounter::new(
            "enc_1".to_string(),
            "Goblin".to_string(),
            "Combat".to_string(),
        );
        deck.add_encounter(encounter.clone());

        let drawn = deck.draw_encounter("enc_1").unwrap();
        assert_eq!(drawn.state, EncounterState::Active);
    }

    #[test]
    fn test_resolve_and_replace() {
        let mut deck = AreaDeck::new("area_1".to_string(), "Forest".to_string());
        let encounter = Encounter::new(
            "enc_1".to_string(),
            "Goblin".to_string(),
            "Combat".to_string(),
        );
        deck.add_encounter(encounter);

        deck.resolve_encounter("enc_1").unwrap();
        let new_encounter =
            Encounter::new("enc_2".to_string(), "Orc".to_string(), "Combat".to_string());
        let replacement = deck.replace_encounter("enc_1", new_encounter).unwrap();

        assert_eq!(replacement.id, "enc_2");
        assert_eq!(replacement.state, EncounterState::Available);
    }

    #[test]
    fn test_deterministic_affix_generation() {
        let mut deck = AreaDeck::new("area_1".to_string(), "Forest".to_string());
        let enc1 = deck.generate_encounter("Combat".to_string(), 12345);
        let enc2 = deck.generate_encounter("Combat".to_string(), 12345);

        assert_eq!(enc1.affixes, enc2.affixes);
    }
}
