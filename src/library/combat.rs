//! Deterministic, pure-data combat resolution (Step 6)
//!
//! This module provides pure functions for resolving combat deterministically
//! using seeded RNG. Current combat scope is minimal: attack cards reduce
//! opponent HP via token manipulation. Features like dodge, stamina, and
//! advanced mechanics are deferred to later roadmap steps.

use super::types::{CardDef, CombatAction, CombatState, EffectTarget};
use rand::RngCore;
use rand_pcg::Lcg64Xsh32;
use std::collections::HashMap;

/// Result of resolving a combat tick: (updated snapshot, updated player tokens, rng values used).
pub type CombatTickResult =
    Result<(CombatState, HashMap<super::types::Token, i64>, Vec<u64>), String>;

/// Resolve a single combat action (card play) deterministically.
///
/// Looks up the card definition, applies its effects as token operations,
/// checks victory/defeat, and advances the turn.
/// Returns an error if the card_id is unknown.
pub fn resolve_combat_tick(
    current_state: &CombatState,
    player_tokens: &HashMap<super::types::Token, i64>,
    action: &CombatAction,
    card_defs: &HashMap<u64, CardDef>,
    rng: &mut Lcg64Xsh32,
) -> CombatTickResult {
    let card = card_defs
        .get(&action.card_id)
        .ok_or_else(|| format!("Unknown card_id: {}", action.card_id))?;

    let mut rng_values = Vec::new();
    let mut state_after = current_state.clone();
    let mut pt_after = player_tokens.clone();

    let rng_val = rng.next_u64();
    rng_values.push(rng_val);

    for effect in &card.effects {
        let (target, token_type, amount) = match &effect.kind {
            super::types::CardEffectKind::ChangeTokens {
                target,
                token_type,
                amount,
            } => (target, token_type, *amount),
            super::types::CardEffectKind::DrawCards { .. } => continue,
        };
        let actor_tokens = match (target, action.is_player) {
            (EffectTarget::OnSelf, true) | (EffectTarget::OnOpponent, false) => &mut pt_after,
            (EffectTarget::OnOpponent, true) | (EffectTarget::OnSelf, false) => {
                &mut state_after.enemy_tokens
            }
        };
        let token_key = super::types::Token {
            token_type: token_type.clone(),
            lifecycle: effect.lifecycle.clone(),
        };
        let entry = actor_tokens.entry(token_key).or_insert(0);
        *entry = (*entry + amount).max(0);

        if *token_type == super::types::TokenType::Health && *entry == 0 {
            state_after.is_finished = true;
            let affected_is_player = matches!(
                (target, action.is_player),
                (EffectTarget::OnSelf, true) | (EffectTarget::OnOpponent, false)
            );
            state_after.outcome = if affected_is_player {
                super::types::CombatOutcome::EnemyWon
            } else {
                super::types::CombatOutcome::PlayerWon
            };
        }
    }

    if !state_after.is_finished {
        state_after.player_turn = !state_after.player_turn;
    }

    Ok((state_after, pt_after, rng_values))
}

/// Simulate a full combat encounter from a seed and initial state.
///
/// Returns the final combat snapshot and player tokens. Pure-data; no side effects.
pub fn simulate_combat(
    initial_state: CombatState,
    initial_player_tokens: HashMap<super::types::Token, i64>,
    seed: u64,
    actions: Vec<CombatAction>,
    card_defs: &HashMap<u64, CardDef>,
) -> (CombatState, HashMap<super::types::Token, i64>) {
    use rand::SeedableRng;

    let seed_bytes: [u8; 16] = {
        let s = seed.to_le_bytes();
        let mut bytes = [0u8; 16];
        bytes[0..8].copy_from_slice(&s);
        bytes[8..16].copy_from_slice(&s);
        bytes
    };
    let mut rng = Lcg64Xsh32::from_seed(seed_bytes);

    let mut current_state = initial_state;
    let mut player_tokens = initial_player_tokens;

    for action in &actions {
        match resolve_combat_tick(&current_state, &player_tokens, action, card_defs, &mut rng) {
            Ok((next_state, next_pt, _rng_vals)) => {
                current_state = next_state;
                player_tokens = next_pt;
                if current_state.is_finished {
                    break;
                }
            }
            Err(_) => break,
        }
    }

    (current_state, player_tokens)
}
