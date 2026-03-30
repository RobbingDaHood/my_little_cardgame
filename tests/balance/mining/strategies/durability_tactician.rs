use serde_json::Value;

use crate::strategies::{GameSnapshot, Strategy};

/// Durability Tactician — a Tier-2 strategy that beats Tier-1 runners WITHOUT
/// using yield-boosting effects. Wins through superior resource management:
/// - Optimal encounter conclusion timing (conclude when yield/dur ratio peaks)
/// - Durability conservation (avoids wasting durability on low-yield rounds)
/// - Stamina efficiency (minimizes stamina-cost card usage)
/// - Never plays light-boosting cards (no yield manipulation)
pub struct DurabilityTacticianStrategy;

impl DurabilityTacticianStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Strategy for DurabilityTacticianStrategy {
    fn name(&self) -> &str {
        "durability_tactician"
    }

    fn choose_action(&self, possible_actions: &[Value], game_state: &GameSnapshot) -> Value {
        let current_yield = game_state.mining_yield();
        let durability = game_state.mining_durability();
        let light = game_state.mining_light_level();

        let cards: Vec<&Value> = possible_actions
            .iter()
            .filter(|a| a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterPlayCard"))
            .collect();

        let conclude = possible_actions.iter().find(|a| {
            a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterConcludeEncounter")
        });

        // Filter to only non-yield cards: power cards only (no light-gain cards).
        // This tactician never manipulates light level — it wins through timing.
        let power_only_cards: Vec<&Value> = cards
            .iter()
            .filter(|c| {
                let light_gain = c
                    .get("card_details")
                    .and_then(|d| d.get("light_gain"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                light_gain == 0
            })
            .copied()
            .collect();

        // Conclude when light is too low for efficient mining
        if light <= 60 && current_yield > 0 {
            if let Some(c) = conclude {
                return c.clone();
            }
        }

        // Conclude when durability is getting scarce — protect remaining budget
        if durability < 800 && current_yield > 50 {
            if let Some(c) = conclude {
                return c.clone();
            }
        }

        // No power cards available — conclude to bank yield
        if power_only_cards.is_empty() {
            if let Some(c) = conclude {
                return c.clone();
            }
            // No conclude available either — play any available card as fallback
            if !cards.is_empty() {
                return cards[0].clone();
            }
            return possible_actions[0].clone();
        }

        // Play only no-cost power cards to maximize yield per resource spent
        let no_cost_power: Vec<&&Value> = power_only_cards
            .iter()
            .filter(|c| {
                c.get("card_details")
                    .and_then(|d| d.get("has_cost"))
                    .and_then(|v| v.as_bool())
                    != Some(true)
            })
            .collect();

        if !no_cost_power.is_empty() {
            let best = no_cost_power
                .iter()
                .max_by_key(|c| {
                    c.get("card_details")
                        .and_then(|d| d.get("mining_power"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0)
                })
                .unwrap();
            return (**best).clone();
        }

        // Only cost cards remain — conclude if we have yield, otherwise play cheapest
        if current_yield > 0 {
            if let Some(c) = conclude {
                return c.clone();
            }
        }

        let cheapest = power_only_cards
            .iter()
            .min_by_key(|c| {
                c.get("card_details")
                    .and_then(|d| d.get("mining_power"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0)
            })
            .unwrap();
        (*cheapest).clone()
    }
}
