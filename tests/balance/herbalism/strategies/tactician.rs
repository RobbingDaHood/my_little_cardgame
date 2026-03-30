use serde_json::Value;
use std::collections::HashMap;

use crate::strategies::{GameSnapshot, Strategy};

/// Tactician strategy: reads plant composition and computes exact removal counts
/// to select the optimal card. Never plays a card that would eliminate ALL plants
/// (which causes a loss). Prefers free cards over costly ones, and targets
/// removal that leaves exactly 1 plant for a win.
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

fn pick_tactical_card(play_cards: &[Value], game_state: &GameSnapshot) -> Value {
    let plant_count = game_state.herbalism_plant_count().unwrap_or(0);
    let plant_chars = game_state
        .herbalism_plant_characteristics()
        .unwrap_or_default();

    if plant_count == 0 {
        return play_cards[0].clone();
    }

    let char_freq = build_char_freq(&plant_chars);
    let target_removal = plant_count - 1; // Leave exactly 1 → win

    // Score each card with exact removal
    let scored: Vec<(usize, &Value)> = play_cards
        .iter()
        .map(|c| {
            let match_info = c.get("card_details").and_then(|d| d.get("match_info"));
            let removal = exact_removal(match_info, &plant_chars, &char_freq);
            (removal, c)
        })
        .collect();

    // Safety: filter out cards that would remove ALL plants (0 remaining = loss)
    let safe: Vec<(usize, &Value)> = scored
        .iter()
        .filter(|(removal, _)| *removal < plant_count)
        .cloned()
        .collect();

    let candidates = if safe.is_empty() { &scored } else { &safe };

    type ScoredCard<'a> = (usize, &'a Value);

    // Partition by cost
    let (no_cost, with_cost): (Vec<&ScoredCard>, Vec<&ScoredCard>) =
        candidates.iter().partition(|(_, c)| !has_extra_cost(c));

    // Scoring: perfect win (removal == target) > high removal > low removal
    let score = |removal: usize| -> (u8, usize) {
        if removal == target_removal {
            (2, 0) // Perfect: leaves exactly 1 plant
        } else {
            (1, removal) // More removal is better
        }
    };

    let pick_best = |cards: &[&ScoredCard]| -> Option<Value> {
        cards
            .iter()
            .max_by_key(|(removal, _)| score(*removal))
            .map(|(_, c)| (*c).clone())
    };

    if let Some(best) = pick_best(&no_cost) {
        return best;
    }
    if let Some(best) = pick_best(&with_cost) {
        return best;
    }

    play_cards[0].clone()
}

fn build_char_freq(plant_chars: &[Vec<String>]) -> HashMap<String, usize> {
    let mut freq: HashMap<String, usize> = HashMap::new();
    for chars in plant_chars {
        for c in chars {
            *freq.entry(c.clone()).or_insert(0) += 1;
        }
    }
    freq
}

/// Compute exact number of plants removed by a card's match mode, accounting
/// for multi-characteristic plants correctly (no double-counting).
fn exact_removal(
    match_info: Option<&Value>,
    plant_chars: &[Vec<String>],
    char_freq: &HashMap<String, usize>,
) -> usize {
    let match_info = match match_info {
        Some(v) if !v.is_null() => v,
        _ => return 0,
    };

    if let Some(or_obj) = match_info.get("Or") {
        let types = extract_types(or_obj);
        return plant_chars
            .iter()
            .filter(|p| types.iter().any(|t| p.contains(t)))
            .count();
    }

    if let Some(and_obj) = match_info.get("And") {
        let types = extract_types(and_obj);
        if types.is_empty() {
            return 0;
        }
        return plant_chars
            .iter()
            .filter(|p| types.iter().all(|t| p.contains(t)))
            .count();
    }

    if let Some(mc_obj) = match_info.get("MostCommon") {
        let limit = mc_obj.get("limit").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
        let target_chars = most_common_chars(char_freq, limit);
        return plant_chars
            .iter()
            .filter(|p| target_chars.iter().any(|t| p.contains(t)))
            .count();
    }

    if let Some(lc_obj) = match_info.get("LeastCommon") {
        let limit = lc_obj.get("limit").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
        let target_chars = least_common_chars(char_freq, limit);
        return plant_chars
            .iter()
            .filter(|p| target_chars.iter().any(|t| p.contains(t)))
            .count();
    }

    0
}

fn extract_types(obj: &Value) -> Vec<String> {
    obj.get("types")
        .and_then(|t| t.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn most_common_chars(char_freq: &HashMap<String, usize>, limit: usize) -> Vec<String> {
    let mut pairs: Vec<(&String, &usize)> = char_freq.iter().collect();
    pairs.sort_unstable_by(|a, b| b.1.cmp(a.1));
    pairs
        .into_iter()
        .take(limit)
        .map(|(k, _)| k.clone())
        .collect()
}

fn least_common_chars(char_freq: &HashMap<String, usize>, limit: usize) -> Vec<String> {
    let mut pairs: Vec<(&String, &usize)> = char_freq.iter().collect();
    pairs.sort_unstable_by(|a, b| a.1.cmp(b.1));
    pairs
        .into_iter()
        .take(limit)
        .map(|(k, _)| k.clone())
        .collect()
}

fn has_extra_cost(card: &Value) -> bool {
    card.get("card_details")
        .and_then(|d| d.get("has_cost"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}
