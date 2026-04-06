use serde_json::Value;

use crate::strategies::{GameSnapshot, Strategy};

/// Yield-optimizer tactician that maximises Fish-per-durability through:
///
/// 1. **Scouting** — always picks the highest-reward encounter (harder fish,
///    but reward scales with difficulty via scouting).
/// 2. **FishAmount boosting** — plays FishAmount-modifier cards early to
///    increase the reward multiplier before securing wins.
/// 3. **Fish-value-aware card selection** — uses the visible fish value to
///    pick the cheapest card that wins, conserving durability.
///
/// Once enough wins are secured (or impossible), switches to cheapest cards.
pub struct YieldOptimizerFishingStrategy;

impl Strategy for YieldOptimizerFishingStrategy {
    fn name(&self) -> &str {
        "FishingYieldOptimizer"
    }

    fn choose_action(&self, actions: &[Value], snapshot: &GameSnapshot) -> Value {
        if actions.is_empty() {
            return serde_json::json!({"action_type": "EncounterAbort"});
        }

        if is_encounter_pick(actions) {
            return pick_highest_reward(actions);
        }

        let card_actions: Vec<&Value> = actions
            .iter()
            .filter(|a| {
                !a.get("is_conclude")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            })
            .collect();

        if card_actions.is_empty() {
            return actions[0].clone();
        }

        if !wins_matter(snapshot) {
            return pick_cheapest(&card_actions);
        }

        // Phase 1: Boost FishAmount early (first few rounds)
        let round = snapshot.fishing_round().unwrap_or(1);
        if round <= 3 {
            if let Some(fa) = cheapest_fish_amount_card(&card_actions) {
                return fa;
            }
        }

        // Phase 2: Fish-value-aware card selection
        let fish_value = snapshot.fishing_current_fish_value();
        let range = snapshot.fishing_valid_range();

        if let (Some(fv), Some((rmin, rmax))) = (fish_value, range) {
            if let Some(winner) = cheapest_winning_card(&card_actions, fv, rmin, rmax) {
                return winner;
            }
        }

        pick_cheapest(&card_actions)
    }
}

fn is_encounter_pick(actions: &[Value]) -> bool {
    actions
        .iter()
        .any(|a| a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterPickEncounter"))
}

fn pick_highest_reward(actions: &[Value]) -> Value {
    let best = actions
        .iter()
        .max_by_key(|a| a.get("fish_reward").and_then(|v| v.as_i64()).unwrap_or(0))
        .unwrap();
    serde_json::json!({
        "action_type": "EncounterPickEncounter",
        "card_id": best.get("card_id").and_then(|v| v.as_u64()).unwrap_or(0)
    })
}

fn wins_matter(snapshot: &GameSnapshot) -> bool {
    let turns_won = match snapshot.fishing_turns_won() {
        Some(w) => w,
        None => return true,
    };
    let needed = snapshot.fishing_win_turns_needed().unwrap_or(5);

    if turns_won >= needed as i32 {
        return false;
    }

    let round = snapshot.fishing_round().unwrap_or(1);
    let max_turns = snapshot.fishing_max_turns().unwrap_or(12);
    let rounds_remaining = max_turns.saturating_sub(round - 1) as i32;
    let wins_still_needed = needed as i32 - turns_won;
    wins_still_needed <= rounds_remaining
}

fn cheapest_fish_amount_card(cards: &[&Value]) -> Option<Value> {
    let fa_cards: Vec<&&Value> = cards
        .iter()
        .filter(|a| {
            a.get("card_details")
                .and_then(|d| d.get("has_fish_amount_modifier"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        })
        .collect();

    if fa_cards.is_empty() {
        return None;
    }

    let best = fa_cards
        .iter()
        .min_by_key(|a| get_durability_cost(a))
        .unwrap();
    Some(build_play_action(best))
}

fn cheapest_winning_card(cards: &[&Value], fish_value: i64, rmin: i64, rmax: i64) -> Option<Value> {
    let mut winners: Vec<(&Value, i64)> = cards
        .iter()
        .filter_map(|card| {
            if card_can_win(card, fish_value, rmin, rmax) {
                Some((*card, get_durability_cost(card)))
            } else {
                None
            }
        })
        .collect();
    if winners.is_empty() {
        return None;
    }
    winners.sort_by_key(|(_, cost)| *cost);
    Some(build_play_action(winners[0].0))
}

fn card_can_win(card: &Value, fish_value: i64, rmin: i64, rmax: i64) -> bool {
    get_fishing_values(card).iter().any(|&v| {
        let result = (v - fish_value).max(0);
        result >= rmin && result <= rmax
    })
}

fn pick_cheapest(cards: &[&Value]) -> Value {
    let best = cards.iter().min_by_key(|a| get_durability_cost(a)).unwrap();
    build_play_action(best)
}

fn build_play_action(action: &Value) -> Value {
    serde_json::json!({
        "action_type": "EncounterPlayCard",
        "card_id": action.get("card_id").and_then(|v| v.as_u64()).unwrap_or(0)
    })
}

fn get_fishing_values(action: &Value) -> Vec<i64> {
    action
        .get("card_details")
        .and_then(|d| d.get("fishing_values"))
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect())
        .unwrap_or_default()
}

fn get_durability_cost(action: &Value) -> i64 {
    action
        .get("card_details")
        .and_then(|d| d.get("total_durability_cost"))
        .and_then(|v| v.as_i64())
        .unwrap_or(i64::MAX)
}
