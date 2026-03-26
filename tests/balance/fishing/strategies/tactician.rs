use serde_json::Value;

use crate::strategies::{GameSnapshot, Strategy};

/// Three-lever optimization:
/// 1. Best-matching value selection (highest FishingValue within valid range)
/// 2. Range management (play range-expanding cards early)
/// 3. FishAmount boosting (play FishAmount cards when range is favorable)
pub struct TacticianFishingStrategy;

impl Strategy for TacticianFishingStrategy {
    fn name(&self) -> &str {
        "FishingTactician"
    }

    fn choose_action(&self, actions: &[Value], snapshot: &GameSnapshot) -> Value {
        if actions.is_empty() {
            return serde_json::json!({"action_type": "EncounterAbort"});
        }

        // Separate conclude actions from card actions
        let mut range_cards: Vec<&Value> = Vec::new();
        let mut fish_amount_cards: Vec<&Value> = Vec::new();
        let mut value_cards: Vec<&Value> = Vec::new();
        let mut utility_cards: Vec<&Value> = Vec::new();
        let mut conclude_action: Option<&Value> = None;

        for action in actions {
            if action
                .get("is_conclude")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                conclude_action = Some(action);
                continue;
            }

            let details = action.get("card_details");
            let has_range = details
                .and_then(|d| d.get("has_range_modifier"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let has_fish_amount = details
                .and_then(|d| d.get("has_fish_amount_modifier"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let fishing_value = details
                .and_then(|d| d.get("max_fishing_value"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            if has_range {
                range_cards.push(action);
            } else if has_fish_amount {
                fish_amount_cards.push(action);
            } else if fishing_value > 0 {
                value_cards.push(action);
            } else {
                utility_cards.push(action);
            }
        }

        // Lever 1: Range management — play range-expanding cards early
        let encounter_round = estimate_round(snapshot);
        if encounter_round < 3 && !range_cards.is_empty() {
            let card = range_cards[0];
            return build_play_action(card);
        }

        // Lever 2: FishAmount boosting — if we have range advantage, boost rewards
        if !fish_amount_cards.is_empty() {
            let card = fish_amount_cards[0];
            return build_play_action(card);
        }

        // Lever 3: Best-matching value — prefer cards within valid range
        if !value_cards.is_empty() {
            let (range_min, range_max) = get_valid_range(snapshot);

            let in_range: Vec<&&Value> = value_cards
                .iter()
                .filter(|a| {
                    let val = get_fishing_value(a);
                    val >= range_min && val <= range_max
                })
                .collect();

            if !in_range.is_empty() {
                let best = in_range
                    .iter()
                    .max_by_key(|a| get_fishing_value(a))
                    .unwrap();
                return build_play_action(best);
            }

            // No in-range cards: pick lowest value to minimize overshoot
            let best = value_cards
                .iter()
                .min_by_key(|a| get_fishing_value(a))
                .unwrap();
            return build_play_action(best);
        }

        // Fallback: play any utility card
        if !utility_cards.is_empty() {
            return build_play_action(utility_cards[0]);
        }

        // Last resort: conclude if available, otherwise first action
        if let Some(conclude) = conclude_action {
            return conclude.clone();
        }

        build_play_action(actions.first().unwrap())
    }
}

fn build_play_action(action: &Value) -> Value {
    serde_json::json!({
        "action_type": "EncounterPlayCard",
        "card_id": action.get("card_id").and_then(|v| v.as_u64()).unwrap_or(0)
    })
}

fn estimate_round(snapshot: &GameSnapshot) -> u32 {
    snapshot
        .encounter
        .as_ref()
        .and_then(|e| e.get("round"))
        .and_then(|r| r.as_u64())
        .unwrap_or(0) as u32
}

fn get_fishing_value(action: &Value) -> i64 {
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
