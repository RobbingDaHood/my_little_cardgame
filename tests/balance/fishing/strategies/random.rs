use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde_json::Value;
use std::sync::Mutex;

use crate::strategies::{GameSnapshot, Strategy};

/// Plays a random fishing card each turn.
pub struct RandomFishingStrategy {
    rng: Mutex<rand_pcg::Lcg64Xsh32>,
}

impl RandomFishingStrategy {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Mutex::new(rand_pcg::Lcg64Xsh32::seed_from_u64(seed)),
        }
    }
}

impl Strategy for RandomFishingStrategy {
    fn name(&self) -> &str {
        "FishingRandom"
    }

    fn choose_action(&self, actions: &[Value], _snapshot: &GameSnapshot) -> Value {
        if actions.is_empty() {
            return serde_json::json!({"action_type": "EncounterAbort"});
        }
        let mut rng = self.rng.lock().expect("rng lock");
        actions
            .choose(&mut *rng)
            .cloned()
            .unwrap_or_else(|| serde_json::json!({"action_type": "EncounterAbort"}))
    }
}
