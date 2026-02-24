//! Encounter loop state machine (Step 7)
//!
//! Pure-data functions that manage encounter state transitions
//! based on player actions. Works with EncounterAction and EncounterState.

use super::types::{EncounterAction, EncounterPhase, EncounterState};

/// Process an EncounterAction and transition state accordingly.
///
/// Returns the new EncounterState after applying the action.
/// Returns None if the action is invalid for the current phase.
pub fn apply_action(state: &EncounterState, action: EncounterAction) -> Option<EncounterState> {
    match (&state.phase, action) {
        // Ready phase: can pick encounter or finish
        (EncounterPhase::Ready, EncounterAction::PickEncounter { card_id: _ }) => {
            let mut new_state = state.clone();
            new_state.phase = EncounterPhase::InCombat;
            Some(new_state)
        }
        (EncounterPhase::Ready, EncounterAction::FinishEncounter) => {
            let mut new_state = state.clone();
            new_state.phase = EncounterPhase::NoEncounter;
            Some(new_state)
        }

        // InCombat phase: play cards or end encounter
        (EncounterPhase::InCombat, EncounterAction::PlayCard { .. }) => {
            let new_state = state.clone();
            Some(new_state)
        }
        (EncounterPhase::InCombat, EncounterAction::FinishEncounter) => {
            let mut new_state = state.clone();
            new_state.phase = EncounterPhase::NoEncounter;
            Some(new_state)
        }

        // Scouting phase: apply scouting or finish
        (EncounterPhase::Scouting, EncounterAction::ApplyScouting { card_ids: _ }) => {
            let new_state = state.clone();
            // Scouting keeps encounter in Scouting phase until explicitly finished
            Some(new_state)
        }
        (EncounterPhase::Scouting, EncounterAction::FinishEncounter) => {
            let mut new_state = state.clone();
            new_state.phase = EncounterPhase::NoEncounter;
            Some(new_state)
        }

        // Invalid: all other state/action combinations
        _ => None,
    }
}

/// Check if encounter is finished
pub fn is_finished(state: &EncounterState) -> bool {
    state.phase == EncounterPhase::NoEncounter
}

/// Check if combat is active
pub fn is_in_combat(state: &EncounterState) -> bool {
    state.phase == EncounterPhase::InCombat
}

/// Check if post-encounter scouting is available
pub fn can_scout(state: &EncounterState) -> bool {
    state.phase == EncounterPhase::Scouting
}

/// Derive preview count from Foresight tokens.
///
/// Base preview is 1; each Foresight token adds 1 additional preview.
pub fn derive_preview_count(foresight_tokens: u64) -> u64 {
    1 + foresight_tokens
}
