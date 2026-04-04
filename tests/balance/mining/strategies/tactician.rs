use serde_json::Value;

use crate::strategies::{GameSnapshot, Strategy};

/// Tactician mining strategy — the "yield optimizer" Tier-2 runner.
///
/// Core insight: each round an ore card plays regardless of what the player does,
/// costing ~19 durability. Playing a light card gives 0 yield that round, making it
/// a net durability loss. Instead, play ONLY power cards while light is high enough
/// for efficient mining (yield/dur > 1.5), then conclude immediately.
///
/// At initial light 200 with avg power 21 and avg ore dur ~19:
///   - Round at L=200: yield 42, yield/dur = 2.2
///   - Round at L=170: yield 36, yield/dur = 1.9
///   - Round at L=140: yield 29, yield/dur = 1.5
///   - Round at L=110: yield 23, yield/dur = 1.2 (inefficient)
///
/// The Tactician plays 2-3 rounds of pure power, then concludes before efficiency drops.
pub struct TacticianStrategy;

const LIGHT_CONCLUDE_THRESHOLD: i64 = 140;
const LIGHT_COST_CARD_THRESHOLD: i64 = 170;
const DURABILITY_SAFETY_MARGIN: i64 = 500;

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

        let conclude = possible_actions.iter().find(|a| {
            a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterConcludeEncounter")
        });

        // Conclude when light is too low for efficient mining
        if light < LIGHT_CONCLUDE_THRESHOLD && current_yield > 0 {
            if let Some(c) = conclude {
                return c.clone();
            }
        }

        // Conclude when durability is getting low
        if durability < DURABILITY_SAFETY_MARGIN && current_yield > 0 {
            if let Some(c) = conclude {
                return c.clone();
            }
        }

        let cards: Vec<&Value> = possible_actions
            .iter()
            .filter(|a| a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterPlayCard"))
            .collect();

        let free_power_cards: Vec<&&Value> = cards
            .iter()
            .filter(|c| card_mining_power(c) > 0 && !card_has_cost(c))
            .collect();

        let cost_power_cards: Vec<&&Value> = cards
            .iter()
            .filter(|c| card_mining_power(c) > 0 && card_has_cost(c))
            .collect();

        let has_any_power = !free_power_cards.is_empty() || !cost_power_cards.is_empty();

        // No power cards available → conclude or abort.
        // Returning conclude with yield=0 triggers a 400 → driver aborts.
        // This prevents wasting rounds on non-power cards (0 yield, ~19 dur cost).
        if !has_any_power {
            if let Some(c) = conclude {
                return c.clone();
            }
            if !cards.is_empty() {
                return cards[0].clone();
            }
            return possible_actions[0].clone();
        }

        // Play best free power card
        if !free_power_cards.is_empty() {
            let best = free_power_cards
                .iter()
                .max_by_key(|c| card_mining_power(c))
                .unwrap();
            return (**best).clone();
        }

        // Play cost power card only at high light (maximize the investment)
        if light >= LIGHT_COST_CARD_THRESHOLD && !cost_power_cards.is_empty() {
            let best = cost_power_cards
                .iter()
                .max_by_key(|c| card_mining_power(c))
                .unwrap();
            return (**best).clone();
        }

        // Cost power available but light too low → conclude
        if let Some(c) = conclude {
            return c.clone();
        }
        cards[0].clone()
    }
}

fn card_mining_power(c: &Value) -> i64 {
    c.get("card_details")
        .and_then(|d| d.get("mining_power"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
}

fn card_has_cost(c: &Value) -> bool {
    c.get("card_details")
        .and_then(|d| d.get("has_cost"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}
