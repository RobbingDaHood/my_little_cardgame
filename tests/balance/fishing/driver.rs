use std::collections::HashMap;

use rocket::local::blocking::Client;
use serde_json::Value;

use crate::game_driver::{
    get_json, get_possible_actions, get_snapshot, post_action, DisciplineDriver, GameResult,
};
use crate::strategies::{GameSnapshot, Strategy};

/// Maps effect_id -> (effect_type, optional token_type for GainTokens effects).
fn build_effect_type_map(client: &Client) -> HashMap<u64, (String, Option<String>)> {
    let data = get_json(client, "/library/card-effects");
    let mut map = HashMap::new();

    for section_key in &["player_effects", "enemy_effects"] {
        if let Some(effects) = data.get(section_key).and_then(|v| v.as_array()) {
            for entry in effects {
                let id = entry.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                let kind_obj = entry
                    .get("card")
                    .and_then(|c| c.get("kind"))
                    .and_then(|k| k.get("kind"));

                if let Some(kind) = kind_obj {
                    let effect_type = kind
                        .get("effect_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let token_type = kind
                        .get("token_type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    map.insert(id, (effect_type, token_type));
                }
            }
        }
    }

    map
}

pub struct FishingDisciplineDriver;

impl DisciplineDriver for FishingDisciplineDriver {
    fn get_encounter_ids(&self, client: &Client) -> Vec<u64> {
        get_fishing_encounter_ids(client)
    }

    fn get_encounter_choices_filtered(&self, client: &Client, exclude_ids: &[u64]) -> Vec<Value> {
        get_fishing_encounter_choices_filtered(client, exclude_ids)
    }

    fn find_encounter(&self, client: &Client) -> Option<usize> {
        find_fishing_encounter(client)
    }

    fn play_encounter(&self, client: &Client, strategy: &dyn Strategy, max_actions: u32) -> u32 {
        play_fishing_encounter(client, strategy, max_actions)
    }

    fn pre_encounter(&self, client: &Client) -> Option<Value> {
        let snapshot = get_snapshot(client);
        Some(serde_json::json!({
            "fish_before": snapshot.fish_tokens(),
            "durability_before": snapshot.fishing_durability(),
        }))
    }

    fn post_encounter(&self, client: &Client, pre_state: &Option<Value>, result: &mut GameResult) {
        if let Some(pre) = pre_state {
            let snapshot = get_snapshot(client);
            let fish_before = pre.get("fish_before").and_then(|v| v.as_i64()).unwrap_or(0);
            let durability_before = pre
                .get("durability_before")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            let fish_after = snapshot.fish_tokens();
            let durability_after = snapshot.fishing_durability();

            let fish_gained = fish_after - fish_before;
            let durability_spent = durability_before - durability_after;

            if fish_gained > 0 {
                result.yield_total += fish_gained;
            }
            if durability_spent > 0 {
                result.durability_spent += durability_spent;
            }
        }
    }
}

/// Play a fishing encounter to completion using the given strategy.
/// Returns the number of rounds played.
///
/// Conclude is NOT offered to strategies as a normal action (it creates
/// degenerate "free-win" behaviour when chosen voluntarily).  Instead,
/// conclude is used automatically as a fallback when every card in hand
/// has been blacklisted (unaffordable).
pub fn play_fishing_encounter(client: &Client, strategy: &dyn Strategy, max_actions: u32) -> u32 {
    let mut rounds = 0;
    let mut unplayable_card_ids: Vec<u64> = Vec::new();

    for _ in 0..max_actions {
        let snapshot = get_snapshot(client);

        // Check if fishing encounter is still active
        if let Some(enc) = &snapshot.encounter {
            let state_type = enc
                .get("encounter_state_type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if state_type != "Fishing" {
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

        // Build card-only action list for the strategy (no conclude)
        let card_actions: Vec<Value> = if can_play_card {
            get_playable_fishing_cards(client, &snapshot)
                .into_iter()
                .filter(|c| {
                    let id = c.get("card_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    !unplayable_card_ids.contains(&id)
                })
                .collect()
        } else {
            vec![]
        };

        // Fallback: all cards blacklisted → auto-conclude if available
        if card_actions.is_empty() {
            if has_conclude {
                let conclude = serde_json::json!({
                    "action_type": "EncounterConcludeEncounter"
                });
                post_action(client, &conclude);
                rounds += 1;
            }
            return rounds;
        }

        let action = strategy.choose_action(&card_actions, &snapshot);
        let (status, _) = post_action(client, &action);

        if status.code >= 400 {
            if let Some(id) = action.get("card_id").and_then(|v| v.as_u64()) {
                unplayable_card_ids.push(id);
            }
            continue;
        }

        unplayable_card_ids.clear();
        rounds += 1;
    }

    rounds
}

/// Find a fishing encounter in the player's encounter hand.
pub fn find_fishing_encounter(client: &Client) -> Option<usize> {
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
                == Some("Fishing")
        })
        .and_then(|c| c.get("id").and_then(|v| v.as_u64()))
        .map(|v| v as usize)
}

/// Get IDs of all fishing encounter cards currently in the encounter hand.
pub fn get_fishing_encounter_ids(client: &Client) -> Vec<u64> {
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
                == Some("Fishing")
        })
        .filter_map(|c| c.get("id").and_then(|v| v.as_u64()))
        .collect()
}

/// Get fishing encounter choices, excluding encounters with IDs in `exclude_ids`.
/// Includes `fish_reward` so strategies can pick encounters by reward value.
pub fn get_fishing_encounter_choices_filtered(client: &Client, exclude_ids: &[u64]) -> Vec<Value> {
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
                == Some("Fishing")
                && !exclude_ids.contains(&id)
        })
        .map(|c| {
            let card_id = c.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
            let fishing_def = c
                .get("kind")
                .and_then(|k| k.get("encounter_kind"))
                .and_then(|ek| ek.get("fishing_def"));
            let wins_required = fishing_def
                .and_then(|fd| fd.get("wins_required"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let fish_reward = fishing_def
                .and_then(|fd| fd.get("rewards"))
                .and_then(|r| r.get("Fish"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            serde_json::json!({
                "action_type": "EncounterPickEncounter",
                "card_id": card_id,
                "wins_required": wins_required,
                "fish_reward": fish_reward
            })
        })
        .collect()
}

/// Get playable fishing cards enriched with effect details for strategy decision-making.
pub fn get_playable_fishing_cards(client: &Client, _snapshot: &GameSnapshot) -> Vec<Value> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Fishing");
    let effect_type_map = build_effect_type_map(client);

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

            let mut max_fishing_value: i64 = 0;
            let mut num_fishing_values: i64 = 0;
            let mut total_durability_cost: i64 = 0;
            let mut has_range_modifier = false;
            let mut has_fish_amount_modifier = false;

            if let Some(effects_arr) = effects.as_array() {
                for effect in effects_arr {
                    let effect_id = effect
                        .get("effect_id")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);

                    if let Some((effect_type, token_type)) = effect_type_map.get(&effect_id) {
                        match effect_type.as_str() {
                            "FishingValue" => {
                                num_fishing_values += 1;
                                let rolled = effect
                                    .get("rolled_value")
                                    .and_then(|v| v.as_i64())
                                    .unwrap_or(0);
                                if rolled > max_fishing_value {
                                    max_fishing_value = rolled;
                                }
                            }
                            "GainTokens" => match token_type.as_deref() {
                                Some("FishingRangeMin") | Some("FishingRangeMax") => {
                                    has_range_modifier = true;
                                }
                                Some("FishAmount") => {
                                    has_fish_amount_modifier = true;
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                    }

                    if let Some(costs) = effect.get("rolled_costs").and_then(|c| c.as_array()) {
                        for cost in costs {
                            let is_durability = cost
                                .get("token_type")
                                .and_then(|tt| tt.as_str())
                                .map(|s| s == "Durability" || s == "FishingDurability")
                                .unwrap_or(false);
                            if is_durability {
                                let amount =
                                    cost.get("amount").and_then(|a| a.as_i64()).unwrap_or(0);
                                total_durability_cost += amount;
                            }
                        }
                    }
                }
            }

            serde_json::json!({
                "action_type": "EncounterPlayCard",
                "card_id": card_id,
                "card_details": {
                    "effects": effects,
                    "max_fishing_value": max_fishing_value,
                    "num_fishing_values": num_fishing_values,
                    "total_durability_cost": total_durability_cost,
                    "has_range_modifier": has_range_modifier,
                    "has_fish_amount_modifier": has_fish_amount_modifier,
                }
            })
        })
        .collect()
}
