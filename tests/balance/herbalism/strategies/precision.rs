use serde_json::Value;
use std::collections::HashMap;

use crate::strategies::{GameSnapshot, Strategy};

/// Precision Tactician (yield-optimizer): uses 2-step look-ahead to find
/// winning card sequences that greedy single-card selection misses. When no
/// win is possible from the current hand, minimises durability waste by
/// playing the cheapest card. This differs from the base Tactician which
/// always picks the highest-removal free card greedily.
pub struct PrecisionTacticianStrategy;

impl PrecisionTacticianStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Strategy for PrecisionTacticianStrategy {
    fn name(&self) -> &str {
        "precision_tactician"
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

        pick_precision_card(&play_cards, game_state)
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

struct CardInfo {
    index: usize,
    match_info: Option<Value>,
    has_cost: bool,
}

/// Precision strategy with 2-step look-ahead:
/// 1. If any single card wins (removes exactly N-1 plants) → play it (prefer free)
/// 2. If any 2-card sequence wins → play the first card (prefer sequences starting with free cards)
/// 3. Fallback: play highest-removal free card (same as Tactician)
/// 4. If only costly cards remain: play cheapest (minimise durability waste)
fn pick_precision_card(play_cards: &[Value], game_state: &GameSnapshot) -> Value {
    let plant_chars = game_state
        .herbalism_plant_characteristics()
        .unwrap_or_default();

    if plant_chars.is_empty() {
        return play_cards[0].clone();
    }

    let target_remaining = 1_usize;
    let char_freq = build_char_freq(&plant_chars);

    let cards: Vec<CardInfo> = play_cards
        .iter()
        .enumerate()
        .map(|(i, c)| CardInfo {
            index: i,
            match_info: c
                .get("card_details")
                .and_then(|d| d.get("match_info"))
                .cloned(),
            has_cost: c
                .get("card_details")
                .and_then(|d| d.get("has_cost"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        })
        .collect();

    // Step 1: Check for 1-step wins
    let mut one_step_winners: Vec<&CardInfo> = Vec::new();
    for card in &cards {
        let removal = compute_removal(&card.match_info, &plant_chars, &char_freq);
        if removal > 0 && plant_chars.len() - removal == target_remaining {
            one_step_winners.push(card);
        }
    }
    if !one_step_winners.is_empty() {
        // Prefer free card among winners
        let best = one_step_winners
            .iter()
            .min_by_key(|c| if c.has_cost { 1_u8 } else { 0 })
            .unwrap();
        return play_cards[best.index].clone();
    }

    // Step 2: Check for 2-step wins
    let mut two_step_winners: Vec<(usize, usize)> = Vec::new();
    for (i, first) in cards.iter().enumerate() {
        let remaining_after_first = simulate_removal(&first.match_info, &plant_chars, &char_freq);
        if remaining_after_first.is_empty() || remaining_after_first.len() == plant_chars.len() {
            continue; // Total wipe or no effect — skip
        }
        let new_freq = build_char_freq(&remaining_after_first);
        for (j, second) in cards.iter().enumerate() {
            if i == j {
                continue;
            }
            let removal_j = compute_removal(&second.match_info, &remaining_after_first, &new_freq);
            if removal_j > 0 && remaining_after_first.len() - removal_j == target_remaining {
                two_step_winners.push((i, j));
            }
        }
    }
    if !two_step_winners.is_empty() {
        // Prefer sequences where the first card is free, then fewest total costly cards
        let best = two_step_winners
            .iter()
            .min_by_key(|(i, j)| {
                let cost_first = if cards[*i].has_cost { 1_u8 } else { 0 };
                let cost_second = if cards[*j].has_cost { 1_u8 } else { 0 };
                (cost_first, cost_second)
            })
            .unwrap();
        return play_cards[best.0].clone();
    }

    // Step 3: No win in 1-2 steps — greedy fallback with safety filter
    let scored: Vec<(usize, &CardInfo)> = cards
        .iter()
        .map(|c| {
            let removal = compute_removal(&c.match_info, &plant_chars, &char_freq);
            (removal, c)
        })
        .collect();

    // Safety: don't remove all plants (causes loss)
    let safe: Vec<(usize, &CardInfo)> = scored
        .iter()
        .filter(|(removal, _)| *removal < plant_chars.len())
        .cloned()
        .collect();

    let candidates = if safe.is_empty() { &scored } else { &safe };

    // Among safe cards, prefer free cards with highest removal
    candidates
        .iter()
        .max_by_key(|(removal, card)| {
            let cost_rank = if card.has_cost { 0_u8 } else { 1 };
            (cost_rank, *removal)
        })
        .map(|(_, card)| play_cards[card.index].clone())
        .unwrap_or_else(|| play_cards[0].clone())
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

/// Compute how many plants would be removed by this card's match mode.
fn compute_removal(
    match_info: &Option<Value>,
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

/// Simulate playing a card and return the remaining plants after removal.
fn simulate_removal(
    match_info: &Option<Value>,
    plant_chars: &[Vec<String>],
    char_freq: &HashMap<String, usize>,
) -> Vec<Vec<String>> {
    let match_info = match match_info {
        Some(v) if !v.is_null() => v,
        _ => return plant_chars.to_vec(),
    };

    let matches = |plant: &Vec<String>| -> bool {
        if let Some(or_obj) = match_info.get("Or") {
            let types = extract_types(or_obj);
            return types.iter().any(|t| plant.contains(t));
        }
        if let Some(and_obj) = match_info.get("And") {
            let types = extract_types(and_obj);
            return !types.is_empty() && types.iter().all(|t| plant.contains(t));
        }
        if let Some(mc_obj) = match_info.get("MostCommon") {
            let limit = mc_obj.get("limit").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
            let target_chars = most_common_chars(char_freq, limit);
            return target_chars.iter().any(|t| plant.contains(t));
        }
        if let Some(lc_obj) = match_info.get("LeastCommon") {
            let limit = lc_obj.get("limit").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
            let target_chars = least_common_chars(char_freq, limit);
            return target_chars.iter().any(|t| plant.contains(t));
        }
        false
    };

    plant_chars
        .iter()
        .filter(|p| !matches(p))
        .cloned()
        .collect()
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
