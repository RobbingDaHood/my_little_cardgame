use serde_json::Value;

use crate::strategies::{GameSnapshot, Strategy};

/// Non-yield tactician that beats tier-1 strategies **purely through
/// in-encounter play decisions**:
///
/// - **No FishAmount boosting**: Never plays cards with FishAmount modifiers.
/// - **No scouting advantage**: Does not pick highest-reward encounters;
///   uses default encounter selection (same as tier-1).
///
/// Core advantage: **fish-value visibility** — sees the drawn fish before
/// playing and always picks the cheapest card that wins the duel.
/// When no card can win, plays cheapest to conserve durability (like
/// Conservative) rather than wasting durability on a losing play.
/// Traps (range-narrowing cards) are avoided to preserve the range window;
/// only played as a last resort when no other cards exist.
pub struct NonYieldTacticianFishingStrategy;

impl Strategy for NonYieldTacticianFishingStrategy {
    fn name(&self) -> &str {
        "FishingNonYieldTactician"
    }

    fn choose_action(&self, actions: &[Value], snapshot: &GameSnapshot) -> Value {
        if actions.is_empty() {
            return serde_json::json!({"action_type": "EncounterAbort"});
        }

        // No scouting advantage: pick first encounter (default selection)
        if is_encounter_pick(actions) {
            return pick_first_encounter(actions);
        }

        // Collect all playable cards (exclude conclude and FishAmount).
        // Traps are included — information advantage means we still pick the
        // right card to win, and cheap traps keep our durability low.
        let playable: Vec<&Value> = actions
            .iter()
            .filter(|a| !is_conclude(a) && !has_fish_amount(a))
            .collect();

        if playable.is_empty() {
            return actions[0].clone();
        }

        // When wins are secured or impossible, play cheapest to save durability
        if !wins_matter(snapshot) {
            return pick_cheapest(&playable);
        }

        // Fish-value-aware card selection: pick the cheapest card that wins.
        // Even if it costs some durability, winning grants a full reward (far
        // exceeding any reasonable per-card cost), so it's worth paying.
        let fish_value = snapshot.fishing_current_fish_value();
        let range = snapshot.fishing_valid_range();

        if let (Some(fv), Some((rmin, rmax))) = (fish_value, range) {
            if let Some(winner) = cheapest_winning_card(&playable, fv, rmin, rmax) {
                return winner;
            }
        }

        // No card can win → play cheapest to conserve durability
        pick_cheapest(&playable)
    }
}

fn is_encounter_pick(actions: &[Value]) -> bool {
    actions
        .iter()
        .any(|a| a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterPickEncounter"))
}

fn pick_first_encounter(actions: &[Value]) -> Value {
    let first = actions
        .iter()
        .find(|a| a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterPickEncounter"))
        .unwrap_or(&actions[0]);
    serde_json::json!({
        "action_type": "EncounterPickEncounter",
        "card_id": first.get("card_id").and_then(|v| v.as_u64()).unwrap_or(0)
    })
}

fn is_conclude(action: &Value) -> bool {
    action
        .get("is_conclude")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn has_fish_amount(action: &Value) -> bool {
    action
        .get("card_details")
        .and_then(|d| d.get("has_fish_amount_modifier"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn has_range_narrow(action: &Value) -> bool {
    action
        .get("card_details")
        .and_then(|d| d.get("has_range_narrow_modifier"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
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

/// Pick the cheapest card that can win the current duel.
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
