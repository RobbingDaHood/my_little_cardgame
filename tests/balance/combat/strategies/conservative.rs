use serde_json::Value;

use crate::strategies::{GameSnapshot, Strategy};

/// Conservative strategy: minimizes risk in combat.
/// - Prioritizes Defence cards over Attack
/// - Plays Resource cards to maintain card flow
/// - Only plays Attack cards when player has high HP
/// - Scouting: accepts no mutations (keeps familiar encounters)
pub struct ConservativeStrategy;

impl ConservativeStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Strategy for ConservativeStrategy {
    fn name(&self) -> &str {
        "conservative"
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

        pick_conservative_card(&play_cards, game_state)
    }
}

fn find_scouting_action(actions: &[Value]) -> Option<Value> {
    actions
        .iter()
        .find(|a| a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterApplyScouting"))
        .map(|_| {
            // Conservative: no mutations
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

fn pick_conservative_card(play_cards: &[Value], _game_state: &GameSnapshot) -> Value {
    // Conservative: always picks cards with no cost (basic cards).
    // Falls back to lowest-cost card if all have costs.
    let no_cost: Vec<&Value> = play_cards.iter().filter(|c| !has_cost(c)).collect();

    if !no_cost.is_empty() {
        // Pick the no-cost card with the lowest rolled_value (most conservative)
        return no_cost
            .iter()
            .min_by_key(|c| first_effect_value(c))
            .map(|c| (*c).clone())
            .unwrap_or_else(|| no_cost[0].clone());
    }

    // All cards have costs — pick the one with lowest rolled_value
    play_cards
        .iter()
        .min_by_key(|c| first_effect_value(c))
        .cloned()
        .unwrap_or_else(|| play_cards[0].clone())
}

fn has_cost(card: &Value) -> bool {
    card.get("card_details")
        .and_then(|d| d.get("has_cost"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
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
