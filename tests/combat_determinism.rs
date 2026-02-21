//! Test deterministic combat replay (Step 6 acceptance)

#[cfg(test)]
mod tests {
    use my_little_cardgame::library::combat;
    use my_little_cardgame::library::types::{
        CardDef, CardEffect, CombatAction, CombatState, Combatant, EffectTarget,
    };
    use std::collections::HashMap;

    fn attack_card(id: u64, damage: i64) -> CardDef {
        CardDef {
            id,
            card_type: "Attack".to_string(),
            effects: vec![CardEffect {
                target: EffectTarget::OnOpponent,
                token_id: "health".to_string(),
                amount: -damage,
            }],
        }
    }

    fn heal_card(id: u64, amount: i64) -> CardDef {
        CardDef {
            id,
            card_type: "Resource".to_string(),
            effects: vec![CardEffect {
                target: EffectTarget::OnSelf,
                token_id: "health".to_string(),
                amount,
            }],
        }
    }

    fn buff_card(id: u64, token: &str, amount: i64) -> CardDef {
        CardDef {
            id,
            card_type: "Resource".to_string(),
            effects: vec![CardEffect {
                target: EffectTarget::OnSelf,
                token_id: token.to_string(),
                amount,
            }],
        }
    }

    fn test_card_defs() -> HashMap<u64, CardDef> {
        let mut defs = HashMap::new();
        defs.insert(1, attack_card(1, 20));
        defs.insert(2, attack_card(2, 10));
        defs.insert(3, attack_card(3, 30));
        defs.insert(4, heal_card(4, 5));
        defs.insert(5, buff_card(5, "Health", 10));
        defs.insert(6, buff_card(6, "Health", 5));
        defs.insert(7, buff_card(7, "Health", -7));
        defs
    }

    fn two_combatant_state(player_hp: i64, enemy_hp: i64) -> CombatState {
        CombatState {
            round: 1,
            current_turn: "player".to_string(),
            combatants: vec![
                Combatant {
                    id: "player".to_string(),
                    active_tokens: HashMap::from([
                        ("health".to_string(), player_hp),
                        ("max_health".to_string(), player_hp),
                    ]),
                },
                Combatant {
                    id: "enemy_0".to_string(),
                    active_tokens: HashMap::from([
                        ("health".to_string(), enemy_hp),
                        ("max_health".to_string(), enemy_hp),
                    ]),
                },
            ],
            is_finished: false,
            winner: None,
        }
    }

    #[test]
    fn test_deterministic_combat_same_seed_same_log() {
        let initial_state = two_combatant_state(100, 50);
        let card_defs = test_card_defs();

        let actions = vec![
            CombatAction {
                combatant_id: "player".to_string(),
                card_id: 1,
            }, // 20 damage to enemy
            CombatAction {
                combatant_id: "enemy_0".to_string(),
                card_id: 2,
            }, // 10 damage to player
            CombatAction {
                combatant_id: "player".to_string(),
                card_id: 3,
            }, // 30 damage to enemy (defeats)
        ];

        let seed = 42u64;
        let state1 =
            combat::simulate_combat(initial_state.clone(), seed, actions.clone(), &card_defs);
        let state2 = combat::simulate_combat(initial_state.clone(), seed, actions, &card_defs);

        assert_eq!(state1.winner, state2.winner);
        assert_eq!(state1.is_finished, state2.is_finished);

        for (c1, c2) in state1.combatants.iter().zip(state2.combatants.iter()) {
            assert_eq!(c1.id, c2.id);
            assert_eq!(
                c1.active_tokens.get("health"),
                c2.active_tokens.get("health")
            );
            assert_eq!(
                c1.active_tokens.get("max_health"),
                c2.active_tokens.get("max_health")
            );
        }
    }

    #[test]
    fn test_different_seeds_may_differ() {
        let initial_state = two_combatant_state(100, 50);
        let card_defs = test_card_defs();

        let actions = vec![
            CombatAction {
                combatant_id: "player".to_string(),
                card_id: 1,
            },
            CombatAction {
                combatant_id: "player".to_string(),
                card_id: 4,
            },
        ];

        let _state1 =
            combat::simulate_combat(initial_state.clone(), 42u64, actions.clone(), &card_defs);
        let _state2 = combat::simulate_combat(initial_state.clone(), 123u64, actions, &card_defs);
    }

    #[test]
    fn test_empty_combat_produces_log() {
        let initial_state = CombatState {
            round: 1,
            current_turn: "player".to_string(),
            combatants: vec![Combatant {
                id: "player".to_string(),
                active_tokens: HashMap::from([
                    ("health".to_string(), 100),
                    ("max_health".to_string(), 100),
                ]),
            }],
            is_finished: false,
            winner: None,
        };
        let card_defs = test_card_defs();

        let state = combat::simulate_combat(initial_state, 42u64, vec![], &card_defs);
        assert!(!state.is_finished);
    }

    #[test]
    fn test_combat_ends_when_enemy_defeated() {
        let initial_state = two_combatant_state(100, 30);
        let card_defs = test_card_defs();

        let actions = vec![
            CombatAction {
                combatant_id: "player".to_string(),
                card_id: 3,
            }, // 30 damage defeats enemy
            CombatAction {
                combatant_id: "enemy_0".to_string(),
                card_id: 2,
            }, // should be ignored
        ];

        let state = combat::simulate_combat(initial_state, 42u64, actions, &card_defs);

        assert!(state.is_finished);
        assert_eq!(state.winner, Some("player".to_string()));
    }

    #[test]
    fn test_token_operations_deterministic() {
        let initial_state = CombatState {
            round: 1,
            current_turn: "player".to_string(),
            combatants: vec![Combatant {
                id: "player".to_string(),
                active_tokens: HashMap::from([
                    ("health".to_string(), 100),
                    ("max_health".to_string(), 100),
                ]),
            }],
            is_finished: false,
            winner: None,
        };
        let card_defs = test_card_defs();

        // Cards 5, 6, 7 grant Health +10, +5, -7 on self -> net +8
        let actions = vec![
            CombatAction {
                combatant_id: "player".to_string(),
                card_id: 5,
            },
            CombatAction {
                combatant_id: "player".to_string(),
                card_id: 6,
            },
            CombatAction {
                combatant_id: "player".to_string(),
                card_id: 7,
            },
        ];

        let seed = 42u64;
        let state1 =
            combat::simulate_combat(initial_state.clone(), seed, actions.clone(), &card_defs);
        let state2 = combat::simulate_combat(initial_state, seed, actions, &card_defs);

        let tokens1 = &state1.combatants[0].active_tokens;
        let tokens2 = &state2.combatants[0].active_tokens;
        assert_eq!(tokens1.get("Health"), tokens2.get("Health"));
        assert_eq!(tokens1.get("Health"), Some(&8)); // 10 + 5 - 7
    }
}
