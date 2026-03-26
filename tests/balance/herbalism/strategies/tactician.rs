use serde_json::Value;
use std::collections::HashMap;

use crate::strategies::{GameSnapshot, Strategy};

/// Tactician strategy: reads plant composition from the encounter state and
/// selects the optimal match card. When many plants remain, uses broad Or-mode
/// cards for efficient elimination. When close to 1 remaining, switches to
/// narrow And/LeastCommon to avoid over-elimination. Avoids expensive cards
/// (Stamina/Health costs) when cheaper alternatives suffice.
pub struct TacticianStrategy;

impl TacticianStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Strategy for TacticianStrategy {
    fn name(&self) -> &str {
        "tactician"
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

        pick_tactical_card(&play_cards, game_state)
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

/// Analyze plant composition and choose optimal card.
fn pick_tactical_card(play_cards: &[Value], game_state: &GameSnapshot) -> Value {
    let plant_count = game_state.herbalism_plant_count().unwrap_or(0);
    let characteristics = game_state
        .herbalism_plant_characteristics()
        .unwrap_or_default();

    // Build frequency map of characteristics across surviving plants
    let char_freq = build_characteristic_frequency(&characteristics);

    if plant_count <= 2 {
        // Close to winning — need precise elimination to leave exactly 1
        pick_narrow_card(play_cards, &char_freq, plant_count)
    } else {
        // Many plants — broad elimination is efficient
        pick_broad_card(play_cards, &char_freq, plant_count)
    }
}

fn build_characteristic_frequency(plant_chars: &[Vec<String>]) -> HashMap<String, usize> {
    let mut freq: HashMap<String, usize> = HashMap::new();
    for chars in plant_chars {
        for c in chars {
            *freq.entry(c.clone()).or_insert(0) += 1;
        }
    }
    freq
}

/// When close to 1 plant remaining: pick a card that eliminates exactly
/// (plant_count - 1) plants if possible, or the narrowest match to minimize
/// over-elimination risk. Prefer costless cards.
fn pick_narrow_card(
    play_cards: &[Value],
    char_freq: &HashMap<String, usize>,
    plant_count: usize,
) -> Value {
    // Separate costless from costly cards
    let (no_cost, with_cost): (Vec<&Value>, Vec<&Value>) =
        play_cards.iter().partition(|c| !has_extra_cost(c));

    let target_removal = plant_count.saturating_sub(1);

    // Score each card: how close its expected removal is to target
    let score_card = |card: &Value| -> i64 {
        let match_info = card.get("card_details").and_then(|d| d.get("match_info"));

        let expected_removal = estimate_removal(match_info, char_freq);
        let diff = (expected_removal as i64 - target_removal as i64).abs();
        // Lower diff is better; negate so max_by_key works
        -diff
    };

    // Prefer costless cards that get us to exactly 1 plant
    if !no_cost.is_empty() {
        return no_cost
            .iter()
            .max_by_key(|c| score_card(c))
            .map(|c| (*c).clone())
            .unwrap_or_else(|| no_cost[0].clone());
    }

    if !with_cost.is_empty() {
        return with_cost
            .iter()
            .max_by_key(|c| score_card(c))
            .map(|c| (*c).clone())
            .unwrap_or_else(|| with_cost[0].clone());
    }

    play_cards[0].clone()
}

/// When many plants remain: pick the broadest match for efficient mass elimination.
/// Prefer low-cost cards.
fn pick_broad_card(
    play_cards: &[Value],
    char_freq: &HashMap<String, usize>,
    _plant_count: usize,
) -> Value {
    let (no_cost, with_cost): (Vec<&Value>, Vec<&Value>) =
        play_cards.iter().partition(|c| !has_extra_cost(c));

    let score_card = |card: &Value| -> usize {
        let match_info = card.get("card_details").and_then(|d| d.get("match_info"));
        estimate_removal(match_info, char_freq)
    };

    // Prefer costless cards with highest removal
    if !no_cost.is_empty() {
        return no_cost
            .iter()
            .max_by_key(|c| score_card(c))
            .map(|c| (*c).clone())
            .unwrap_or_else(|| no_cost[0].clone());
    }

    if !with_cost.is_empty() {
        return with_cost
            .iter()
            .max_by_key(|c| score_card(c))
            .map(|c| (*c).clone())
            .unwrap_or_else(|| with_cost[0].clone());
    }

    play_cards[0].clone()
}

/// Estimate how many plants a card's match mode would remove given current
/// characteristic frequency.
fn estimate_removal(match_info: Option<&Value>, char_freq: &HashMap<String, usize>) -> usize {
    let match_info = match match_info {
        Some(v) if !v.is_null() => v,
        _ => return 1, // Default estimate for unknown cards
    };

    // HerbalismMatchMode is externally tagged: {"Or": {"types": [...]}}
    if let Some(or_obj) = match_info.get("Or") {
        let empty = vec![];
        let types = or_obj
            .get("types")
            .and_then(|t| t.as_array())
            .unwrap_or(&empty);
        // Or mode: remove plants with ANY listed characteristic
        // Rough estimate: union of plants with any of the listed characteristics
        let mut removed: usize = 0;
        for t in types {
            if let Some(name) = t.as_str() {
                removed += char_freq.get(name).copied().unwrap_or(0);
            }
        }
        // May double-count plants with multiple matching characteristics, but
        // overestimate is fine for strategy selection
        return removed;
    }

    if let Some(and_obj) = match_info.get("And") {
        let empty = vec![];
        let types = and_obj
            .get("types")
            .and_then(|t| t.as_array())
            .unwrap_or(&empty);
        if types.is_empty() {
            return 0;
        }
        // And mode: remove plants with ALL listed characteristics
        // Rough estimate: min frequency of the listed characteristics
        return types
            .iter()
            .filter_map(|t| t.as_str().and_then(|name| char_freq.get(name).copied()))
            .min()
            .unwrap_or(0);
    }

    if let Some(mc_obj) = match_info.get("MostCommon") {
        let limit = mc_obj.get("limit").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
        // Remove plants matching the most common characteristic(s)
        let mut freqs: Vec<usize> = char_freq.values().copied().collect();
        freqs.sort_unstable_by(|a, b| b.cmp(a));
        return freqs.iter().take(limit).sum();
    }

    if let Some(lc_obj) = match_info.get("LeastCommon") {
        let limit = lc_obj.get("limit").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
        // Remove plants matching the least common characteristic(s)
        let mut freqs: Vec<usize> = char_freq.values().copied().collect();
        freqs.sort_unstable();
        return freqs.iter().take(limit).sum();
    }

    1
}

fn has_extra_cost(card: &Value) -> bool {
    card.get("card_details")
        .and_then(|d| d.get("has_cost"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}
