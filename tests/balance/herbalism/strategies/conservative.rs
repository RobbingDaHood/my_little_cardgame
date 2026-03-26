use serde_json::Value;

use crate::strategies::{GameSnapshot, Strategy};

/// Conservative strategy: minimizes cost per play. Picks cards with no
/// Stamina/Health costs first (durability-only cards). Among costless cards,
/// picks the narrowest match (fewest characteristics) for precision to avoid
/// over-elimination. Maximizes durability conservation.
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

        pick_conservative_card(&play_cards)
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

/// Pick the safest card: prefer no extra costs (Stamina/Health), then narrowest match.
fn pick_conservative_card(play_cards: &[Value]) -> Value {
    let no_extra_cost: Vec<&Value> = play_cards.iter().filter(|c| !has_extra_cost(c)).collect();

    if !no_extra_cost.is_empty() {
        return no_extra_cost
            .iter()
            .min_by_key(|c| match_breadth(c))
            .map(|c| (*c).clone())
            .unwrap_or_else(|| no_extra_cost[0].clone());
    }

    // All cards have extra costs — pick narrowest match
    play_cards
        .iter()
        .min_by_key(|c| match_breadth(c))
        .cloned()
        .unwrap_or_else(|| play_cards[0].clone())
}

/// Check if card has pre-play costs (Stamina or Health — not durability which is post-play).
fn has_extra_cost(card: &Value) -> bool {
    card.get("card_details")
        .and_then(|d| d.get("has_cost"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Count how many characteristics the card's match mode targets.
fn match_breadth(card: &Value) -> usize {
    let match_info = card.get("card_details").and_then(|d| d.get("match_info"));
    let match_info = match match_info {
        Some(v) if !v.is_null() => v,
        _ => return usize::MAX, // Unknown cards sort last (most conservative)
    };
    let types = match_info
        .get("Or")
        .and_then(|o| o.get("types"))
        .or_else(|| match_info.get("And").and_then(|a| a.get("types")));
    if let Some(arr) = types.and_then(|t| t.as_array()) {
        return arr.len();
    }
    // MostCommon/LeastCommon have a limit field
    match_info
        .get("MostCommon")
        .or_else(|| match_info.get("LeastCommon"))
        .and_then(|m| m.get("limit"))
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as usize
}
