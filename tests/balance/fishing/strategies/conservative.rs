use serde_json::Value;

use crate::strategies::{GameSnapshot, Strategy};

/// Plays the card with the lowest durability cost.
pub struct ConservativeFishingStrategy;

impl Strategy for ConservativeFishingStrategy {
    fn name(&self) -> &str {
        "FishingConservative"
    }

    fn choose_action(&self, actions: &[Value], _snapshot: &GameSnapshot) -> Value {
        if actions.is_empty() {
            return serde_json::json!({"action_type": "EncounterAbort"});
        }

        let card_actions: Vec<&Value> = actions
            .iter()
            .filter(|a| {
                !a.get("is_conclude")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            })
            .collect();

        if card_actions.is_empty() {
            // Only conclude actions available — return the first one
            return actions[0].clone();
        }

        let best = card_actions
            .iter()
            .min_by_key(|a| {
                a.get("card_details")
                    .and_then(|d| d.get("total_durability_cost"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(i64::MAX)
            })
            .unwrap();

        serde_json::json!({
            "action_type": "EncounterPlayCard",
            "card_id": best.get("card_id").and_then(|v| v.as_u64()).unwrap_or(0)
        })
    }
}
