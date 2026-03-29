use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde_json::Value;
use std::sync::Mutex;

use crate::strategies::{GameSnapshot, Strategy};

/// Random woodcutting strategy — plays any available action uniformly at random.
pub struct RandomStrategy {
    rng: Mutex<rand_pcg::Lcg64Xsh32>,
}

impl RandomStrategy {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Mutex::new(rand_pcg::Lcg64Xsh32::seed_from_u64(seed)),
        }
    }
}

impl Strategy for RandomStrategy {
    fn name(&self) -> &str {
        "random"
    }

    fn choose_action(&self, possible_actions: &[Value], _game_state: &GameSnapshot) -> Value {
        if possible_actions.is_empty() {
            return serde_json::json!({"action_type": "EncounterConcludeEncounter"});
        }
        let mut rng = self.rng.lock().expect("rng lock");
        possible_actions
            .choose(&mut *rng)
            .cloned()
            .unwrap_or_else(|| serde_json::json!({"action_type": "EncounterConcludeEncounter"}))
    }
}
