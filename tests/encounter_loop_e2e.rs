//! End-to-end tests for Steps 6-7 encounter loop

#[cfg(test)]
mod tests {
    use my_little_cardgame::library::encounter;
    use my_little_cardgame::library::types::{EncounterAction, EncounterPhase, EncounterState};

    #[test]
    fn test_encounter_full_loop_ready_to_combat_to_scouting() {
        // Initialize encounter in Ready phase
        let initial_state = EncounterState {
            phase: EncounterPhase::Ready,
        };

        // Step 1: Pick encounter (Ready -> InCombat)
        let state_after_pick = encounter::apply_action(
            &initial_state,
            EncounterAction::PickEncounter {
                card_id: "encounter_1".to_string(),
            },
        );
        assert!(state_after_pick.is_some());
        let state = state_after_pick.unwrap();
        assert_eq!(state.phase, EncounterPhase::InCombat);
        assert!(encounter::is_in_combat(&state));

        // Step 2: Play card during combat (stays InCombat)
        let state_after_play =
            encounter::apply_action(&state, EncounterAction::PlayCard { card_id: 1 });
        assert!(state_after_play.is_some());
        let state = state_after_play.unwrap();
        assert_eq!(state.phase, EncounterPhase::InCombat);

        // Step 3: Simulate transition to Scouting (caller handles this externally)
        let state = EncounterState {
            phase: EncounterPhase::Scouting,
        };

        // Step 4: Apply scouting (Scouting -> Scouting with updated params)
        let state_after_scout = encounter::apply_action(
            &state,
            EncounterAction::ApplyScouting {
                card_ids: vec!["some_card".to_string()],
            },
        );
        assert!(state_after_scout.is_some());
        let state = state_after_scout.unwrap();
        assert_eq!(state.phase, EncounterPhase::Scouting);
        assert!(encounter::can_scout(&state));

        // Step 5: Finish encounter (Scouting -> NoEncounter)
        let state_after_finish = encounter::apply_action(&state, EncounterAction::FinishEncounter);
        assert!(state_after_finish.is_some());
        let final_state = state_after_finish.unwrap();
        assert_eq!(final_state.phase, EncounterPhase::NoEncounter);
        assert!(encounter::is_finished(&final_state));
    }

