use rocket::local::blocking::Client;
use serde_json::Value;

use crate::game_driver::{
    get_json, get_possible_actions, get_snapshot, post_action, DisciplineDriver, GameResult,
};
use crate::strategies::{GameSnapshot, Strategy};

/// Woodcutting discipline driver — implements DisciplineDriver with yield/durability tracking.
pub struct WoodcuttingDisciplineDriver;

impl DisciplineDriver for WoodcuttingDisciplineDriver {
    fn get_encounter_ids(&self, client: &Client) -> Vec<u64> {
        get_woodcutting_encounter_ids(client)
    }

    fn get_encounter_choices_filtered(&self, client: &Client, exclude_ids: &[u64]) -> Vec<Value> {
        get_woodcutting_encounter_choices_filtered(client, exclude_ids)
    }

    fn find_encounter(&self, client: &Client) -> Option<usize> {
        find_woodcutting_encounter(client)
    }

    fn play_encounter(&self, client: &Client, strategy: &dyn Strategy, max_actions: u32) -> u32 {
        play_woodcutting_encounter(client, strategy, max_actions)
    }

    fn pre_encounter(&self, client: &Client) -> Option<Value> {
        let snapshot = get_snapshot(client);
        Some(serde_json::json!({
            "lumber_before": snapshot.player_lumber(),
            "durability_before": snapshot.woodcutting_durability(),
        }))
    }

    fn post_encounter(&self, client: &Client, pre_state: &Option<Value>, result: &mut GameResult) {
        if let Some(pre) = pre_state {
            let snapshot = get_snapshot(client);
            let lumber_before = pre
                .get("lumber_before")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let durability_before = pre
                .get("durability_before")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            let lumber_after = snapshot.player_lumber();
            let durability_after = snapshot.woodcutting_durability();

            let yield_delta = lumber_after - lumber_before;
            let durability_delta = durability_before - durability_after;

            if yield_delta > 0 {
                result.yield_total += yield_delta;
            }
            if durability_delta > 0 {
                result.durability_spent += durability_delta;
            }
        }
    }
}

