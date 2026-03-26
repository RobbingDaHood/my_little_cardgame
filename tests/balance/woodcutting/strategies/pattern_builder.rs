use serde_json::Value;
use std::collections::HashMap;

use crate::strategies::{GameSnapshot, Strategy};

/// PatternBuilder woodcutting strategy — reads `played_cards` to determine which
/// effect_ids have been played most, then picks cards with the most-common effect_id
/// to build "of a kind" patterns. Falls back to highest `hand_count` (most duplicates).
pub struct PatternBuilderStrategy;

impl PatternBuilderStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Strategy for PatternBuilderStrategy {
    fn name(&self) -> &str {
        "pattern_builder"
    }

    fn choose_action(&self, possible_actions: &[Value], _game_state: &GameSnapshot) -> Value {
        let cards: Vec<&Value> = possible_actions
            .iter()
            .filter(|a| a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterPlayCard"))
            .collect();

        if cards.is_empty() {
            return possible_actions
                .iter()
                .find(|a| {
                    a.get("action_type").and_then(|v| v.as_str())
                        == Some("EncounterConcludeEncounter")
                })
                .cloned()
                .unwrap_or_else(|| possible_actions[0].clone());
        }

        // Count effect_ids from already-played cards
        let mut effect_id_counts: HashMap<String, usize> = HashMap::new();
        if let Some(played) = cards
            .first()
            .and_then(|c| c.get("card_details"))
            .and_then(|d| d.get("played_cards"))
            .and_then(|p| p.as_array())
        {
            for played_card in played {
                // Each played woodcutting card has chop_types
                // Use chop_types as a proxy for effect pattern tracking
                if let Some(chop_types) = played_card.get("chop_types").and_then(|ct| ct.as_array())
                {
                    for ct in chop_types {
                        if let Some(s) = ct.as_str() {
                            *effect_id_counts.entry(s.to_string()).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        let most_common_type = effect_id_counts
            .iter()
            .max_by_key(|(_k, v)| *v)
            .map(|(k, _)| k.clone());

        // If we have a most-common type, prefer cards that produce that chop type
        if let Some(target_type) = &most_common_type {
            let matching: Vec<&&Value> = cards
                .iter()
                .filter(|c| {
                    let effects = c
                        .get("card_details")
                        .and_then(|d| d.get("effects"))
                        .and_then(|e| e.as_array());
                    if let Some(effects) = effects {
                        effects.iter().any(|eff| {
                            eff.get("chop_type")
                                .and_then(|ct| ct.as_str())
                                .map(|s| s == target_type)
                                .unwrap_or(false)
                        })
                    } else {
                        false
                    }
                })
                .collect();

            if !matching.is_empty() {
                let best = matching
                    .iter()
                    .max_by_key(|c| {
                        c.get("card_details")
                            .and_then(|d| d.get("hand_count"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0)
                    })
                    .unwrap();
                return (**best).clone();
            }
        }

        // Fallback: pick card with highest hand_count (most duplicates = better pattern odds)
        let best = cards
            .iter()
            .max_by_key(|c| {
                c.get("card_details")
                    .and_then(|d| d.get("hand_count"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
            })
            .unwrap();
        (*best).clone()
    }
}
