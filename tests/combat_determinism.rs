//! Test deterministic combat replay (Step 6 acceptance)

#[cfg(test)]
mod tests {
    use my_little_cardgame::library::combat;
    use my_little_cardgame::library::types::{
        token_balance_by_type, CardDef, CardEffect, CardEffectKind, CombatAction, CombatOutcome,
        CombatPhase, CombatState, EffectTarget, Token, TokenType,
    };
    use std::collections::HashMap;

    fn attack_card(id: u64, damage: i64) -> CardDef {
        CardDef {
            id,
            card_type: "Attack".to_string(),
            effects: vec![CardEffect {
                kind: CardEffectKind::ChangeTokens {
                    target: EffectTarget::OnOpponent,
                    token_type: TokenType::Health,
                    amount: -damage,
                },
                lifecycle: my_little_cardgame::library::types::TokenLifecycle::PersistentCounter,
                card_effect_id: None,
            }],
        }
    }

    fn heal_card(id: u64, amount: i64) -> CardDef {
        CardDef {
            id,
            card_type: "Resource".to_string(),
            effects: vec![CardEffect {
                kind: CardEffectKind::ChangeTokens {
                    target: EffectTarget::OnSelf,
                    token_type: TokenType::Health,
                    amount,
                },
                lifecycle: my_little_cardgame::library::types::TokenLifecycle::PersistentCounter,
                card_effect_id: None,
            }],
        }
    }

    fn buff_card(id: u64, token: TokenType, amount: i64) -> CardDef {
        CardDef {
            id,
            card_type: "Resource".to_string(),
            effects: vec![CardEffect {
                kind: CardEffectKind::ChangeTokens {
                    target: EffectTarget::OnSelf,
                    token_type: token,
                    amount,
                },
                lifecycle: my_little_cardgame::library::types::TokenLifecycle::PersistentCounter,
                card_effect_id: None,
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

    fn two_combatant_snapshot(player_hp: i64, enemy_hp: i64) -> (CombatState, HashMap<Token, i64>) {
        let player_tokens = HashMap::from([
            (Token::persistent(TokenType::Health), player_hp),
            (Token::persistent(TokenType::MaxHealth), player_hp),
        ]);
        let snapshot = CombatState {
            round: 1,
            player_turn: true,
            phase: CombatPhase::Defending,
            enemy_tokens: HashMap::from([
                (Token::persistent(TokenType::Health), enemy_hp),
                (Token::persistent(TokenType::MaxHealth), enemy_hp),
            ]),
            encounter_card_id: None,
            is_finished: false,
            outcome: CombatOutcome::Undecided,
            enemy_attack_deck: vec![],
            enemy_defence_deck: vec![],
            enemy_resource_deck: vec![],
        };
        (snapshot, player_tokens)
    }

    #[test]
    fn test_deterministic_combat_same_seed_same_log() {
        let (initial_state, initial_pt) = two_combatant_snapshot(100, 50);
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
        let (s1, pt1) = combat::simulate_combat(
            initial_state.clone(),
            initial_pt.clone(),
            seed,
            actions.clone(),
            &card_defs,
        );
        let (s2, pt2) =
            combat::simulate_combat(initial_state, initial_pt, seed, actions, &card_defs);

        assert_eq!(s1.outcome, s2.outcome);
        assert_eq!(s1.is_finished, s2.is_finished);
        assert_eq!(
            token_balance_by_type(&pt1, &TokenType::Health),
            token_balance_by_type(&pt2, &TokenType::Health)
        );
        assert_eq!(
            token_balance_by_type(&s1.enemy_tokens, &TokenType::Health),
            token_balance_by_type(&s2.enemy_tokens, &TokenType::Health)
        );
    }

    #[test]
    fn test_different_seeds_may_differ() {
        let (initial_state, initial_pt) = two_combatant_snapshot(100, 50);
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

        let _r1 = combat::simulate_combat(
            initial_state.clone(),
            initial_pt.clone(),
            42u64,
            actions.clone(),
            &card_defs,
        );
        let _r2 = combat::simulate_combat(initial_state, initial_pt, 123u64, actions, &card_defs);
    }

    #[test]
    fn test_empty_combat_produces_log() {
        let pt = HashMap::from([
            (Token::persistent(TokenType::Health), 100),
            (Token::persistent(TokenType::MaxHealth), 100),
        ]);
        let initial_state = CombatState {
            round: 1,
            player_turn: true,
            phase: CombatPhase::Defending,
            enemy_tokens: HashMap::new(),
            encounter_card_id: None,
            is_finished: false,
            outcome: CombatOutcome::Undecided,
            enemy_attack_deck: vec![],
            enemy_defence_deck: vec![],
            enemy_resource_deck: vec![],
        };
        let card_defs = test_card_defs();

        let (state, _pt) = combat::simulate_combat(initial_state, pt, 42u64, vec![], &card_defs);
        assert!(!state.is_finished);
    }

    #[test]
    fn test_combat_ends_when_enemy_defeated() {
        let (initial_state, initial_pt) = two_combatant_snapshot(100, 30);
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

        let (state, _pt) =
            combat::simulate_combat(initial_state, initial_pt, 42u64, actions, &card_defs);

        assert!(state.is_finished);
        assert_eq!(state.outcome, CombatOutcome::PlayerWon);
    }

    #[test]
    fn test_token_operations_deterministic() {
        let pt = HashMap::from([
            (Token::persistent(TokenType::Health), 100),
            (Token::persistent(TokenType::MaxHealth), 100),
        ]);
        let initial_state = CombatState {
            round: 1,
            player_turn: true,
            phase: CombatPhase::Defending,
            enemy_tokens: HashMap::new(),
            encounter_card_id: None,
            is_finished: false,
            outcome: CombatOutcome::Undecided,
            enemy_attack_deck: vec![],
            enemy_defence_deck: vec![],
            enemy_resource_deck: vec![],
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
        let (_s1, pt1) = combat::simulate_combat(
            initial_state.clone(),
            pt.clone(),
            seed,
            actions.clone(),
            &card_defs,
        );
        let (_s2, pt2) = combat::simulate_combat(initial_state, pt, seed, actions, &card_defs);

        assert_eq!(
            token_balance_by_type(&pt1, &TokenType::Health),
            token_balance_by_type(&pt2, &TokenType::Health)
        );
        assert_eq!(token_balance_by_type(&pt1, &TokenType::Health), 108);
    }
}
