use rocket::local::blocking::Client;
use serde_json::Value;

use crate::game_driver::{get_json, get_possible_actions, get_snapshot, post_action};
use crate::strategies::{GameSnapshot, Strategy};

/// Play a combat encounter to completion using the given strategy.
/// Returns the number of rounds played.
pub fn play_combat_encounter(client: &Client, strategy: &dyn Strategy, max_actions: u32) -> u32 {
    let mut rounds = 0;

    for _ in 0..max_actions {
        let snapshot = get_snapshot(client);

        if let Some(outcome) = snapshot.combat_outcome() {
            if outcome != "Undecided" {
                return rounds;
            }
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
            return rounds;
        }

        let playable = get_playable_combat_cards(client, &snapshot);
        if playable.is_empty() {
            return rounds;
        }

        let action = strategy.choose_action(&playable, &snapshot);
        post_action(client, &action);
        rounds += 1;
    }

    rounds
}

/// Find a combat encounter in the player's encounter hand.
/// Returns the card ID if found.
pub fn find_combat_encounter(client: &Client) -> Option<usize> {
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
                == Some("Combat")
        })
        .and_then(|c| c.get("id").and_then(|v| v.as_u64()))
        .map(|v| v as usize)
}

/// Get playable combat cards enriched with details for strategy decision-making.
/// Returns action-ready JSON values with card_id and card metadata.
pub fn get_playable_combat_cards(client: &Client, snapshot: &GameSnapshot) -> Vec<Value> {
    let phase = snapshot.combat_phase().unwrap_or_default();
    let card_kind = match phase.as_str() {
        "Defending" => "Defence",
        "Attacking" => "Attack",
        "Resourcing" => "Resource",
        _ => return vec![],
    };

    let url = format!("/library/cards?location=Hand&card_kind={}", card_kind);
    let cards = get_json(client, &url);

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
            let has_cost = effects
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|e| e.get("rolled_costs"))
                .and_then(|c| c.as_array())
                .map(|a| !a.is_empty())
                .unwrap_or(false);
            serde_json::json!({
                "action_type": "EncounterPlayCard",
                "card_id": card_id,
                "card_kind": card_kind,
                "card_details": {
                    "effects": effects,
                    "has_cost": has_cost
                }
            })
        })
        .collect()
}
