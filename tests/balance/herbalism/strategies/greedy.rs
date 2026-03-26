use serde_json::Value;

use crate::strategies::{GameSnapshot, Strategy};

/// Greedy strategy: plays the broadest match card available — maximizes plants
/// eliminated per play. Picks cards with the most characteristics in Or mode,
/// or highest rolled_value as fallback. Risks over-elimination but clears
/// encounters quickly.
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

    fn choose_action(&self, possible_actions: &[Value], _game_state: &GameSnapshot) -> Value {
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

        broadest_match_card(&play_cards)
    }
}

fn find_scouting_action(actions: &[Value]) -> Option<Value> {
    actions
        .iter()
        .find(|a| a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterApplyScouting"))
        .map(|_| serde_json::json!({"action_type": "EncounterApplyScouting", "card_ids": []}))
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

/// Pick card with broadest match — most characteristics listed in the match mode.
/// Falls back to highest rolled_value.
fn broadest_match_card(play_cards: &[Value]) -> Value {
    play_cards
        .iter()
        .max_by_key(|c| {
            let breadth = match_breadth(c);
            let value = first_effect_value(c);
            (breadth, value)
        })
        .cloned()
        .unwrap_or_else(|| play_cards[0].clone())
}

fn match_breadth(card: &Value) -> usize {
    let match_info = card.get("card_details").and_then(|d| d.get("match_info"));
    let match_info = match match_info {
        Some(v) if !v.is_null() => v,
        _ => return 0,
    };
    let types = match_info
        .get("Or")
        .and_then(|o| o.get("types"))
        .or_else(|| match_info.get("And").and_then(|a| a.get("types")));
    if let Some(arr) = types.and_then(|t| t.as_array()) {
        return arr.len();
    }
    // MostCommon/LeastCommon have a limit field instead of types list
    match_info
        .get("MostCommon")
        .or_else(|| match_info.get("LeastCommon"))
        .and_then(|m| m.get("limit"))
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as usize
}

fn first_effect_value(card: &Value) -> i64 {
    card.get("card_details")
        .and_then(|d| d.get("effects"))
        .and_then(|e| e.as_array())
        .and_then(|arr| arr.first())
        .and_then(|eff| eff.get("rolled_value"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
}
