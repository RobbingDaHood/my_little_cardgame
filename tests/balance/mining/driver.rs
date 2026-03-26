use rocket::local::blocking::Client;
use serde_json::Value;

use crate::game_driver::{
    get_json, get_possible_actions, get_snapshot, post_action, DisciplineDriver, GameResult,
};
use crate::strategies::{GameSnapshot, Strategy};

/// Mining discipline driver — implements DisciplineDriver with yield/durability tracking.
pub struct MiningDisciplineDriver;

impl DisciplineDriver for MiningDisciplineDriver {
    fn get_encounter_ids(&self, client: &Client) -> Vec<u64> {
        get_mining_encounter_ids(client)
    }

    fn get_encounter_choices_filtered(&self, client: &Client, exclude_ids: &[u64]) -> Vec<Value> {
        get_mining_encounter_choices_filtered(client, exclude_ids)
    }

    fn find_encounter(&self, client: &Client) -> Option<usize> {
        find_mining_encounter(client)
    }

    fn play_encounter(&self, client: &Client, strategy: &dyn Strategy, max_actions: u32) -> u32 {
        play_mining_encounter(client, strategy, max_actions)
    }

    fn pre_encounter(&self, client: &Client) -> Option<Value> {
        let snapshot = get_snapshot(client);
        Some(serde_json::json!({
            "ore_before": snapshot.player_ore(),
            "durability_before": snapshot.mining_durability(),
        }))
    }

    fn post_encounter(&self, client: &Client, pre_state: &Option<Value>, result: &mut GameResult) {
        if let Some(pre) = pre_state {
            let snapshot = get_snapshot(client);
            let ore_before = pre.get("ore_before").and_then(|v| v.as_i64()).unwrap_or(0);
            let durability_before = pre
                .get("durability_before")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            let ore_after = snapshot.player_ore();
            let durability_after = snapshot.mining_durability();

            let ore_gained = ore_after - ore_before;
            let durability_spent = durability_before - durability_after;

            if ore_gained > 0 {
                result.yield_total += ore_gained;
            }
            if durability_spent > 0 {
                result.durability_spent += durability_spent;
            }
        }
    }
}

/// Play a mining encounter to completion using the given strategy.
/// Returns the number of rounds played.
///
/// Mining's `/actions/possible` returns a generic `EncounterPlayCard { card_id: 0 }`
/// placeholder, NOT individual card IDs like combat. We must query the mining hand
/// directly and handle 400 errors for cards the player can't afford.
pub fn play_mining_encounter(client: &Client, strategy: &dyn Strategy, max_actions: u32) -> u32 {
    let mut rounds = 0;
    let mut unplayable_card_ids: Vec<u64> = Vec::new();

    for _ in 0..max_actions {
        let snapshot = get_snapshot(client);

        // Check if mining encounter is still active
        if let Some(enc) = &snapshot.encounter {
            let state_type = enc
                .get("encounter_state_type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if state_type != "Mining" {
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

        if !can_play_card && !has_conclude {
            return rounds;
        }

        // Get mining cards in hand, excluding known-unplayable cards
        let mut playable: Vec<Value> = if can_play_card {
            get_playable_mining_cards(client, &snapshot)
                .into_iter()
                .filter(|c| {
                    let id = c.get("card_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    !unplayable_card_ids.contains(&id)
                })
                .collect()
        } else {
            vec![]
        };

        if has_conclude {
            playable.push(serde_json::json!({
                "action_type": "EncounterConcludeEncounter",
                "is_conclude": true
            }));
        }

        if playable.is_empty() {
            return rounds;
        }

        let action = strategy.choose_action(&playable, &snapshot);
        let (status, _) = post_action(client, &action);

        if status.code >= 400 {
            // Card was rejected (e.g., insufficient resources). Blacklist it.
            if let Some(id) = action.get("card_id").and_then(|v| v.as_u64()) {
                unplayable_card_ids.push(id);
            }
            // Don't count failed actions as rounds; retry immediately
            continue;
        }

        // Reset unplayable list after a successful play — state may have changed
        // allowing previously-unplayable cards to become playable again.
        unplayable_card_ids.clear();
        rounds += 1;
    }

    rounds
}

/// Find a mining encounter in the player's encounter hand.
pub fn find_mining_encounter(client: &Client) -> Option<usize> {
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
                == Some("Mining")
        })
        .and_then(|c| c.get("id").and_then(|v| v.as_u64()))
        .map(|v| v as usize)
}

/// Get IDs of all mining encounter cards currently in the encounter hand.
pub fn get_mining_encounter_ids(client: &Client) -> Vec<u64> {
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
                == Some("Mining")
        })
        .filter_map(|c| c.get("id").and_then(|v| v.as_u64()))
        .collect()
}

/// Get mining encounter choices, excluding encounters with IDs in `exclude_ids`.
pub fn get_mining_encounter_choices_filtered(client: &Client, exclude_ids: &[u64]) -> Vec<Value> {
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
                == Some("Mining")
                && !exclude_ids.contains(&id)
        })
        .map(|c| {
            let card_id = c.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
            let initial_light = c
                .get("kind")
                .and_then(|k| k.get("encounter_kind"))
                .and_then(|ek| ek.get("mining_def"))
                .and_then(|md| md.get("initial_light_level"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            serde_json::json!({
                "action_type": "EncounterPickEncounter",
                "card_id": card_id,
                "initial_light_level": initial_light
            })
        })
        .collect()
}

/// Get playable mining cards enriched with effect details for strategy decision-making.
pub fn get_playable_mining_cards(client: &Client, _snapshot: &GameSnapshot) -> Vec<Value> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Mining");

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
        .map(enrich_mining_card)
        .collect()
}

fn enrich_mining_card(c: &Value) -> Value {
    let card_id = c.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
    let effects = c
        .get("kind")
        .and_then(|k| k.get("effects"))
        .cloned()
        .unwrap_or(Value::Null);
    let has_cost = effects
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|e| e.get("rolled_costs"))
        .and_then(|c| c.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false);

    let mining_power = extract_effect_gain(&effects, "MiningPower");
    let light_gain = extract_effect_gain(&effects, "MiningLightLevel");

    serde_json::json!({
        "action_type": "EncounterPlayCard",
        "card_id": card_id,
        "card_kind": "Mining",
        "card_details": {
            "effects": effects,
            "has_cost": has_cost,
            "mining_power": mining_power,
            "light_gain": light_gain
        }
    })
}

/// Extract the gain amount for a specific token type from a card's effects array.
fn extract_effect_gain(effects: &Value, token_type: &str) -> i64 {
    effects
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter(|e| {
            e.get("effect_type")
                .and_then(|v| v.as_str())
                .map(|s| s == "ApplyTokens" || s == "GainTokens")
                .unwrap_or(false)
        })
        .filter_map(|e| {
            // Check rolled_amounts for the token type
            e.get("rolled_amounts")
                .and_then(|ra| ra.as_object())
                .and_then(|map| map.get(token_type))
                .and_then(|v| v.as_i64())
                .or_else(|| {
                    // Fallback: check amounts field
                    e.get("amounts")
                        .and_then(|a| a.as_object())
                        .and_then(|map| map.get(token_type))
                        .and_then(|v| v.as_i64())
                })
        })
        .sum()
}
