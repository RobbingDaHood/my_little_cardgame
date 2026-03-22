use serde_json::Value;

use super::{GameSnapshot, Strategy};

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

fn pick_conservative_card(play_cards: &[Value], game_state: &GameSnapshot) -> Value {
    let health = game_state.player_health();
    let health_threshold = 5000; // 50% of typical max health of 10000

    // Categorize available cards by kind
    let defence_cards: Vec<&Value> = play_cards
        .iter()
        .filter(|c| card_kind(c) == Some("Defence"))
        .collect();
    let resource_cards: Vec<&Value> = play_cards
        .iter()
        .filter(|c| card_kind(c) == Some("Resource"))
        .collect();
    let attack_cards: Vec<&Value> = play_cards
        .iter()
        .filter(|c| card_kind(c) == Some("Attack"))
        .collect();

    // Priority: Defence > Resource > Attack (unless high HP, then Attack before Resource)
    if health < health_threshold {
        // Low HP: defence first, then resource, then attack
        if let Some(card) = pick_highest_value(&defence_cards) {
            return card.clone();
        }
        if let Some(card) = pick_highest_value(&resource_cards) {
            return card.clone();
        }
        if let Some(card) = pick_highest_value(&attack_cards) {
            return card.clone();
        }
    } else {
        // High HP: attack is acceptable, but still prefer defence
        if let Some(card) = pick_highest_value(&defence_cards) {
            return card.clone();
        }
        if let Some(card) = pick_highest_value(&attack_cards) {
            return card.clone();
        }
        if let Some(card) = pick_highest_value(&resource_cards) {
            return card.clone();
        }
    }

    // Fallback: first available card
    play_cards[0].clone()
}

fn card_kind(card: &Value) -> Option<&str> {
    card.get("card_kind").and_then(|v| v.as_str())
}

fn pick_highest_value<'a>(cards: &[&'a Value]) -> Option<&'a Value> {
    cards.iter().max_by_key(|c| first_effect_value(c)).copied()
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
