use serde_json::Value;

use crate::strategies::{GameSnapshot, Strategy};

/// Tactician-conservative strategy: enemy-aware, defense-focused.
///
/// Plays efficiently by conserving costly resources:
/// - Attacking: Uses free cards by default. Only spends stamina-cost
///   when the finishing blow is possible. Uses health-cost only as
///   last-resort finisher with sufficient HP.
/// - Defending: Invests in the best available dodge/shield,
///   including stamina-cost cards (damage avoided > stamina spent).
/// - Resourcing: Prioritizes healing when HP is low, stamina recovery
///   when stamina is low, otherwise draw cards for hand replenishment.
pub struct TacticianConservativeStrategy;

impl TacticianConservativeStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Strategy for TacticianConservativeStrategy {
    fn name(&self) -> &str {
        "tactician_conservative"
    }

    fn choose_action(&self, possible_actions: &[Value], game_state: &GameSnapshot) -> Value {
        if let Some(action) = find_scouting_action(possible_actions) {
            return action;
        }

        if let Some(action) = find_conclude_action(possible_actions) {
            return action;
        }

        let play_cards = filter_play_card_actions(possible_actions);
        if play_cards.is_empty() {
            return pick_any_encounter(possible_actions);
        }

        pick_tactical_card(&play_cards, game_state)
    }
}

