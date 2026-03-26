use serde_json::Value;

use crate::strategies::{GameSnapshot, Strategy};

/// Tactician mining strategy — manages light level and times power plays for
/// maximum yield per durability. Concludes at optimal moments.
///
/// Key heuristics:
/// - Boost light when it drops below threshold before playing power cards
/// - Play highest-power cards when light is high (multiplier effect)
/// - Conclude when yield/round diminishing or durability getting low
pub struct TacticianStrategy;

impl TacticianStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Strategy for TacticianStrategy {
    fn name(&self) -> &str {
        "tactician"
    }

    fn choose_action(&self, possible_actions: &[Value], game_state: &GameSnapshot) -> Value {
        let light = game_state.mining_light_level();
        let current_yield = game_state.mining_yield();
        let durability = game_state.mining_durability();

        let cards: Vec<&Value> = possible_actions
            .iter()
            .filter(|a| a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterPlayCard"))
            .collect();

        let conclude = possible_actions.iter().find(|a| {
            a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterConcludeEncounter")
        });

        // Conclude conditions:
        // 1. If durability is getting low, secure what we have
        // 2. If light is very low and we have good yield, conclude
        // 3. If we've accumulated significant yield
        if durability < 500 && current_yield > 200 {
            if let Some(c) = conclude {
                return c.clone();
            }
        }
        if light <= 30 && current_yield > 100 {
            if let Some(c) = conclude {
                return c.clone();
            }
        }

        if cards.is_empty() {
            if let Some(c) = conclude {
                return c.clone();
            }
            return possible_actions[0].clone();
        }

        // Separate cards by type
        let light_cards: Vec<&&Value> = cards
            .iter()
            .filter(|c| {
                c.get("card_details")
                    .and_then(|d| d.get("light_gain"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0)
                    > 0
            })
            .collect();

        let power_cards: Vec<&&Value> = cards
            .iter()
            .filter(|c| {
                c.get("card_details")
                    .and_then(|d| d.get("mining_power"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0)
                    > 0
                    && c.get("card_details")
                        .and_then(|d| d.get("light_gain"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0)
                        == 0
            })
            .collect();

        // Priority 1: If light is low, boost it first (if we have light cards)
        if light < 150 && !light_cards.is_empty() {
            let best_light = light_cards
                .iter()
                .max_by_key(|c| {
                    c.get("card_details")
                        .and_then(|d| d.get("light_gain"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0)
                })
                .unwrap();
            return (**best_light).clone();
        }

        // Priority 2: Play power cards when light is high (best efficiency)
        if light >= 150 && !power_cards.is_empty() {
            // Play highest power card, prefer no-cost first
            let no_cost_power: Vec<&&&Value> = power_cards
                .iter()
                .filter(|c| {
                    c.get("card_details")
                        .and_then(|d| d.get("has_cost"))
                        .and_then(|v| v.as_bool())
                        != Some(true)
                })
                .collect();

            let selection = if !no_cost_power.is_empty() {
                no_cost_power
                    .iter()
                    .max_by_key(|c| {
                        c.get("card_details")
                            .and_then(|d| d.get("mining_power"))
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0)
                    })
                    .unwrap()
            } else {
                // Use cost cards when light is high enough to justify it
                power_cards
                    .iter()
                    .max_by_key(|c| {
                        c.get("card_details")
                            .and_then(|d| d.get("mining_power"))
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0)
                    })
                    .unwrap()
            };
            return (***selection).clone();
        }

        // Priority 3: Play any available card (prefer no-cost)
        let no_cost: Vec<&&Value> = cards
            .iter()
            .filter(|c| {
                c.get("card_details")
                    .and_then(|d| d.get("has_cost"))
                    .and_then(|v| v.as_bool())
                    != Some(true)
            })
            .collect();

        if !no_cost.is_empty() {
            let best = no_cost
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

        // Use cost cards as last resort
        let best = cards
            .iter()
            .max_by_key(|c| {
                c.get("card_details")
                    .and_then(|d| d.get("mining_power"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0)
            })
            .unwrap();
        (*best).clone()
    }
}
