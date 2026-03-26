use serde_json::Value;

use crate::strategies::{GameSnapshot, Strategy};

/// Greedy woodcutting strategy — picks the card with highest `rolled_value`.
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

    fn choose_action(&self, possible_actions: &[Value], _game_state: &GameSnapshot) -> Value {
        let cards: Vec<&Value> = possible_actions
            .iter()
            .filter(|a| a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterPlayCard"))
            .collect();

        if !cards.is_empty() {
            let best = cards
                .iter()
                .max_by_key(|c| {
                    c.get("card_details")
                        .and_then(|d| d.get("rolled_value"))
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
