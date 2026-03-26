use serde_json::Value;

use crate::strategies::{GameSnapshot, Strategy};

/// Conservative woodcutting strategy — picks the card with lowest `total_cost`.
/// Tie-breaks on lowest card_id for determinism.
pub struct ConservativeStrategy;

impl ConservativeStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Strategy for ConservativeStrategy {
    fn name(&self) -> &str {
        "conservative"
    }

    fn choose_action(&self, possible_actions: &[Value], _game_state: &GameSnapshot) -> Value {
        let cards: Vec<&Value> = possible_actions
            .iter()
            .filter(|a| a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterPlayCard"))
            .collect();

        if !cards.is_empty() {
            let best = cards
                .iter()
                .min_by(|a, b| {
                    let cost_a = a
                        .get("card_details")
                        .and_then(|d| d.get("total_cost"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(i64::MAX);
                    let cost_b = b
                        .get("card_details")
                        .and_then(|d| d.get("total_cost"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(i64::MAX);
                    cost_a.cmp(&cost_b).then_with(|| {
                        let id_a = a
                            .get("card_id")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(u64::MAX);
                        let id_b = b
                            .get("card_id")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(u64::MAX);
                        id_a.cmp(&id_b)
                    })
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
