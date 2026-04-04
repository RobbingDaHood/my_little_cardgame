use rocket::local::blocking::Client;
use serde_json::Value;
use std::collections::HashMap;

use crate::game_driver::{
    get_json, get_possible_actions, get_snapshot, post_action, DisciplineDriver, GameResult,
};
use crate::strategies::Strategy;

/// Herbalism discipline driver — implements DisciplineDriver with yield/durability tracking.
pub struct HerbalismDisciplineDriver;

impl DisciplineDriver for HerbalismDisciplineDriver {
    fn get_encounter_ids(&self, client: &Client) -> Vec<u64> {
        get_herbalism_encounter_ids(client)
    }

    fn get_encounter_choices_filtered(&self, client: &Client, exclude_ids: &[u64]) -> Vec<Value> {
        get_herbalism_encounter_choices_filtered(client, exclude_ids)
    }

    fn find_encounter(&self, client: &Client) -> Option<usize> {
        find_herbalism_encounter(client)
    }

    fn play_encounter(&self, client: &Client, strategy: &dyn Strategy, max_actions: u32) -> u32 {
        play_herbalism_encounter(client, strategy, max_actions)
    }

    fn pre_encounter(&self, client: &Client) -> Option<Value> {
        let snapshot = get_snapshot(client);
        Some(serde_json::json!({
            "plant_before": snapshot.plant_tokens(),
            "durability_before": snapshot.herbalism_durability(),
        }))
    }

    fn post_encounter(&self, client: &Client, pre_state: &Option<Value>, result: &mut GameResult) {
        if let Some(pre) = pre_state {
            let snapshot = get_snapshot(client);
            let plant_before = pre
                .get("plant_before")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let durability_before = pre
                .get("durability_before")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            let plant_after = snapshot.plant_tokens();
            let durability_after = snapshot.herbalism_durability();

            let plant_gained = plant_after - plant_before;
            let durability_spent = durability_before - durability_after;

            if plant_gained > 0 {
                result.yield_total += plant_gained;
            }
            if durability_spent > 0 {
                result.durability_spent += durability_spent;
            }
        }
    }
}

/// Play an herbalism encounter to completion using the given strategy.
/// Returns the number of rounds played.
pub fn play_herbalism_encounter(client: &Client, strategy: &dyn Strategy, max_actions: u32) -> u32 {
    let effect_map = build_effect_map(client);
    let mut rounds = 0;

    for action_num in 0..max_actions {
        let snapshot = get_snapshot(client);

        if let Some(outcome) = snapshot.herbalism_outcome() {
            if outcome != "Undecided" {
                let possible = get_possible_actions(client);
                if possible.iter().any(|a| {
                    a.get("action_type").and_then(|v| v.as_str())
                        == Some("EncounterConcludeEncounter")
                }) {
                    post_action(
                        client,
                        &serde_json::json!({"action_type": "EncounterConcludeEncounter"}),
                    );
                }
                return rounds;
            }
        } else if action_num > 0 {
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

        if !action_types.contains(&"EncounterPlayCard".to_string()) {
            if action_types.contains(&"EncounterAbort".to_string()) {
                post_action(
                    client,
                    &serde_json::json!({"action_type": "EncounterAbort"}),
                );
            } else if action_types.contains(&"EncounterConcludeEncounter".to_string()) {
                post_action(
                    client,
                    &serde_json::json!({"action_type": "EncounterConcludeEncounter"}),
                );
            }
            return rounds;
        }

        let playable = get_playable_herbalism_cards(client, &effect_map);

        if playable.is_empty() {
            if possible
                .iter()
                .any(|a| a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterAbort"))
            {
                post_action(
                    client,
                    &serde_json::json!({"action_type": "EncounterAbort"}),
                );
            }
            return rounds;
        }

        let action = strategy.choose_action(&playable, &snapshot);
        post_action(client, &action);
        rounds += 1;
    }

    rounds
}

/// Find an herbalism encounter in the player's encounter hand.
pub fn find_herbalism_encounter(client: &Client) -> Option<usize> {
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
                == Some("Herbalism")
        })
        .and_then(|c| c.get("id").and_then(|v| v.as_u64()))
        .map(|v| v as usize)
}

/// Get IDs of all herbalism encounter cards currently in the encounter hand.
pub fn get_herbalism_encounter_ids(client: &Client) -> Vec<u64> {
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
                == Some("Herbalism")
        })
        .filter_map(|c| c.get("id").and_then(|v| v.as_u64()))
        .collect()
}

/// Get herbalism encounter choices, excluding encounters with IDs in `exclude_ids`.
pub fn get_herbalism_encounter_choices_filtered(
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
                == Some("Herbalism")
                && !exclude_ids.contains(&id)
        })
        .map(|c| {
            let card_id = c.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
            serde_json::json!({
                "action_type": "EncounterPickEncounter",
                "card_id": card_id,
            })
        })
        .collect()
}

/// Get playable herbalism cards enriched with match mode info for strategy decisions.
fn get_playable_herbalism_cards(client: &Client, effect_map: &HashMap<usize, Value>) -> Vec<Value> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Herbalism");

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
        .map(|c| {
            let card_id = c.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
            let effects = c
                .get("kind")
                .and_then(|k| k.get("effects"))
                .cloned()
                .unwrap_or(Value::Null);

            let has_extra_cost = effects
                .as_array()
                .map(|arr| {
                    arr.iter().any(|e| {
                        e.get("rolled_costs")
                            .and_then(|c| c.as_array())
                            .map(|costs| {
                                costs.iter().any(|cost| {
                                    let tt = cost
                                        .get("token_type")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");
                                    tt != "HerbalismDurability" && tt != "Durability"
                                })
                            })
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);

            let match_info = effects
                .as_array()
                .and_then(|arr| {
                    arr.iter().find_map(|e| {
                        let eid = e.get("effect_id").and_then(|v| v.as_u64())? as usize;
                        let template = effect_map.get(&eid)?;
                        // API nests as card.kind.kind.effect_type (outer kind has card_kind)
                        let outer_kind = template.get("kind")?;
                        let inner_kind = outer_kind.get("kind")?;
                        let effect_type = inner_kind.get("effect_type").and_then(|v| v.as_str())?;
                        if effect_type == "HerbalismMatch" {
                            inner_kind.get("match_mode").cloned()
                        } else {
                            None
                        }
                    })
                })
                .unwrap_or(Value::Null);

            serde_json::json!({
                "action_type": "EncounterPlayCard",
                "card_id": card_id,
                "card_details": {
                    "effects": effects,
                    "has_cost": has_extra_cost,
                    "match_info": match_info
                }
            })
        })
        .collect()
}

/// Build a map of effect_id → effect template card from /library/card-effects.
fn build_effect_map(client: &Client) -> HashMap<usize, Value> {
    let effects = get_json(client, "/library/card-effects");
    let mut map = HashMap::new();

    for key in &["player_effects", "enemy_effects"] {
        if let Some(arr) = effects.get(*key).and_then(|v| v.as_array()) {
            for entry in arr {
                if let Some(id) = entry.get("id").and_then(|v| v.as_u64()) {
                    if let Some(card) = entry.get("card") {
                        map.insert(id as usize, card.clone());
                    }
                }
            }
        }
    }

    map
}
