use rocket::local::blocking::Client;
use serde::Serialize;
use serde_json::Value;

use crate::game_driver::{
    get_encounter_results_count, get_json, get_last_encounter_outcome, get_possible_actions,
    get_snapshot, post_action,
};
use crate::strategies::Strategy;

/// Result from playing a single fishing encounter.
#[derive(Debug, Clone, Serialize)]
pub struct FishingEncounterResult {
    pub rounds_played: u32,
    pub won: bool,
}

/// Play a fishing encounter to completion using the given strategy.
/// Detects outcome via /encounter/results since fishing auto-completes
/// (encounter state becomes None after win/loss).
pub fn play_fishing_encounter(
    client: &Client,
    strategy: &dyn Strategy,
    max_actions: u32,
) -> FishingEncounterResult {
    let mut rounds = 0;
    let results_before = get_encounter_results_count(client);

    for _ in 0..max_actions {
        let snapshot = get_snapshot(client);

        // Encounter auto-completed — check results to determine outcome
        if snapshot.encounter_state_type().is_none() {
            let won = get_last_encounter_outcome(client, results_before)
                .map(|o| o == "PlayerWon")
                .unwrap_or(false);
            return FishingEncounterResult {
                rounds_played: rounds,
                won,
            };
        }

        // Still in encounter — check if we can play cards
        let possible = get_possible_actions(client);
        let action_types: Vec<String> = possible
            .iter()
            .filter_map(|a| {
                a.get("action_type")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .collect();

        // Need to conclude/finish (and can't play cards)
        if !action_types.contains(&"EncounterPlayCard".to_string())
            && (action_types.contains(&"EncounterConcludeEncounter".to_string())
                || action_types.contains(&"EncounterFinishEncounter".to_string()))
        {
            let won = get_last_encounter_outcome(client, results_before)
                .map(|o| o == "PlayerWon")
                .unwrap_or(false);
            return FishingEncounterResult {
                rounds_played: rounds,
                won,
            };
        }

        if !action_types.contains(&"EncounterPlayCard".to_string()) {
            let won = get_last_encounter_outcome(client, results_before)
                .map(|o| o == "PlayerWon")
                .unwrap_or(false);
            return FishingEncounterResult {
                rounds_played: rounds,
                won,
            };
        }

        let playable = get_playable_fishing_cards(client);
        if playable.is_empty() {
            let won = get_last_encounter_outcome(client, results_before)
                .map(|o| o == "PlayerWon")
                .unwrap_or(false);
            return FishingEncounterResult {
                rounds_played: rounds,
                won,
            };
        }

        let action = strategy.choose_action(&playable, &snapshot);
        post_action(client, &action);
        rounds += 1;
    }

    let won = get_last_encounter_outcome(client, results_before)
        .map(|o| o == "PlayerWon")
        .unwrap_or(false);
    FishingEncounterResult {
        rounds_played: rounds,
        won,
    }
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

/// Get playable fishing cards from hand with metadata for strategy decision-making.
pub fn get_playable_fishing_cards(client: &Client) -> Vec<Value> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Fishing");

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

            // Extract fishing-relevant metadata from effects
            let mut max_fishing_value: i64 = 0;
            let mut total_durability_cost: i64 = 0;
            let mut has_range_modifier = false;
            let mut has_fish_amount_modifier = false;

            if let Some(effects_arr) = effects.as_array() {
                for effect in effects_arr {
                    let effect_kind = effect
                        .get("kind")
                        .and_then(|k| k.as_object())
                        .and_then(|obj| obj.keys().next().cloned());

                    match effect_kind.as_deref() {
                        Some("FishingValue") => {
                            let rolled = effect
                                .get("rolled_value")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0);
                            if rolled > max_fishing_value {
                                max_fishing_value = rolled;
                            }
                        }
                        Some("GainTokens") => {
                            let token_type = effect
                                .get("kind")
                                .and_then(|k| k.get("GainTokens"))
                                .and_then(|gt| gt.get("token_type"))
                                .and_then(|tt| tt.as_str());
                            match token_type {
                                Some("FishingRangeMin") | Some("FishingRangeMax") => {
                                    has_range_modifier = true;
                                }
                                Some("FishAmount") => {
                                    has_fish_amount_modifier = true;
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }

                    // Check durability costs
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
                    "total_durability_cost": total_durability_cost,
                    "has_range_modifier": has_range_modifier,
                    "has_fish_amount_modifier": has_fish_amount_modifier,
                }
            })
        })
        .collect()
}
