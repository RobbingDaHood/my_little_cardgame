use serde_json::Value;

use crate::strategies::{GameSnapshot, Strategy};

/// Fishing Tactician that combines two advantages:
///
/// 1. **Encounter selection** — always picks the highest-reward encounter from
///    scouting choices.  Scouting scales rewards with difficulty, so harder
///    encounters give significantly more Fish.  Simple strategies pick
///    arbitrarily, getting average rewards.
///
/// 2. **Multi-value card preference** — multi-value cards benefit from the
///    auto-select mechanic (the game picks the best sub-value per fish),
///    achieving ~56% per-round win rates.  Once victory is secured or
///    mathematically impossible, switches to cheapest cards to conserve
///    durability.
pub struct TacticianFishingStrategy;

impl Strategy for TacticianFishingStrategy {
    fn name(&self) -> &str {
        "FishingTactician"
    }

    fn choose_action(&self, actions: &[Value], snapshot: &GameSnapshot) -> Value {
        if actions.is_empty() {
            return serde_json::json!({"action_type": "EncounterAbort"});
        }

        // Encounter selection: pick the encounter with the highest fish reward
        if is_encounter_pick(actions) {
            return pick_highest_reward(actions);
        }

        // Card play: filter out conclude (shouldn't appear, but be safe)
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

        // When wins no longer matter (already won or doomed), play cheapest
        if !wins_matter(snapshot) {
            return pick_cheapest(&card_actions);
        }

        // Prefer multi-value cards (highest win rate via auto-select)
        if let Some(mv) = cheapest_multi_value(&card_actions) {
            return mv;
        }

        // Fallback: play cheapest card
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

/// Returns false when victory is already secured or mathematically impossible.
fn wins_matter(snapshot: &GameSnapshot) -> bool {
    let turns_won = match snapshot.fishing_turns_won() {
        Some(w) => w,
        None => return true,
    };
    let round = snapshot.fishing_round().unwrap_or(1);
    let max_turns = snapshot.fishing_max_turns().unwrap_or(8);
    let needed = snapshot.fishing_win_turns_needed().unwrap_or(4);

    if turns_won >= needed {
        return false;
    }

    let rounds_remaining = max_turns.saturating_sub(round - 1);
    let wins_still_needed = needed - turns_won;
    if wins_still_needed > rounds_remaining {
        return false;
    }

    true
}

fn cheapest_multi_value(cards: &[&Value]) -> Option<Value> {
    let multi: Vec<&&Value> = cards
        .iter()
        .filter(|a| get_num_fishing_values(a) > 1)
        .collect();

    if multi.is_empty() {
        return None;
    }

    let best = multi.iter().min_by_key(|a| get_durability_cost(a)).unwrap();
    Some(build_play_action(best))
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

fn get_num_fishing_values(action: &Value) -> i64 {
    action
        .get("card_details")
        .and_then(|d| d.get("num_fishing_values"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
}

fn get_durability_cost(action: &Value) -> i64 {
    action
        .get("card_details")
        .and_then(|d| d.get("total_durability_cost"))
        .and_then(|v| v.as_i64())
        .unwrap_or(i64::MAX)
}
