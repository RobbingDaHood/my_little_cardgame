use rocket::local::blocking::Client;
use serde::Serialize;

use crate::game_driver::{
    advance_to_encounter_pick, get_possible_actions, get_snapshot, post_action,
};
use crate::strategies::Strategy;

use super::driver::{find_fishing_encounter, play_fishing_encounter, FishingEncounterResult};

#[derive(Debug, Clone, Serialize)]
pub struct FishingGameResult {
    pub seed: u64,
    pub fishing_wins: u32,
    pub fishing_losses: u32,
    pub total_fishing_encounters: u32,
    pub initial_durability: i64,
    pub remaining_durability: i64,
    pub total_durability_spent: i64,
    pub total_fish_earned: i64,
    pub yield_per_durability: f64,
    pub encounter_results: Vec<FishingEncounterResult>,
}

pub struct FishingGameDriver {
    pub max_encounters: u32,
    pub max_actions_per_encounter: u32,
}

impl FishingGameDriver {
    pub fn play_game(
        &self,
        client: &Client,
        seed: u64,
        strategy: &dyn Strategy,
    ) -> FishingGameResult {
        // Start new game
        post_action(
            client,
            &serde_json::json!({"action_type": "NewGame", "seed": seed}),
        );

        let initial_snapshot = get_snapshot(client);
        let initial_durability = initial_snapshot.fishing_durability();
        let initial_fish = initial_snapshot.fish_tokens();

        let mut encounter_results = Vec::new();
        let mut fishing_wins: u32 = 0;
        let mut fishing_losses: u32 = 0;
        let mut encounters_played: u32 = 0;

        for _ in 0..self.max_encounters {
            if !advance_to_encounter_pick(client) {
                break;
            }

            // Look for a fishing encounter
            if let Some(encounter_id) = find_fishing_encounter(client) {
                post_action(
                    client,
                    &serde_json::json!({
                        "action_type": "EncounterPickEncounter",
                        "card_id": encounter_id
                    }),
                );

                let result =
                    play_fishing_encounter(client, strategy, self.max_actions_per_encounter);
                if result.won {
                    fishing_wins += 1;
                } else {
                    fishing_losses += 1;
                }
                encounters_played += 1;
                encounter_results.push(result);

                // Conclude the encounter
                conclude_encounter(client);
            } else {
                // No fishing encounter available — pick first available and abort/skip
                pick_and_skip_non_fishing(client);
            }

            // Check if game is over (player dead / no durability)
            let snapshot = get_snapshot(client);
            if snapshot.fishing_durability() <= 0 {
                break;
            }
        }

        let final_snapshot = get_snapshot(client);
        let remaining_durability = final_snapshot.fishing_durability();
        let total_fish_earned = final_snapshot.fish_tokens() - initial_fish;
        let total_durability_spent = initial_durability - remaining_durability;
        let yield_per_durability = if total_durability_spent > 0 {
            total_fish_earned as f64 / total_durability_spent as f64
        } else {
            0.0
        };

        FishingGameResult {
            seed,
            fishing_wins,
            fishing_losses,
            total_fishing_encounters: encounters_played,
            initial_durability,
            remaining_durability,
            total_durability_spent,
            total_fish_earned,
            yield_per_durability,
            encounter_results,
        }
    }
}

fn conclude_encounter(client: &Client) {
    for _ in 0..20 {
        let possible = get_possible_actions(client);
        let action_types: Vec<String> = possible
            .iter()
            .filter_map(|a| {
                a.get("action_type")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .collect();

        if action_types.contains(&"EncounterConcludeEncounter".to_string()) {
            post_action(
                client,
                &serde_json::json!({"action_type": "EncounterConcludeEncounter"}),
            );
            continue;
        }

        if action_types.contains(&"EncounterFinishEncounter".to_string()) {
            post_action(
                client,
                &serde_json::json!({"action_type": "EncounterFinishEncounter"}),
            );
            continue;
        }

        // Done concluding
        break;
    }
}

fn pick_and_skip_non_fishing(client: &Client) {
    let cards =
        crate::game_driver::get_json(client, "/library/cards?location=Hand&card_kind=Encounter");

    let first_id = cards
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|c| c.get("id"))
        .and_then(|v| v.as_u64());

    if let Some(id) = first_id {
        post_action(
            client,
            &serde_json::json!({
                "action_type": "EncounterPickEncounter",
                "card_id": id
            }),
        );
        abort_encounter(client);
    }
}

fn abort_encounter(client: &Client) {
    for _ in 0..20 {
        let possible = get_possible_actions(client);
        let action_types: Vec<String> = possible
            .iter()
            .filter_map(|a| {
                a.get("action_type")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .collect();

        if action_types.contains(&"EncounterAbort".to_string()) {
            post_action(
                client,
                &serde_json::json!({"action_type": "EncounterAbort"}),
            );
            continue;
        }

        if action_types.contains(&"EncounterConcludeEncounter".to_string()) {
            post_action(
                client,
                &serde_json::json!({"action_type": "EncounterConcludeEncounter"}),
            );
            continue;
        }

        if action_types.contains(&"EncounterFinishEncounter".to_string()) {
            post_action(
                client,
                &serde_json::json!({"action_type": "EncounterFinishEncounter"}),
            );
            continue;
        }

        break;
    }
}
