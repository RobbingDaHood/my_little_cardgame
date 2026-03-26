use serde_json::Value;

use crate::strategies::{GameSnapshot, Strategy};

/// Picks the highest FishingValue card that lands within the valid range.
/// Falls back to the lowest out-of-range card to minimize overshoot.
pub struct GreedyFishingStrategy;

impl Strategy for GreedyFishingStrategy {
    fn name(&self) -> &str {
        "FishingGreedy"
    }

    fn choose_action(&self, actions: &[Value], snapshot: &GameSnapshot) -> Value {
        if actions.is_empty() {
            return serde_json::json!({"action_type": "EncounterAbort"});
        }

        // If the chosen action is a conclude, return it directly
        let (conclude_actions, card_actions): (Vec<&Value>, Vec<&Value>) =
            actions.iter().partition(|a| {
                a.get("is_conclude")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            });

        if card_actions.is_empty() {
            if let Some(conclude) = conclude_actions.first() {
                return (*conclude).clone();
            }
            return serde_json::json!({"action_type": "EncounterAbort"});
        }

        let (range_min, range_max) = get_valid_range(snapshot);

        // Find cards whose max_fishing_value is within [range_min, range_max]
        let in_range: Vec<&&Value> = card_actions
            .iter()
            .filter(|a| {
                let val = fishing_value(a);
                val > 0 && val >= range_min && val <= range_max
            })
            .collect();

        let best = if !in_range.is_empty() {
            // Pick highest in-range card
            in_range
                .iter()
                .max_by_key(|a| fishing_value(a))
                .unwrap()
        } else {
            // No in-range cards: pick lowest value to minimize overshoot
            card_actions
                .iter()
                .min_by_key(|a| {
                    let v = fishing_value(a);
                    if v > 0 { v } else { i64::MAX }
                })
                .unwrap()
        };

        serde_json::json!({
            "action_type": "EncounterPlayCard",
            "card_id": best.get("card_id").and_then(|v| v.as_u64()).unwrap_or(0)
        })
    }
}

fn fishing_value(action: &Value) -> i64 {
    action
        .get("card_details")
        .and_then(|d| d.get("max_fishing_value"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
}

fn get_valid_range(snapshot: &GameSnapshot) -> (i64, i64) {
    let enc = snapshot.encounter.as_ref();
    let min = enc
        .and_then(|e| e.get("valid_range_min"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let max = enc
        .and_then(|e| e.get("valid_range_max"))
        .and_then(|v| v.as_i64())
        .unwrap_or(i64::MAX);
    (min, max)
}