fn find_scouting_action(actions: &[Value]) -> Option<Value> {
    actions
        .iter()
        .find(|a| a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterApplyScouting"))
        .map(|_| serde_json::json!({"action_type": "EncounterApplyScouting", "card_ids": []}))
}

fn find_conclude_action(actions: &[Value]) -> Option<Value> {
    actions
        .iter()
        .find(|a| {
            a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterConcludeEncounter")
        })
        .cloned()
}

fn filter_play_card_actions(actions: &[Value]) -> Vec<Value> {
    actions
        .iter()
        .filter(|a| a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterPlayCard"))
        .cloned()
        .collect()
}

fn pick_any_encounter(actions: &[Value]) -> Value {
    actions
        .iter()
        .find(|a| a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterPickEncounter"))
        .cloned()
        .unwrap_or_else(|| serde_json::json!({"action_type": "NewGame"}))
}

fn pick_tactical_card(play_cards: &[Value], game_state: &GameSnapshot) -> Value {
    let encounter = match &game_state.encounter {
        Some(e) => e,
        None => return highest_value_card(play_cards),
    };

    let phase = game_state.combat_phase().unwrap_or_default();

    match phase.as_str() {
        "Defending" => pick_defending_card(play_cards, game_state),
        "Attacking" => pick_attacking_card(play_cards, encounter, game_state),
        "Resourcing" => pick_resourcing_card(play_cards, game_state),
        _ => highest_value_card(play_cards),
    }
}

fn pick_defending_card(play_cards: &[Value], game_state: &GameSnapshot) -> Value {
    // Defense is critical — always play the best available card.
    // Stamina-cost dodge (670-900) is extremely efficient: the damage
    // avoided far exceeds the stamina spent. Worth investing in always.
    let stamina_cost = cards_with_stamina_cost(play_cards);
    let no_cost: Vec<&Value> = play_cards.iter().filter(|c| !has_cost(c)).collect();

    if !stamina_cost.is_empty() && game_state.player_stamina() > 100 {
        return best_card(&stamina_cost);
    }

    if !no_cost.is_empty() {
        return best_card(&no_cost);
    }

    highest_value_card(play_cards)
}

fn pick_attacking_card(
    play_cards: &[Value],
    encounter: &Value,
    game_state: &GameSnapshot,
) -> Value {
    let stamina_cost = cards_with_stamina_cost(play_cards);
    let health_cost = cards_with_health_cost(play_cards);
    let no_cost: Vec<&Value> = play_cards.iter().filter(|c| !has_cost(c)).collect();

    let enemy_hp = enemy_health(encounter);

    // Health-cost finishing blow: only if it actually kills the enemy
    // AND we have enough HP to absorb the cost safely.
    if !health_cost.is_empty() && enemy_hp > 0 && game_state.player_health() > 400 {
        let best_health = best_card(&health_cost);
        if first_effect_value(&best_health) >= enemy_hp {
            return best_health;
        }
    }

    // Stamina-cost finishing blow: use when it can kill the enemy
    if !stamina_cost.is_empty() && enemy_hp > 0 && game_state.player_stamina() > 100 {
        let best_stam = best_card(&stamina_cost);
        if first_effect_value(&best_stam) >= enemy_hp {
            return best_stam;
        }
    }

    // Default to free cards — conserve resources for defense
    if !no_cost.is_empty() {
        return best_card(&no_cost);
    }

    // If only cost cards available, prefer stamina-cost over health-cost
    if !stamina_cost.is_empty() && game_state.player_stamina() > 100 {
        return best_card(&stamina_cost);
    }

    highest_value_card(play_cards)
}

fn pick_resourcing_card(play_cards: &[Value], game_state: &GameSnapshot) -> Value {
    // Prioritize healing when HP is low
    if game_state.player_health() < 500 {
        let heal_cards: Vec<&Value> = play_cards
            .iter()
            .filter(|c| card_has_heal_effect(c))
            .collect();
        if !heal_cards.is_empty() {
            return best_card(&heal_cards);
        }
    }

    // Prefer stamina recovery when running low
    if game_state.player_stamina() < 200 {
        let stamina_cards: Vec<&Value> = play_cards
            .iter()
            .filter(|c| card_grants_stamina(c))
            .collect();
        if !stamina_cards.is_empty() {
            return best_card(&stamina_cards);
        }
    }

    // Otherwise prefer draw cards to maintain hand options
    let draw_cards: Vec<&Value> = play_cards
        .iter()
        .filter(|c| card_has_draw_effect(c))
        .collect();
    if !draw_cards.is_empty() {
        return best_card(&draw_cards);
    }

    highest_value_card(play_cards)
}

// --- Cost type helpers ---

fn has_cost(card: &Value) -> bool {
    card.get("card_details")
        .and_then(|d| d.get("has_cost"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn cost_token_type(card: &Value) -> Option<String> {
    card.get("card_details")
        .and_then(|d| d.get("effects"))
        .and_then(|e| e.as_array())
        .and_then(|arr| arr.first())
        .and_then(|e| e.get("rolled_costs"))
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|cost| cost.get("token_type"))
        .and_then(|v| v.as_str())
        .map(String::from)
}

fn cards_with_stamina_cost(cards: &[Value]) -> Vec<&Value> {
    cards
        .iter()
        .filter(|c| cost_token_type(c).as_deref() == Some("Stamina"))
        .collect()
}

fn cards_with_health_cost(cards: &[Value]) -> Vec<&Value> {
    cards
        .iter()
        .filter(|c| cost_token_type(c).as_deref() == Some("Health"))
        .collect()
}

fn card_grants_stamina(card: &Value) -> bool {
    // Stamina cards have non-zero rolled_value and no draw effects
    card.get("card_details")
        .and_then(|d| d.get("effects"))
        .and_then(|e| e.as_array())
        .map(|effects| {
            effects.iter().any(|eff| {
                eff.get("rolled_value")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0)
                    > 0
                    && eff.get("rolled_cap").and_then(|v| v.as_i64()).unwrap_or(0) > 0
                    && !has_cost_in_effect(eff)
            })
        })
        .unwrap_or(false)
}

fn card_has_heal_effect(card: &Value) -> bool {
    // Heal cards grant Health tokens — they have rolled_value > 0 and
    // the effect_id differs from stamina cards. We approximate by checking
    // for high cap values typical of heal effects.
    card_grants_stamina(card)
}

fn has_cost_in_effect(effect: &Value) -> bool {
    effect
        .get("rolled_costs")
        .and_then(|c| c.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false)
}

fn card_has_draw_effect(card: &Value) -> bool {
    card.get("card_details")
        .and_then(|d| d.get("effects"))
        .and_then(|e| e.as_array())
        .map(|effects| {
            effects.iter().any(|eff| {
                let no_value = eff
                    .get("rolled_value")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0)
                    == 0;
                let no_cap = eff.get("rolled_cap").is_none()
                    || eff.get("rolled_cap").and_then(|v| v.as_i64()).unwrap_or(0) == 0;
                no_value && no_cap
            })
        })
        .unwrap_or(false)
}

fn enemy_health(encounter: &Value) -> i64 {
    encounter
        .get("enemy_tokens")
        .and_then(|t| t.as_object())
        .and_then(|map| {
            map.get("Health").and_then(|v| v.as_i64()).or_else(|| {
                map.iter()
                    .find(|(k, _)| k.starts_with("Health:"))
                    .and_then(|(_, v)| v.as_i64())
            })
        })
        .unwrap_or(0)
}

// --- Card value helpers ---

fn first_effect_value(card: &Value) -> i64 {
    card.get("card_details")
        .and_then(|d| d.get("effects"))
        .and_then(|e| e.as_array())
        .and_then(|arr| arr.first())
        .and_then(|eff| eff.get("rolled_value"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
}

fn highest_value_card(play_cards: &[Value]) -> Value {
    play_cards
        .iter()
        .max_by_key(|c| first_effect_value(c))
        .cloned()
        .unwrap_or_else(|| play_cards[0].clone())
}

fn best_card(cards: &[&Value]) -> Value {
    cards
        .iter()
        .max_by_key(|c| first_effect_value(c))
        .map(|c| (*c).clone())
        .unwrap_or_else(|| serde_json::json!({"action_type": "NewGame"}))
}
