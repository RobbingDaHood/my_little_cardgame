//! Test deterministic combat replay (Step 6 acceptance)

#[cfg(test)]
mod tests {
    use my_little_cardgame::library::combat;
    use my_little_cardgame::library::types::{
        CardDef, CardEffect, CombatAction, CombatOutcome, CombatPhase, CombatSnapshot, Combatant,
        EffectTarget, TokenType,
    };
    use std::collections::HashMap;

    fn attack_card(id: u64, damage: i64) -> CardDef {
        CardDef {
            id,
            card_type: "Attack".to_string(),
            effects: vec![CardEffect {
                target: EffectTarget::OnOpponent,
                token_id: TokenType::Health,
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
                token_id: TokenType::Health,
                amount,
            }],
        }
    }

    fn buff_card(id: u64, token: TokenType, amount: i64) -> CardDef {
        CardDef {
            id,
            card_type: "Resource".to_string(),
            effects: vec![CardEffect {
                target: EffectTarget::OnSelf,
                token_id: token,
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
        defs.insert(5, buff_card(5, TokenType::Health, 10));
        defs.insert(6, buff_card(6, TokenType::Health, 5));
        defs.insert(7, buff_card(7, TokenType::Health, -7));
        defs
    }

    fn two_combatant_snapshot(player_hp: i64, enemy_hp: i64) -> CombatSnapshot {
        CombatSnapshot {
            round: 1,
            player_turn: true,
            phase: CombatPhase::Defending,
            player_tokens: HashMap::from([
                (TokenType::Health, player_hp),
                (TokenType::MaxHealth, player_hp),
            ]),
            enemy: Combatant {
                active_tokens: HashMap::from([
                    (TokenType::Health, enemy_hp),
                    (TokenType::MaxHealth, enemy_hp),
                ]),
            },
            encounter_card_id: None,
            is_finished: false,
            outcome: CombatOutcome::Undecided,
        }
    }

    #[test]
    fn test_deterministic_combat_same_seed_same_log() {
        let initial_state = two_combatant_snapshot(100, 50);
        let card_defs = test_card_defs();

        let actions = vec![
            CombatAction {
                is_player: true,
                card_id: 1,
            },
            CombatAction {
                is_player: false,
                card_id: 2,
            },
            CombatAction {
                is_player: true,
                card_id: 3,
            },
        ];

        let seed = 42u64;
        let state1 =
            combat::simulate_combat(initial_state.clone(), seed, actions.clone(), &card_defs);
        let state2 = combat::simulate_combat(initial_state.clone(), seed, actions, &card_defs);

        assert_eq!(state1.outcome, state2.outcome);
        assert_eq!(state1.is_finished, state2.is_finished);
        assert_eq!(
            state1.player_tokens.get(&TokenType::Health),
            state2.player_tokens.get(&TokenType::Health)
        );
        assert_eq!(
            state1.enemy.active_tokens.get(&TokenType::Health),
            state2.enemy.active_tokens.get(&TokenType::Health)
        );
    }

    #[test]
    fn test_different_seeds_may_differ() {
        let initial_state = two_combatant_snapshot(100, 50);
        let card_defs = test_card_defs();

        let actions = vec![
            CombatAction {
                is_player: true,
                card_id: 1,
            },
            CombatAction {
                is_player: true,
                card_id: 4,
            },
        ];

        let _state1 =
            combat::simulate_combat(initial_state.clone(), 42u64, actions.clone(), &card_defs);
        let _state2 = combat::simulate_combat(initial_state.clone(), 123u64, actions, &card_defs);
    }

    #[test]
    fn test_empty_combat_produces_log() {
        let initial_state = CombatSnapshot {
            round: 1,
            player_turn: true,
            phase: CombatPhase::Defending,
            player_tokens: HashMap::from([(TokenType::Health, 100), (TokenType::MaxHealth, 100)]),
            enemy: Combatant {
                active_tokens: HashMap::new(),
            },
            encounter_card_id: None,
            is_finished: false,
            outcome: CombatOutcome::Undecided,
        };
        let card_defs = test_card_defs();

        let state = combat::simulate_combat(initial_state, 42u64, vec![], &card_defs);
        assert!(!state.is_finished);
    }

    #[test]
    fn test_combat_ends_when_enemy_defeated() {
        let initial_state = two_combatant_snapshot(100, 30);
        let card_defs = test_card_defs();

        let actions = vec![
            CombatAction {
                is_player: true,
                card_id: 3,
            },
            CombatAction {
                is_player: false,
                card_id: 2,
            },
        ];

        let state = combat::simulate_combat(initial_state, 42u64, actions, &card_defs);

        assert!(state.is_finished);
        assert_eq!(state.outcome, CombatOutcome::PlayerWon);
    }

    #[test]
    fn test_token_operations_deterministic() {
        let initial_state = CombatSnapshot {
            round: 1,
            player_turn: true,
            phase: CombatPhase::Defending,
            player_tokens: HashMap::from([(TokenType::Health, 100), (TokenType::MaxHealth, 100)]),
            enemy: Combatant {
                active_tokens: HashMap::new(),
            },
            encounter_card_id: None,
            is_finished: false,
            outcome: CombatOutcome::Undecided,
        };
        let card_defs = test_card_defs();

        let actions = vec![
            CombatAction {
                is_player: true,
                card_id: 5,
            },
            CombatAction {
                is_player: true,
                card_id: 6,
            },
            CombatAction {
                is_player: true,
                card_id: 7,
            },
        ];

        let seed = 42u64;
        let state1 =
            combat::simulate_combat(initial_state.clone(), seed, actions.clone(), &card_defs);
        let state2 = combat::simulate_combat(initial_state, seed, actions, &card_defs);

        assert_eq!(
            state1.player_tokens.get(&TokenType::Health),
            state2.player_tokens.get(&TokenType::Health)
        );
        assert_eq!(state1.player_tokens.get(&TokenType::Health), Some(&108));
    }
}
