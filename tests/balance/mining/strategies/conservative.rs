use serde_json::Value;

use crate::strategies::{GameSnapshot, Strategy};

/// Conservative mining strategy — plays only no-cost cards, concludes early
/// to preserve durability and secure yield.
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

    fn choose_action(&self, possible_actions: &[Value], game_state: &GameSnapshot) -> Value {
        let current_yield = game_state.mining_yield();
        let light = game_state.mining_light_level();

        // Conclude early if we have decent yield or light is low
        if current_yield > 500 || light <= 80 {
            if let Some(conclude) = possible_actions.iter().find(|a| {
                a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterConcludeEncounter")
            }) {
                return conclude.clone();
            }
        }

        // Play no-cost cards only
        let no_cost_cards: Vec<&Value> = possible_actions
            .iter()
            .filter(|a| {
                a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterPlayCard")
                    && a.get("card_details")
                        .and_then(|d| d.get("has_cost"))
                        .and_then(|v| v.as_bool())
                        != Some(true)
            })
            .collect();

        if !no_cost_cards.is_empty() {
            // Prefer cards with highest power among no-cost
            let best = no_cost_cards
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

        // No free cards left — conclude
        possible_actions
            .iter()
            .find(|a| {
                a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterConcludeEncounter")
            })
            .cloned()
            .unwrap_or_else(|| possible_actions[0].clone())
    }
}
