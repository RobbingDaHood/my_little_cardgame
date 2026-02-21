//! End-to-end tests for Steps 6-7 encounter loop

#[cfg(test)]
mod tests {
    use my_little_cardgame::library::encounter;
    use my_little_cardgame::library::types::{
        EncounterAction, EncounterPhase, EncounterState, ScoutingParameters,
    };

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
        let state_after_play = encounter::apply_action(
            &state,
            EncounterAction::PlayCard {
                card_id: 1,
                effects: vec!["damage".to_string()],
            },
        );
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
}
