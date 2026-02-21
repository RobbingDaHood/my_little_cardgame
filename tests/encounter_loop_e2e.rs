//! End-to-end tests for Steps 6-7 encounter loop

#[cfg(test)]
mod tests {
    use my_little_cardgame::library::encounter;
    use my_little_cardgame::library::types::{
        CombatState, Combatant, EncounterAction, EncounterPhase, EncounterState, ScoutingParameters,
    };

    #[test]
    fn test_encounter_full_loop_ready_to_combat_to_scouting() {
        // Initialize encounter in Ready phase
        let initial_state = EncounterState {
            encounter_id: "encounter_1".to_string(),
            area_id: "forest".to_string(),
            combat_state: CombatState {
                round: 0,
                current_turn: "player".to_string(),
                combatants: vec![
                    Combatant {
                        id: "player".to_string(),
                        current_hp: 100,
                        max_hp: 100,
                        active_tokens: std::collections::HashMap::new(),
                    },
                    Combatant {
                        id: "enemy_0".to_string(),
                        current_hp: 30,
                        max_hp: 30,
                        active_tokens: std::collections::HashMap::new(),
                    },
                ],
                is_finished: false,
                winner: None,
            },
            phase: EncounterPhase::Ready,
            scouting_parameters: ScoutingParameters {
                preview_count: 1,
                affix_bias: "balanced".to_string(),
                pool_modifier: 1.0,
            },
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
        let state_after_play = encounter::apply_action(
            &state,
            EncounterAction::PlayCard {
                card_id: 1,
                effects: vec!["damage".to_string()],
            },
        );
        assert!(state_after_play.is_some());
        let mut state = state_after_play.unwrap();
        assert_eq!(state.phase, EncounterPhase::InCombat);

        // Step 3: Simulate combat finishing (would normally happen from combat resolution)
        state.combat_state.is_finished = true;
        state.combat_state.winner = Some("player".to_string());

        // Check: can_scout should now work when combat is finished
        // In a real scenario, the state machine would auto-transition on combat end
        if state.combat_state.is_finished {
            state.phase = EncounterPhase::PostEncounter;
        }

        // Step 4: Apply scouting (PostEncounter -> PostEncounter with updated params)
        let state_after_scout = encounter::apply_action(
            &state,
            EncounterAction::ApplyScouting {
                card_ids: vec!["some_card".to_string()],
            },
        );
        assert!(state_after_scout.is_some());
        let state = state_after_scout.unwrap();
        assert_eq!(state.phase, EncounterPhase::PostEncounter);
        assert!(encounter::can_scout(&state));

        // Step 5: Finish encounter (PostEncounter -> Finished)
        let state_after_finish = encounter::apply_action(&state, EncounterAction::FinishEncounter);
        assert!(state_after_finish.is_some());
        let final_state = state_after_finish.unwrap();
        assert_eq!(final_state.phase, EncounterPhase::Finished);
        assert!(encounter::is_finished(&final_state));
    }

    #[test]
    fn test_encounter_invalid_actions_return_none() {
        let state = EncounterState {
            encounter_id: "enc".to_string(),
            area_id: "area".to_string(),
            combat_state: CombatState {
                round: 0,
                current_turn: "player".to_string(),
                combatants: vec![],
                is_finished: false,
                winner: None,
            },
            phase: EncounterPhase::Ready,
            scouting_parameters: ScoutingParameters {
                preview_count: 1,
                affix_bias: "balanced".to_string(),
                pool_modifier: 1.0,
            },
        };

        // PlayCard is invalid in Ready phase
        let result = encounter::apply_action(
            &state,
            EncounterAction::PlayCard {
                card_id: 1,
                effects: vec![],
            },
        );
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
    fn test_scouting_parameters_apply_more_preview() {
        let base = ScoutingParameters {
            preview_count: 1,
            affix_bias: "balanced".to_string(),
            pool_modifier: 1.0,
        };

        let updated = encounter::apply_scouting_parameters(&base, "more_preview");
        assert_eq!(updated.preview_count, 2); // incremented by 1
        assert_eq!(updated.pool_modifier, 1.0); // unchanged
    }

    #[test]
    fn test_scouting_parameters_apply_affix_boost() {
        let base = ScoutingParameters {
            preview_count: 1,
            affix_bias: "balanced".to_string(),
            pool_modifier: 1.0,
        };

        let updated = encounter::apply_scouting_parameters(&base, "affix_boost");
        assert_eq!(updated.preview_count, 1); // unchanged
        assert!(updated.pool_modifier > 1.0); // boosted by 10%
    }

    #[test]
    fn test_reset_scouting_parameters_returns_defaults() {
        let defaults = encounter::reset_scouting_parameters();
        assert_eq!(defaults.preview_count, 1);
        assert_eq!(defaults.affix_bias, "balanced".to_string());
        assert_eq!(defaults.pool_modifier, 1.0);
    }

    #[test]
    fn test_encounter_abandoned_in_combat() {
        let state = EncounterState {
            encounter_id: "enc".to_string(),
            area_id: "area".to_string(),
            combat_state: CombatState {
                round: 1,
                current_turn: "enemy".to_string(),
                combatants: vec![],
                is_finished: false,
                winner: None,
            },
            phase: EncounterPhase::InCombat,
            scouting_parameters: ScoutingParameters {
                preview_count: 1,
                affix_bias: "balanced".to_string(),
                pool_modifier: 1.0,
            },
        };

        // Player can finish/abandon encounter while in combat
        let result = encounter::apply_action(&state, EncounterAction::FinishEncounter);
        assert!(result.is_some());
        let finished = result.unwrap();
        assert_eq!(finished.phase, EncounterPhase::Finished);
    }

    #[test]
    fn test_encounter_multiple_scouting_choices_sequential() {
        let mut state = EncounterState {
            encounter_id: "enc".to_string(),
            area_id: "area".to_string(),
            combat_state: CombatState {
                round: 1,
                current_turn: "player".to_string(),
                combatants: vec![],
                is_finished: true,
                winner: Some("player".to_string()),
            },
            phase: EncounterPhase::PostEncounter,
            scouting_parameters: ScoutingParameters {
                preview_count: 1,
                affix_bias: "balanced".to_string(),
                pool_modifier: 1.0,
            },
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
        assert_eq!(state.phase, EncounterPhase::PostEncounter);

        // Second scouting choice (overrides first)
        let scout2 = encounter::apply_action(
            &state,
            EncounterAction::ApplyScouting {
                card_ids: vec!["second_card".to_string()],
            },
        );
        assert!(scout2.is_some());
        let final_state = scout2.unwrap();
        assert_eq!(final_state.phase, EncounterPhase::PostEncounter);
    }
}
