use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;
use serde_json::Value;

use crate::strategies::{GameSnapshot, Strategy};

/// Results from a single game session.
#[derive(Debug, Clone)]
pub struct GameResult {
    pub seed: u64,
    pub combat_wins: u32,
    pub combat_losses: u32,
    pub total_encounters: u32,
    pub deaths: i64,
    pub final_health: i64,
    pub ended_by_death: bool,
    pub rounds_per_encounter: Vec<u32>,
}

/// Drives one full game session through repeated combat encounters.
pub struct GameDriver {
    max_encounters: u32,
    max_actions_per_encounter: u32,
}

impl GameDriver {
    pub fn new(max_encounters: u32) -> Self {
        Self {
            max_encounters,
            max_actions_per_encounter: 200,
        }
    }

    pub fn play_game(&self, seed: u64, strategy: &dyn Strategy) -> GameResult {
        let client =
            Client::tracked(my_little_cardgame::rocket_initialize()).expect("valid rocket");

        let new_game = serde_json::json!({"action_type": "NewGame", "seed": seed});
        post_action(&client, &new_game);

        let mut result = GameResult {
            seed,
            combat_wins: 0,
            combat_losses: 0,
            total_encounters: 0,
            deaths: 0,
            final_health: 0,
            ended_by_death: false,
            rounds_per_encounter: Vec::new(),
        };

        let initial_deaths = get_snapshot(&client).player_deaths();

        for _ in 0..self.max_encounters {
            // Drive the game to the next encounter pick by handling conclude/scout first
            if !self.advance_to_encounter_pick(&client, strategy) {
                break;
            }

            // Pick a combat encounter
            let combat_enc_id = find_combat_encounter(&client);
            let combat_enc_id = match combat_enc_id {
                Some(id) => id,
                None => {
                    // No combat encounters — pick any encounter and skip
                    let enc_ids = get_encounter_hand_ids(&client);
                    if enc_ids.is_empty() {
                        break;
                    }
                    let pick = serde_json::json!({
                        "action_type": "EncounterPickEncounter",
                        "card_id": enc_ids[0]
                    });
                    post_action(&client, &pick);
                    self.play_non_combat_encounter(&client, strategy);
                    continue;
                }
            };

            let pick = serde_json::json!({
                "action_type": "EncounterPickEncounter",
                "card_id": combat_enc_id
            });
            post_action(&client, &pick);
            result.total_encounters += 1;

            // Play combat cards until resolved
            let results_before = get_encounter_results_count(&client);
            let rounds = self.play_combat_encounter(&client, strategy);
            result.rounds_per_encounter.push(rounds);

            // Read outcome from /encounter/results (combat auto-concludes)
            let outcome = get_last_encounter_outcome(&client, results_before);
            match outcome.as_deref() {
                Some("PlayerWon") => result.combat_wins += 1,
                Some("PlayerLost") => result.combat_losses += 1,
                _ => {}
            }

            // Check for death
            let snapshot = get_snapshot(&client);
            let current_deaths = snapshot.player_deaths();
            if current_deaths > initial_deaths + result.deaths {
                result.deaths = current_deaths - initial_deaths;
                result.ended_by_death = true;
            }
        }

        result.final_health = get_snapshot(&client).player_health();
        result
    }

    /// Advance game state until EncounterPickEncounter is available.
    /// Handles conclude and scouting phases. Returns false if stuck or no encounters.
    fn advance_to_encounter_pick(&self, client: &Client, _strategy: &dyn Strategy) -> bool {
        for _ in 0..50 {
            let possible = get_possible_actions(client);
            let action_types: Vec<String> = possible
                .iter()
                .filter_map(|a| {
                    a.get("action_type")
                        .and_then(|v| v.as_str())
                        .map(String::from)
                })
                .collect();

            if action_types.contains(&"EncounterPickEncounter".to_string()) {
                return true;
            }

            if action_types.contains(&"EncounterConcludeEncounter".to_string()) {
                post_action(
                    client,
                    &serde_json::json!({"action_type": "EncounterConcludeEncounter"}),
                );
                continue;
            }

            if action_types.contains(&"EncounterApplyScouting".to_string()) {
                post_action(
                    client,
                    &serde_json::json!({"action_type": "EncounterApplyScouting", "card_ids": []}),
                );
                continue;
            }

            // No recognized transition action — game over or stuck
            return false;
        }
        false
    }

    fn play_combat_encounter(&self, client: &Client, strategy: &dyn Strategy) -> u32 {
        let mut rounds = 0;

        for _ in 0..self.max_actions_per_encounter {
            let snapshot = get_snapshot(client);

            // Check if encounter is over
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

            // Get playable cards for current phase
            let playable = get_playable_combat_cards(client, &snapshot);
            if playable.is_empty() {
                return rounds;
            }

            // Let strategy choose
            let action = strategy.choose_action(&playable, &snapshot);
            post_action(client, &action);
            rounds += 1;
        }

        rounds
    }

