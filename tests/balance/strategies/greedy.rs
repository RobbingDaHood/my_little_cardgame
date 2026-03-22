use serde_json::Value;

use super::{GameSnapshot, Strategy};

/// Greedy strategy: maximizes immediate combat value.
/// - Attack phase: plays the card with highest rolled_value
/// - Defence phase: plays the card with highest rolled_value
/// - Resource phase: plays any available resource card (draws more cards)
/// - Scouting: accepts all mutations
pub struct GreedyStrategy;

impl GreedyStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Strategy for GreedyStrategy {
    fn name(&self) -> &str {
        "greedy"
    }

    fn choose_action(&self, possible_actions: &[Value], game_state: &GameSnapshot) -> Value {
        if let Some(action) = find_scouting_action(possible_actions) {
            return action;
        }

        if let Some(action) = find_conclude_action(possible_actions) {
            return action;
        }

        let play_cards = filter_play_card_actions(possible_actions);
        if play_cards.is_empty() {
            return pick_any_encounter(possible_actions);
        }

        best_card_by_value(&play_cards, game_state)
    }
}

fn find_scouting_action(actions: &[Value]) -> Option<Value> {
    actions
        .iter()
        .find(|a| a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterApplyScouting"))
        .map(|_| {
            // Greedy accepts all scouting mutations — pass empty to accept defaults
            serde_json::json!({"action_type": "EncounterApplyScouting", "card_ids": []})
        })
}

fn find_conclude_action(actions: &[Value]) -> Option<Value> {
    actions
        .iter()
        .find(|a| {
            a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterConcludeEncounter")
        })
        .cloned()
}

fn filter_play_card_actions(actions: &[Value]) -> Vec<Value> {
    actions
        .iter()
        .filter(|a| a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterPlayCard"))
        .cloned()
        .collect()
}

fn pick_any_encounter(actions: &[Value]) -> Value {
    actions
        .iter()
        .find(|a| a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterPickEncounter"))
        .cloned()
        .unwrap_or_else(|| serde_json::json!({"action_type": "NewGame"}))
}

fn best_card_by_value(play_cards: &[Value], _game_state: &GameSnapshot) -> Value {
    // Pick the card with the highest first effect rolled_value
    // This applies regardless of phase — greedy always picks highest value
    play_cards
        .iter()
        .max_by_key(|c| {
            c.get("card_details")
                .and_then(|d| d.get("effects"))
                .and_then(|e| e.as_array())
                .and_then(|arr| arr.first())
                .and_then(|eff| eff.get("rolled_value"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0)
        })
        .cloned()
        .unwrap_or_else(|| play_cards[0].clone())
}