/// Play a woodcutting encounter to completion using the given strategy.
/// Returns the number of rounds played.
///
/// Woodcutting's `/actions/possible` returns a generic `EncounterPlayCard { card_id: 0 }`
/// placeholder, NOT individual card IDs. We must query the woodcutting hand
/// directly and handle 400 errors for cards the player can't afford.
pub fn play_woodcutting_encounter(
    client: &Client,
    strategy: &dyn Strategy,
    max_actions: u32,
) -> u32 {
    let mut rounds = 0;
    let mut unplayable_card_ids: Vec<u64> = Vec::new();

    for _ in 0..max_actions {
        let snapshot = get_snapshot(client);

        // Check if woodcutting encounter is still active
        if let Some(enc) = &snapshot.encounter {
            let state_type = enc
                .get("encounter_state_type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if state_type != "Woodcutting" {
                return rounds;
            }
            if let Some(outcome) = enc.get("outcome").and_then(|v| v.as_str()) {
                if outcome != "Undecided" {
                    return rounds;
                }
            }
        } else {
            return rounds;
        }

        let possible = get_possible_actions(client);
        let can_play_card = possible
            .iter()
            .any(|a| a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterPlayCard"));
        let has_conclude = possible.iter().any(|a| {
            a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterConcludeEncounter")
        });
        let has_abort = possible
            .iter()
            .any(|a| a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterAbort"));
        let has_scouting = possible.iter().any(|a| {
            a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterApplyScouting")
        });

        if !can_play_card && !has_conclude && !has_abort {
            if has_scouting {
                post_action(
                    client,
                    &serde_json::json!({"action_type": "EncounterApplyScouting", "card_ids": []}),
                );
                continue;
            }
            return rounds;
        }

        // Get woodcutting cards in hand, excluding known-unplayable cards
        let playable: Vec<Value> = if can_play_card {
            get_playable_woodcutting_cards(client, &snapshot)
                .into_iter()
                .filter(|c| {
                    let id = c.get("card_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    !unplayable_card_ids.contains(&id)
                })
                .collect()
        } else {
            vec![]
        };

        if playable.is_empty() && !has_conclude && !has_abort {
            return rounds;
        }

        // If all cards are unplayable (filtered out), conclude or abort
        if playable.is_empty() {
            if has_conclude {
                post_action(
                    client,
                    &serde_json::json!({"action_type": "EncounterConcludeEncounter"}),
                );
            } else if has_abort {
                post_action(
                    client,
                    &serde_json::json!({"action_type": "EncounterAbort"}),
                );
            }
            return rounds;
        }

        let action = strategy.choose_action(&playable, &snapshot);
        let (status, _) = post_action(client, &action);

        if status.code >= 400 {
            // Card was rejected (e.g., insufficient stamina). Blacklist it.
            if let Some(id) = action.get("card_id").and_then(|v| v.as_u64()) {
                unplayable_card_ids.push(id);
            }
            continue;
        }

        // Reset unplayable list after a successful play
        unplayable_card_ids.clear();
        rounds += 1;
    }

    rounds
}

/// Find a woodcutting encounter in the player's encounter hand.
pub fn find_woodcutting_encounter(client: &Client) -> Option<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .find(|c| {
            c.get("kind")
                .and_then(|k| k.get("encounter_kind"))
                .and_then(|ek| ek.get("encounter_type"))
                .and_then(|et| et.as_str())
                == Some("Woodcutting")
        })
        .and_then(|c| c.get("id").and_then(|v| v.as_u64()))
        .map(|v| v as usize)
}

/// Get IDs of all woodcutting encounter cards currently in the encounter hand.
pub fn get_woodcutting_encounter_ids(client: &Client) -> Vec<u64> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter(|c| {
            c.get("kind")
                .and_then(|k| k.get("encounter_kind"))
                .and_then(|ek| ek.get("encounter_type"))
                .and_then(|et| et.as_str())
                == Some("Woodcutting")
        })
        .filter_map(|c| c.get("id").and_then(|v| v.as_u64()))
        .collect()
}

/// Get woodcutting encounter choices, excluding encounters with IDs in `exclude_ids`.
pub fn get_woodcutting_encounter_choices_filtered(
    client: &Client,
    exclude_ids: &[u64],
) -> Vec<Value> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter(|c| {
            let id = c.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
            c.get("kind")
                .and_then(|k| k.get("encounter_kind"))
                .and_then(|ek| ek.get("encounter_type"))
                .and_then(|et| et.as_str())
                == Some("Woodcutting")
                && !exclude_ids.contains(&id)
        })
        .map(|c| {
            let card_id = c.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
            let max_plays = c
                .get("kind")
                .and_then(|k| k.get("encounter_kind"))
                .and_then(|ek| ek.get("woodcutting_def"))
                .and_then(|wd| wd.get("max_plays"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let base_rewards = c
                .get("kind")
                .and_then(|k| k.get("encounter_kind"))
                .and_then(|ek| ek.get("woodcutting_def"))
                .and_then(|wd| wd.get("base_rewards"))
                .cloned()
                .unwrap_or(Value::Null);
            serde_json::json!({
                "action_type": "EncounterPickEncounter",
                "card_id": card_id,
                "max_plays": max_plays,
                "base_rewards": base_rewards
            })
        })
        .collect()
}

/// Get playable woodcutting cards enriched with effect details for strategy decision-making.
pub fn get_playable_woodcutting_cards(client: &Client, snapshot: &GameSnapshot) -> Vec<Value> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Woodcutting");

    let played_cards = snapshot
        .encounter
        .as_ref()
        .and_then(|e| e.get("played_cards"))
        .cloned()
        .unwrap_or(Value::Null);

    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter(|c| {
            c.get("counts")
                .and_then(|co| co.get("hand"))
                .and_then(|h| h.as_u64())
                .unwrap_or(0)
                > 0
        })
        .map(|c| enrich_woodcutting_card(c, &played_cards))
        .collect()
}

fn enrich_woodcutting_card(c: &Value, played_cards: &Value) -> Value {
    let card_id = c.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
    let hand_count = c
        .get("counts")
        .and_then(|co| co.get("hand"))
        .and_then(|h| h.as_u64())
        .unwrap_or(0);
    let effects = c
        .get("kind")
        .and_then(|k| k.get("effects"))
        .cloned()
        .unwrap_or(Value::Null);

    let has_cost = effects
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|e| e.get("rolled_costs"))
        .and_then(|co| co.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false);

    let total_cost = effects
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .flat_map(|e| {
            e.get("rolled_costs")
                .and_then(|c| c.as_array())
                .cloned()
                .unwrap_or_default()
        })
        .filter_map(|cost| cost.get("amount").and_then(|v| v.as_i64()))
        .sum::<i64>();

    let rolled_value = effects
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|e| e.get("rolled_value"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let effect_id = effects
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|e| e.get("effect_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    serde_json::json!({
        "action_type": "EncounterPlayCard",
        "card_id": card_id,
        "card_kind": "Woodcutting",
        "card_details": {
            "effects": effects,
            "has_cost": has_cost,
            "total_cost": total_cost,
            "rolled_value": rolled_value,
            "effect_id": effect_id,
            "hand_count": hand_count,
            "played_cards": played_cards
        }
    })
}
