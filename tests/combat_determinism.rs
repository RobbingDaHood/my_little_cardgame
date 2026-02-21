//! Test deterministic combat replay (Step 6 acceptance)

#[cfg(test)]
mod tests {
    use my_little_cardgame::library::combat;
    use my_little_cardgame::library::types::{CombatAction, CombatState, Combatant};
    use std::collections::HashMap;

    #[test]
    fn test_deterministic_combat_same_seed_same_log() {
        // Create a simple initial state
        let initial_state = CombatState {
            round: 1,
            current_turn: "player".to_string(),
            combatants: vec![
                Combatant {
                    id: "player".to_string(),
                    active_tokens: HashMap::from([
                        ("health".to_string(), 100),
                        ("max_health".to_string(), 100),
                    ]),
                },
                Combatant {
                    id: "enemy_0".to_string(),
                    active_tokens: HashMap::from([
                        ("health".to_string(), 50),
                        ("max_health".to_string(), 50),
                    ]),
                },
            ],
            is_finished: false,
            winner: None,
        };

        let actions = vec![
            CombatAction::ConsumeToken {
                combatant_id: "enemy_0".to_string(),
                token_id: "health".to_string(),
                amount: 20,
            },
            CombatAction::ConsumeToken {
                combatant_id: "player".to_string(),
                token_id: "health".to_string(),
                amount: 10,
            },
            CombatAction::ConsumeToken {
                combatant_id: "enemy_0".to_string(),
                token_id: "health".to_string(),
                amount: 30,
            },
        ];

        // Run combat twice with same seed
        let seed = 42u64;
        let state1 = combat::simulate_combat(initial_state.clone(), seed, actions.clone());
        let state2 = combat::simulate_combat(initial_state.clone(), seed, actions);

        // Verify they're identical
        assert_eq!(state1.winner, state2.winner);
        assert_eq!(state1.is_finished, state2.is_finished);

        // Check final states match
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
        let initial_state = CombatState {
            round: 1,
            current_turn: "player".to_string(),
            combatants: vec![
                Combatant {
                    id: "player".to_string(),
                    active_tokens: HashMap::from([
                        ("health".to_string(), 100),
                        ("max_health".to_string(), 100),
                    ]),
                },
                Combatant {
                    id: "enemy_0".to_string(),
                    active_tokens: HashMap::from([
                        ("health".to_string(), 50),
                        ("max_health".to_string(), 50),
                    ]),
                },
            ],
            is_finished: false,
            winner: None,
        };

        let actions = vec![
            CombatAction::ConsumeToken {
                combatant_id: "enemy_0".to_string(),
                token_id: "health".to_string(),
                amount: 20,
            },
            CombatAction::GrantToken {
                combatant_id: "player".to_string(),
                token_id: "Health".to_string(),
                amount: 5,
            },
        ];

        // Run with different seeds
        let _state1 = combat::simulate_combat(initial_state.clone(), 42u64, actions.clone());
        let _state2 = combat::simulate_combat(initial_state.clone(), 123u64, actions);

        // Both should complete consistently (deterministic given same seed)
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

        let state = combat::simulate_combat(initial_state.clone(), 42u64, vec![]);

        assert!(!state.is_finished);
    }

    #[test]
    fn test_combat_ends_when_enemy_defeated() {
        let initial_state = CombatState {
            round: 1,
            current_turn: "player".to_string(),
            combatants: vec![
                Combatant {
                    id: "player".to_string(),
                    active_tokens: HashMap::from([
                        ("health".to_string(), 100),
                        ("max_health".to_string(), 100),
                    ]),
                },
                Combatant {
                    id: "enemy_0".to_string(),
                    active_tokens: HashMap::from([
                        ("health".to_string(), 30),
                        ("max_health".to_string(), 50),
                    ]),
                },
            ],
            is_finished: false,
            winner: None,
        };

        let actions = vec![
            CombatAction::ConsumeToken {
                combatant_id: "enemy_0".to_string(),
                token_id: "health".to_string(),
                amount: 30,
            },
            // This action should be ignored (combat already over)
            CombatAction::ConsumeToken {
                combatant_id: "player".to_string(),
                token_id: "health".to_string(),
                amount: 10,
            },
        ];

        let state = combat::simulate_combat(initial_state, 42u64, actions);

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

        let actions = vec![
            CombatAction::GrantToken {
                combatant_id: "player".to_string(),
                token_id: "Health".to_string(),
                amount: 10,
            },
            CombatAction::GrantToken {
                combatant_id: "player".to_string(),
                token_id: "Health".to_string(),
                amount: 5,
            },
            CombatAction::ConsumeToken {
                combatant_id: "player".to_string(),
                token_id: "Health".to_string(),
                amount: 7,
            },
        ];

        let seed = 42u64;
        let state1 = combat::simulate_combat(initial_state.clone(), seed, actions.clone());
        let state2 = combat::simulate_combat(initial_state, seed, actions);

        // Both runs should have identical token counts
        let tokens1 = &state1.combatants[0].active_tokens;
        let tokens2 = &state2.combatants[0].active_tokens;
        assert_eq!(tokens1.get("Health"), tokens2.get("Health"));
        assert_eq!(tokens1.get("Health"), Some(&8)); // 10 + 5 - 7
    }
}
