use serde_json::Value;

use crate::strategies::{GameSnapshot, Strategy};

/// Always plays the card with the highest FishingValue.
pub struct GreedyFishingStrategy;

impl Strategy for GreedyFishingStrategy {
    fn name(&self) -> &str {
        "FishingGreedy"
    }

    fn choose_action(&self, actions: &[Value], _snapshot: &GameSnapshot) -> Value {
        if actions.is_empty() {
            return serde_json::json!({"action_type": "EncounterAbort"});
        }

        let best = actions
            .iter()
            .max_by_key(|a| {
                a.get("card_details")
                    .and_then(|d| d.get("max_fishing_value"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0)
            })
            .unwrap();

        serde_json::json!({
            "action_type": "EncounterPlayCard",
            "card_id": best.get("card_id").and_then(|v| v.as_u64()).unwrap_or(0)
        })
    }
}