    fn play_non_combat_encounter(&self, client: &Client, _strategy: &dyn Strategy) {
        for _ in 0..self.max_actions_per_encounter {
            let possible = get_possible_actions(client);
            let action_types: Vec<String> = possible
                .iter()
                .filter_map(|a| {
                    a.get("action_type")
                        .and_then(|v| v.as_str())
                        .map(String::from)
                })
                .collect();

            if action_types.contains(&"EncounterApplyScouting".to_string()) {
                post_action(
                    client,
                    &serde_json::json!({"action_type": "EncounterApplyScouting", "card_ids": []}),
                );
                return;
            }
            if action_types.contains(&"EncounterConcludeEncounter".to_string()) {
                post_action(
                    client,
                    &serde_json::json!({"action_type": "EncounterConcludeEncounter"}),
                );
                continue;
            }
            if action_types.contains(&"EncounterAbort".to_string()) {
                post_action(
                    client,
                    &serde_json::json!({"action_type": "EncounterAbort"}),
                );
                continue;
            }
            if action_types.contains(&"EncounterPlayCard".to_string()) {
                let cards = get_hand_cards(client);
                if let Some(card_id) = cards.first() {
                    post_action(
                        client,
                        &serde_json::json!({"action_type": "EncounterPlayCard", "card_id": card_id}),
                    );
                    continue;
                }
            }
            return;
        }
    }
}

// --- HTTP helper functions (public API only) ---

fn post_action(client: &Client, action: &Value) -> (Status, Value) {
    let response = client
        .post("/action")
        .header(ContentType::JSON)
        .body(action.to_string())
        .dispatch();
    let status = response.status();
    let body: Value = response
        .into_string()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(Value::Null);
    (status, body)
}

fn get_json(client: &Client, path: &str) -> Value {
    let response = client.get(path).dispatch();
    response
        .into_string()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(Value::Null)
}

fn get_possible_actions(client: &Client) -> Vec<Value> {
    let val = get_json(client, "/actions/possible");
    val.as_array().cloned().unwrap_or_default()
}

fn get_snapshot(client: &Client) -> GameSnapshot {
    let encounter = {
        let resp = client.get("/encounter").dispatch();
        if resp.status() == Status::Ok {
            resp.into_string()
                .and_then(|s| serde_json::from_str(&s).ok())
        } else {
            None
        }
    };
    let tokens = get_json(client, "/player/tokens");
    GameSnapshot { encounter, tokens }
}

fn find_combat_encounter(client: &Client) -> Option<usize> {
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

fn get_encounter_hand_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| c.get("id").and_then(|v| v.as_u64()).map(|v| v as usize))
        .collect()
}

fn get_hand_cards(client: &Client) -> Vec<u64> {
    // Get all hand cards (any kind except Encounter and effect types)
    let cards = get_json(client, "/library/cards?location=Hand");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter(|c| {
            let kind = c.get("kind");
            // Exclude encounter and effect cards
            !matches!(
                kind.and_then(|k| k.as_object())
                    .and_then(|obj| obj.keys().next())
                    .map(|s| s.as_str()),
                Some("Encounter" | "PlayerCardEffect" | "EnemyCardEffect")
            )
        })
        .filter_map(|c| c.get("id").and_then(|v| v.as_u64()))
        .collect()
}

/// Get playable combat cards enriched with details for strategy decision-making.
/// Returns action-ready JSON values with card_id and card metadata.
fn get_playable_combat_cards(client: &Client, snapshot: &GameSnapshot) -> Vec<Value> {
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
                .and_then(|k| k.as_object())
                .and_then(|obj| obj.values().next())
                .and_then(|v| v.get("effects"))
                .cloned()
                .unwrap_or(Value::Null);
            serde_json::json!({
                "action_type": "EncounterPlayCard",
                "card_id": card_id,
                "card_kind": card_kind,
                "card_details": {
                    "effects": effects
                }
            })
        })
        .collect()
}

fn get_encounter_results_count(client: &Client) -> usize {
    let results = get_json(client, "/encounter/results");
    results.as_array().map(|a| a.len()).unwrap_or(0)
}

fn get_last_encounter_outcome(client: &Client, previous_count: usize) -> Option<String> {
    let results = get_json(client, "/encounter/results");
    let arr = results.as_array()?;
    if arr.len() > previous_count {
        arr.last().and_then(|v| v.as_str()).map(String::from)
    } else {
        None
    }
}
