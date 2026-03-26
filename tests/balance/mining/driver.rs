use std::collections::HashMap;

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
///
/// Key edge cases:
/// - Cards with cross-discipline costs (e.g., Lumber) are permanently unplayable in
///   a mining-only simulation. They get blacklisted on first 400 error and stay blacklisted.
/// - Conclude fails when yield=0 ("No yield accumulated; abort instead").
///   In that case we abort to exit the encounter cleanly.
pub fn play_mining_encounter(client: &Client, strategy: &dyn Strategy, max_actions: u32) -> u32 {
    let mut rounds = 0;
    let mut unplayable_card_ids: Vec<u64> = Vec::new();
    let effect_map = load_effect_token_map(client);
    // Track cards that fail due to resource costs — these stay blacklisted even after
    // a successful play, since the missing resource won't appear mid-encounter.
    let mut permanently_unplayable: Vec<u64> = Vec::new();

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
        let action_types: Vec<String> = possible
            .iter()
            .filter_map(|a| {
                a.get("action_type")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .collect();
        let can_play_card = action_types.contains(&"EncounterPlayCard".to_string());
        let has_conclude = action_types.contains(&"EncounterConcludeEncounter".to_string());
        let has_abort = action_types.contains(&"EncounterAbort".to_string());

        if !can_play_card && !has_conclude && !has_abort {
            return rounds;
        }

        // Get mining cards in hand, excluding unplayable cards
        let all_blacklisted: Vec<u64> = unplayable_card_ids
            .iter()
            .chain(permanently_unplayable.iter())
            .copied()
            .collect();
        let mut playable: Vec<Value> = if can_play_card {
            get_playable_mining_cards(client, &snapshot, &effect_map)
                .into_iter()
                .filter(|c| {
                    let id = c.get("card_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    !all_blacklisted.contains(&id)
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

        // No playable cards and no conclude — abort the encounter
        if playable.is_empty() {
            if has_abort {
                post_action(
                    client,
                    &serde_json::json!({"action_type": "EncounterAbort"}),
                );
            }
            return rounds;
        }

        let action = strategy.choose_action(&playable, &snapshot);
        let is_conclude = action
            .get("is_conclude")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Strip non-API fields before posting
        let api_action = if is_conclude {
            serde_json::json!({"action_type": "EncounterConcludeEncounter"})
        } else {
            serde_json::json!({
                "action_type": action.get("action_type").and_then(|v| v.as_str()).unwrap_or(""),
                "card_id": action.get("card_id").and_then(|v| v.as_u64()).unwrap_or(0)
            })
        };
        let (status, _) = post_action(client, &api_action);

        if status.code >= 400 {
            if is_conclude {
                // Conclude rejected (e.g., "No yield accumulated") — abort instead
                if has_abort {
                    post_action(
                        client,
                        &serde_json::json!({"action_type": "EncounterAbort"}),
                    );
                }
                return rounds;
            }
            // Card play rejected — blacklist it. Resource-cost failures are permanent.
            if let Some(id) = action.get("card_id").and_then(|v| v.as_u64()) {
                permanently_unplayable.push(id);
            }
            continue;
        }

        if is_conclude {
            return rounds;
        }

        // Successful card play — reset temporary blacklist (state changed),
        // but keep permanent blacklist (resource costs don't change mid-encounter).
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

/// Load effect templates and build a mapping from effect_id → token_type string.
/// This maps each effect definition to the token it affects (e.g., "MiningPower", "MiningLightLevel").
fn load_effect_token_map(client: &Client) -> HashMap<u64, String> {
    let effects = get_json(client, "/library/card-effects");
    let mut map = HashMap::new();

    for key in ["player_effects", "enemy_effects"] {
        if let Some(arr) = effects.get(key).and_then(|v| v.as_array()) {
            for entry in arr {
                let id = entry.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                // CardKind uses internally-tagged serde: { "card_kind": "PlayerCardEffect", "kind": { "effect_type": "GainTokens", "token_type": "MiningPower" } }
                // Navigate: entry.card.kind.kind.token_type
                if let Some(token_type) = entry
                    .get("card")
                    .and_then(|c| c.get("kind"))
                    .and_then(|k| k.get("kind"))
                    .and_then(|ik| ik.get("token_type"))
                    .and_then(|v| v.as_str())
                {
                    map.insert(id, token_type.to_string());
                }
            }
        }
    }
    map
}

/// Get playable mining cards enriched with effect details for strategy decision-making.
pub fn get_playable_mining_cards(
    client: &Client,
    _snapshot: &GameSnapshot,
    effect_map: &HashMap<u64, String>,
) -> Vec<Value> {
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
        .map(|c| enrich_mining_card(c, effect_map))
        .collect()
}

fn enrich_mining_card(c: &Value, effect_map: &HashMap<u64, String>) -> Value {
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

    // Use effect_id → token_type mapping to extract mining_power and light_gain
    let mut mining_power: i64 = 0;
    let mut light_gain: i64 = 0;

    if let Some(arr) = effects.as_array() {
        for eff in arr {
            let eff_id = eff.get("effect_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let rolled_value = eff
                .get("rolled_value")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            if let Some(token_type) = effect_map.get(&eff_id) {
                match token_type.as_str() {
                    "MiningPower" => mining_power += rolled_value,
                    "MiningLightLevel" => light_gain += rolled_value,
                    _ => {}
                }
            }
        }
    }

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
