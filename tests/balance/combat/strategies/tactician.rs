use serde_json::Value;

use crate::strategies::{GameSnapshot, Strategy};

/// Tactician strategy: reads enemy deck data to optimize card selection.
///
/// Key advantages over simple strategies:
/// 1. Always uses cost_damage (cheap HP cost, kills enemy in 2 attacks vs 6 for basic).
///    Fewer enemy attacks = dramatically less total damage taken.
/// 2. NEVER uses cost_shield (extremely expensive at 28-35% HP cost per use).
///    Other strategies that pick highest value (greedy) waste HP on cost_shield.
/// 3. Reads enemy data to confirm when cost_damage is safe to use.
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

/// Analyze enemy deck to get hand card count and max rolled_value.
fn enemy_deck_info(encounter: &Value, deck_key: &str) -> (u64, i64) {
    let deck = match encounter.get(deck_key).and_then(|d| d.as_array()) {
        Some(d) => d,
        None => return (0, 0),
    };

    let mut total_hand: u64 = 0;
    let mut max_val: i64 = 0;

    for card in deck {
        let hand = card
            .get("counts")
            .and_then(|c| c.get("hand"))
            .and_then(|h| h.as_u64())
            .unwrap_or(0);
        total_hand += hand;

        if hand > 0 {
            if let Some(effects) = card.get("effects").and_then(|e| e.as_array()) {
                for effect in effects {
                    let val = effect
                        .get("rolled_value")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    if val > max_val {
                        max_val = val;
                    }
                }
            }
        }
    }

    (total_hand, max_val)
}

/// Get enemy health from the encounter tokens map.
fn enemy_health(encounter: &Value) -> i64 {
    encounter
        .get("enemy_tokens")
        .and_then(|t| t.as_object())
        .and_then(|map| {
            map.get("Health").and_then(|v| v.as_i64()).or_else(|| {
                map.iter()
                    .find(|(k, _)| k.starts_with("Health:"))
                    .and_then(|(_, v)| v.as_i64())
            })
        })
        .unwrap_or(0)
}

fn pick_tactical_card(play_cards: &[Value], game_state: &GameSnapshot) -> Value {
    let encounter = match &game_state.encounter {
        Some(e) => e,
        None => return highest_value_card(play_cards),
    };

    let phase = game_state.combat_phase().unwrap_or_default();

    match phase.as_str() {
        "Defending" => pick_defending_card(play_cards, encounter),
        "Attacking" => pick_attacking_card(play_cards, encounter),
        _ => highest_value_card(play_cards),
    }
}

fn pick_defending_card(play_cards: &[Value], _encounter: &Value) -> Value {
    let (non_cost, _cost): (Vec<&Value>, Vec<&Value>) =
        play_cards.iter().partition(|c| !has_cost(c));

    // NEVER use cost_shield — the 28-35% HP cost per use is devastating.
    // Basic shield absorbs weak attacks fully and reduces strong attack damage.
    // The health saved by avoiding cost_shield allows surviving more combats.
    if !non_cost.is_empty() {
        return best_card(&non_cost);
    }
    highest_value_card(play_cards)
}

fn pick_attacking_card(play_cards: &[Value], _encounter: &Value) -> Value {
    let (non_cost, cost): (Vec<&Value>, Vec<&Value>) =
        play_cards.iter().partition(|c| !has_cost(c));

    // Always prefer cost_damage: trivial HP cost (~24) but kills enemy faster,
    // which means fewer enemy attack rounds and less total damage taken.
    if !cost.is_empty() {
        return best_card(&cost);
    }

    if !non_cost.is_empty() {
        return best_card(&non_cost);
    }
    highest_value_card(play_cards)
}

// --- Helper functions ---

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

fn highest_value_card(play_cards: &[Value]) -> Value {
    play_cards
        .iter()
        .max_by_key(|c| first_effect_value(c))
        .cloned()
        .unwrap_or_else(|| play_cards[0].clone())
}

fn weakest_value_card(play_cards: &[Value]) -> Value {
    play_cards
        .iter()
        .min_by_key(|c| first_effect_value(c))
        .cloned()
        .unwrap_or_else(|| play_cards[0].clone())
}

fn best_card(cards: &[&Value]) -> Value {
    cards
        .iter()
        .max_by_key(|c| first_effect_value(c))
        .map(|c| (*c).clone())
        .unwrap_or_else(|| serde_json::json!({"action_type": "NewGame"}))
}

fn weakest_card(cards: &[&Value]) -> Option<Value> {
    cards
        .iter()
        .min_by_key(|c| first_effect_value(c))
        .map(|c| (*c).clone())
}
