use serde_json::Value;

use crate::strategies::{GameSnapshot, Strategy};

/// Greedy mining strategy — always plays the highest MiningPower card.
/// Only concludes when light level drops to ≤ 50 or no playable cards remain.
pub struct GreedyStrategy;

impl GreedyStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Strategy for GreedyStrategy {
    fn name(&self) -> &str {
        "greedy"
    }

    fn choose_action(&self, possible_actions: &[Value], game_state: &GameSnapshot) -> Value {
        let light = game_state.mining_light_level();

        let cards: Vec<&Value> = possible_actions
            .iter()
            .filter(|a| a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterPlayCard"))
            .collect();

        // Conclude if light is very low — mining is inefficient
        if light <= 50 && cards.is_empty() {
            if let Some(conclude) = possible_actions.iter().find(|a| {
                a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterConcludeEncounter")
            }) {
                return conclude.clone();
            }
        }

        if light <= 50 {
            if let Some(conclude) = possible_actions.iter().find(|a| {
                a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterConcludeEncounter")
            }) {
                return conclude.clone();
            }
        }

        // Pick highest mining power card
        if !cards.is_empty() {
            let best = cards
                .iter()
                .max_by_key(|c| {
                    c.get("card_details")
                        .and_then(|d| d.get("mining_power"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0)
                })
                .unwrap();
            return (*best).clone();
        }

        // Fallback: conclude
        possible_actions
            .iter()
            .find(|a| {
                a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterConcludeEncounter")
            })
            .cloned()
            .unwrap_or_else(|| possible_actions[0].clone())
    }
}
