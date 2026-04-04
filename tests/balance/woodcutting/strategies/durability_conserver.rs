use serde_json::Value;

use crate::strategies::{GameSnapshot, Strategy};

/// DurabilityConserver woodcutting strategy (tier-2, non-yield tactician).
///
/// Beats tier-1 through encounter-state-aware durability management:
/// - Picks lowest durability-cost card (ignoring stamina/health costs)
/// - Concludes early when durability budget is tight relative to remaining encounters
/// - Never optimizes for patterns — wins purely through resource conservation
pub struct DurabilityConserverStrategy;

impl DurabilityConserverStrategy {
    pub fn new() -> Self {
        Self
    }

    fn durability_cost(card: &Value) -> i64 {
        card.get("card_details")
            .and_then(|d| d.get("effects"))
            .and_then(|e| e.as_array())
            .unwrap_or(&vec![])
            .iter()
            .flat_map(|eff| {
                eff.get("rolled_costs")
                    .and_then(|c| c.as_array())
                    .cloned()
                    .unwrap_or_default()
            })
            .filter(|cost| {
                cost.get("token_type")
                    .and_then(|t| t.as_str())
                    .map(|s| s.contains("Durability"))
                    .unwrap_or(false)
            })
            .filter_map(|cost| cost.get("amount").and_then(|v| v.as_i64()))
            .sum()
    }

    fn played_card_count(card: &Value) -> usize {
        card.get("card_details")
            .and_then(|d| d.get("played_cards"))
            .and_then(|p| p.as_array())
            .map(|a| a.len())
            .unwrap_or(0)
    }

    fn should_conclude_early(cards: &[&Value], game_state: &GameSnapshot) -> bool {
        let durability = game_state.woodcutting_durability();
        if durability <= 0 {
            return true;
        }

        let played = cards
            .first()
            .map(|c| Self::played_card_count(c))
            .unwrap_or(0);

        // After at least 1 play, conserve durability for future encounters.
        // Estimate: need ~50 durability minimum per future encounter (cheapest card cost).
        // Reserve budget for remaining encounters in the session.
        if played >= 1 {
            let cheapest_dur_cost = cards
                .iter()
                .map(|c| Self::durability_cost(c))
                .filter(|&c| c > 0)
                .min()
                .unwrap_or(50);

            // If remaining durability can barely afford 2 more plays total,
            // conclude now to spread budget across more encounters.
            if durability < cheapest_dur_cost * 3 {
                return true;
            }
        }

        false
    }
}

impl Strategy for DurabilityConserverStrategy {
    fn name(&self) -> &str {
        "durability_conserver"
    }

    fn choose_action(&self, possible_actions: &[Value], game_state: &GameSnapshot) -> Value {
        let cards: Vec<&Value> = possible_actions
            .iter()
            .filter(|a| a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterPlayCard"))
            .collect();

        let has_conclude = possible_actions.iter().any(|a| {
            a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterConcludeEncounter")
        });

        // Early conclude decision based on durability budget
        if !cards.is_empty() && has_conclude && Self::should_conclude_early(&cards, game_state) {
            return possible_actions
                .iter()
                .find(|a| {
                    a.get("action_type").and_then(|v| v.as_str())
                        == Some("EncounterConcludeEncounter")
                })
                .cloned()
                .unwrap_or_else(|| possible_actions[0].clone());
        }

        if !cards.is_empty() {
            // Pick card with lowest durability cost (not total cost — ignore stamina/health)
            let best = cards
                .iter()
                .min_by(|a, b| {
                    let dur_a = Self::durability_cost(a);
                    let dur_b = Self::durability_cost(b);
                    dur_a.cmp(&dur_b).then_with(|| {
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