    #[test]
    fn test_encounter_invalid_actions_return_none() {
        let state = EncounterState {
            phase: EncounterPhase::Ready,
        };

        // PlayCard is invalid in Ready phase
        let result = encounter::apply_action(&state, EncounterAction::PlayCard { card_id: 1 });
        assert!(result.is_none());

        // ApplyScouting is invalid in Ready phase
        let result = encounter::apply_action(
            &state,
            EncounterAction::ApplyScouting {
                card_ids: vec!["some_card".to_string()],
            },
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_derive_preview_count_base() {
        assert_eq!(encounter::derive_preview_count(0), 1);
    }

    #[test]
    fn test_derive_preview_count_with_foresight() {
        assert_eq!(encounter::derive_preview_count(3), 4);
    }

    #[test]
    fn test_encounter_abandoned_in_combat() {
        let state = EncounterState {
            phase: EncounterPhase::InCombat,
        };

        // Player can finish/abandon encounter while in combat
        let result = encounter::apply_action(&state, EncounterAction::FinishEncounter);
        assert!(result.is_some());
        let finished = result.unwrap();
        assert_eq!(finished.phase, EncounterPhase::NoEncounter);
    }

    #[test]
    fn test_encounter_multiple_scouting_choices_sequential() {
        let mut state = EncounterState {
            phase: EncounterPhase::Scouting,
        };

        // First scouting choice
        let scout1 = encounter::apply_action(
            &state,
            EncounterAction::ApplyScouting {
                card_ids: vec!["first_card".to_string()],
            },
        );
        assert!(scout1.is_some());
        state = scout1.unwrap();
        assert_eq!(state.phase, EncounterPhase::Scouting);

        // Second scouting choice (overrides first)
        let scout2 = encounter::apply_action(
            &state,
            EncounterAction::ApplyScouting {
                card_ids: vec!["second_card".to_string()],
            },
        );
        assert!(scout2.is_some());
        let final_state = scout2.unwrap();
        assert_eq!(final_state.phase, EncounterPhase::Scouting);
    }

    #[test]
    fn test_encounter_loop_replay_from_seed() {
        use my_little_cardgame::library::combat;
        use my_little_cardgame::library::types::{
            CardDef, CardEffect, CombatAction, CombatSnapshot, Combatant, EffectTarget,
        };
        use std::collections::HashMap;

        let seed = 42u64;

        // Define cards
        let mut card_defs = HashMap::new();
        card_defs.insert(
            1,
            CardDef {
                id: 1,
                card_type: "Attack".to_string(),
                effects: vec![CardEffect {
                    target: EffectTarget::OnOpponent,
                    token_id: "health".to_string(),
                    amount: -15,
                }],
            },
        );

        // Record an action log (encounter actions + combat actions)
        let mut encounter_actions = Vec::new();
        let mut combat_actions = Vec::new();

        // Phase 1: Pick encounter
        let mut enc_state = EncounterState {
            phase: EncounterPhase::Ready,
        };
        let pick = EncounterAction::PickEncounter {
            card_id: "enc_1".to_string(),
        };
        encounter_actions.push(pick.clone());
        enc_state = encounter::apply_action(&enc_state, pick).unwrap();
        assert_eq!(enc_state.phase, EncounterPhase::InCombat);

        // Phase 2: Combat — play cards to defeat enemy
        let initial_combat = CombatSnapshot {
            round: 1,
            player_turn: true,
            player_tokens: HashMap::from([
                ("health".to_string(), 100),
                ("max_health".to_string(), 100),
            ]),
            enemy: Combatant {
                active_tokens: HashMap::from([
                    ("health".to_string(), 30),
                    ("max_health".to_string(), 30),
                ]),
            },
            is_finished: false,
            winner: None,
        };

        combat_actions.push(CombatAction {
            is_player: true,
            card_id: 1,
        });
        combat_actions.push(CombatAction {
            is_player: true,
            card_id: 1,
        });

        let combat_result = combat::simulate_combat(
            initial_combat.clone(),
            seed,
            combat_actions.clone(),
            &card_defs,
        );
        assert!(combat_result.is_finished);
        assert_eq!(combat_result.winner, Some("player".to_string()));

        // Phase 3: Scouting
        enc_state = EncounterState {
            phase: EncounterPhase::Scouting,
        };
        let scout = EncounterAction::ApplyScouting {
            card_ids: vec!["replacement".to_string()],
        };
        encounter_actions.push(scout.clone());
        enc_state = encounter::apply_action(&enc_state, scout).unwrap();
        assert_eq!(enc_state.phase, EncounterPhase::Scouting);

        let finish = EncounterAction::FinishEncounter;
        encounter_actions.push(finish.clone());
        enc_state = encounter::apply_action(&enc_state, finish).unwrap();
        assert_eq!(enc_state.phase, EncounterPhase::NoEncounter);

        // REPLAY: same seed + same actions produce same combat result
        let replay_result =
            combat::simulate_combat(initial_combat, seed, combat_actions, &card_defs);
        assert_eq!(replay_result.is_finished, combat_result.is_finished);
        assert_eq!(replay_result.winner, combat_result.winner);
        assert_eq!(
            replay_result.player_tokens.get("health"),
            combat_result.player_tokens.get("health")
        );
        assert_eq!(
            replay_result.enemy.active_tokens.get("health"),
            combat_result.enemy.active_tokens.get("health")
        );

        // REPLAY: same encounter actions produce same state machine result
        let mut replay_enc = EncounterState {
            phase: EncounterPhase::Ready,
        };
        for action in &encounter_actions {
            if let Some(next) = encounter::apply_action(&replay_enc, action.clone()) {
                replay_enc = next;
            }
            // Manually transition to Scouting after combat (same as original)
            if replay_enc.phase == EncounterPhase::InCombat
                && matches!(action, EncounterAction::PickEncounter { .. })
            {
                replay_enc = EncounterState {
                    phase: EncounterPhase::Scouting,
                };
            }
        }
        assert_eq!(replay_enc.phase, EncounterPhase::NoEncounter);
    }
}
