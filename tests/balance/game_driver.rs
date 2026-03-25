use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;
use serde_json::Value;

use crate::combat::driver::{find_combat_encounter, play_combat_encounter};
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
    /// Consecutive win streaks per "life" (reset on death).
    /// Each entry is the streak length before the next death or game end.
    pub win_streaks: Vec<u32>,
    /// Maximum consecutive wins achieved in this game.
    pub max_win_streak: u32,
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
            win_streaks: Vec::new(),
            max_win_streak: 0,
        };

        let mut initial_deaths = get_snapshot(&client).player_deaths();
        let mut current_streak: u32 = 0;

        for encounter_num in 0..self.max_encounters {
            if !self.advance_to_encounter_pick(&client, strategy) {
                // Game is stuck — restart to continue simulation (counts as death/streak break).
                let new_game = serde_json::json!({"action_type": "NewGame", "seed": seed.wrapping_add(encounter_num as u64 * 1000)});
                post_action(&client, &new_game);
                initial_deaths = get_snapshot(&client).player_deaths();
                result.deaths += 1;
                if current_streak > 0 {
                    result.win_streaks.push(current_streak);
                }
                current_streak = 0;
                continue;
            }

            let combat_enc_id = find_combat_encounter(&client);
            let combat_enc_id = match combat_enc_id {
                Some(id) => id,
                None => {
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

            let results_before = get_encounter_results_count(&client);
            let rounds = play_combat_encounter(&client, strategy, self.max_actions_per_encounter);
            result.rounds_per_encounter.push(rounds);

            let outcome = get_last_encounter_outcome(&client, results_before);
            match outcome.as_deref() {
                Some("PlayerWon") => {
                    result.combat_wins += 1;
                    current_streak += 1;
                    if current_streak > result.max_win_streak {
                        result.max_win_streak = current_streak;
                    }
                }
                Some("PlayerLost") => {
                    result.combat_losses += 1;
                    if current_streak > 0 {
                        result.win_streaks.push(current_streak);
                    }
                    current_streak = 0;
                }
                _ => {}
            }

            let snapshot = get_snapshot(&client);
            let current_deaths = snapshot.player_deaths();
            let new_deaths = current_deaths - initial_deaths;
            if new_deaths > 0 {
                result.deaths += new_deaths;
                initial_deaths = current_deaths;
                result.ended_by_death = true;
                if current_streak > 0 {
                    result.win_streaks.push(current_streak);
                }
                current_streak = 0;
            }
        }

        // Push final streak if game ended without death/loss
        if current_streak > 0 {
            result.win_streaks.push(current_streak);
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

            // Stuck (e.g., in combat with no playable cards).
            if action_types.contains(&"NewGame".to_string()) {
                return false;
            }

            return false;
        }
        false
    }

    fn play_non_combat_encounter(&self, client: &Client, _strategy: &dyn Strategy) {
        // Abort non-combat encounters immediately — simulation only tracks combat.
        // Then handle scouting to return to NoEncounter phase.
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
            if action_types.contains(&"EncounterPickEncounter".to_string()) {
                return;
            }
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

pub(crate) fn post_action(client: &Client, action: &Value) -> (Status, Value) {
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

pub(crate) fn get_json(client: &Client, path: &str) -> Value {
    let response = client.get(path).dispatch();
    response
        .into_string()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(Value::Null)
}

pub(crate) fn get_possible_actions(client: &Client) -> Vec<Value> {
    let val = get_json(client, "/actions/possible");
    val.as_array().cloned().unwrap_or_default()
}

pub(crate) fn get_snapshot(client: &Client) -> GameSnapshot {
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
    let cards = get_json(client, "/library/cards?location=Hand");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter(|c| {
            let kind = c.get("kind");
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
