use crate::library::types::{
    self, CardEffectKind, CardKind, EncounterKind, EncounterOutcome, EncounterState,
    RestEncounterState,
};
use crate::library::GameState;

impl GameState {
    /// Initialize a rest encounter. Draws rest cards from the player's Library
    /// deck to hand and sets initial rest tokens (1–2).
    pub fn start_rest_encounter(
        &mut self,
        encounter_card_id: usize,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let lib_card = self
            .library
            .get(encounter_card_id)
            .ok_or_else(|| format!("Card {} not found in Library", encounter_card_id))?
            .clone();
        let rest_def = match &lib_card.kind {
            CardKind::Encounter {
                encounter_kind: EncounterKind::Rest { rest_def },
            } => rest_def.clone(),
            _ => {
                return Err(format!(
                    "Card {} is not a rest encounter",
                    encounter_card_id
                ))
            }
        };

        // Draw rest cards from deck to hand
        let max_hand =
            types::token_balance_by_type(&self.token_balances, &types::TokenType::RestMaxHand);
        self.draw_player_cards_of_kind(
            max_hand as u32,
            |k| matches!(k, CardKind::Rest { .. }),
            rng,
            Some(types::TokenType::RestMaxHand),
        );

        // Set initial rest tokens from encounter definition
        let initial_rest_tokens = crate::library::game_state::roll_range(
            rng,
            rest_def.rest_token_min,
            rest_def.rest_token_max,
        );

        let state = RestEncounterState {
            encounter_card_id,
            outcome: EncounterOutcome::Undecided,
            rest_tokens: initial_rest_tokens,
        };
        self.current_encounter = Some(EncounterState::Rest(state));
        self.encounter_phase = types::EncounterPhase::InEncounter;
        Ok(())
    }

    /// Play a rest card from the player's hand.
    /// Applies GainTokens effects, deducts RestToken costs from encounter state,
    /// deducts other costs from player token_balances, and moves card hand→discard.
    pub fn resolve_rest_card_play(
        &mut self,
        card_id: usize,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let lib_card = self
            .library
            .get(card_id)
            .ok_or_else(|| format!("Card {} not found in Library", card_id))?
            .clone();
        let effects = match &lib_card.kind {
            CardKind::Rest { effects } => effects.clone(),
            _ => return Err("Cannot play a non-rest card in rest encounter".to_string()),
        };
        let rest_token_cost = GameState::extract_rest_token_cost(&effects);

        if lib_card.counts.hand == 0 {
            return Err(format!("Card {} is not in hand", card_id));
        }

        // Check rest tokens are sufficient
        {
            let rest_state = match &self.current_encounter {
                Some(EncounterState::Rest(r)) => r,
                _ => return Err("No active rest encounter".to_string()),
            };
            if rest_state.rest_tokens < rest_token_cost {
                return Err(format!(
                    "Insufficient RestTokens: need {} but have {}",
                    rest_token_cost, rest_state.rest_tokens
                ));
            }
        }

        // Check and deduct costs from player token_balances
        GameState::check_and_deduct_costs(&effects, &mut self.token_balances)?;

        // Apply GainTokens/Insight effects
        for effect in &effects {
            let effect_kind = self.library.resolve_effect(effect.effect_id);
            match effect_kind {
                Some(CardEffectKind::GainTokens {
                    token_type,
                    duration,
                    ..
                }) => {
                    let token = types::Token {
                        token_type: token_type.clone(),
                        lifecycle: duration.clone(),
                    };
                    let entry = self.token_balances.entry(token).or_insert(0);
                    *entry += effect.rolled_value;
                }
                Some(CardEffectKind::Insight { .. }) => {
                    let entry = types::token_entry_by_type(
                        &mut self.token_balances,
                        &types::TokenType::RestInsight,
                    );
                    *entry += effect.rolled_value;
                }
                _ => {}
            }
        }

        // Move card hand→discard
        self.library.play(card_id)?;

        // Deduct rest tokens from encounter state
        if let Some(EncounterState::Rest(ref mut rest_state)) = self.current_encounter {
            rest_state.rest_tokens -= rest_token_cost;
        }

        // Draw replacement rest card
        self.draw_player_cards_of_kind(
            1,
            |k| matches!(k, CardKind::Rest { .. }),
            rng,
            Some(types::TokenType::RestMaxHand),
        );

        // Check if rest tokens are depleted → auto-complete
        let tokens_depleted = matches!(
            &self.current_encounter,
            Some(EncounterState::Rest(r)) if r.rest_tokens <= 0
        );
        if tokens_depleted {
            self.complete_rest_encounter();
        }

        Ok(())
    }

    /// Complete the rest encounter as PlayerWon.
    fn complete_rest_encounter(&mut self) {
        if let Some(EncounterState::Rest(ref mut r)) = self.current_encounter {
            r.outcome = EncounterOutcome::PlayerWon;
        }
        self.record_encounter_finish(types::Discipline::Rest, EncounterOutcome::PlayerWon, 1);
        // Return rest cards from hand to discard
        for card in self.library.cards.iter_mut() {
            if matches!(card.kind, CardKind::Rest { .. }) && card.counts.hand > 0 {
                card.counts.discard += card.counts.hand;
                card.counts.hand = 0;
            }
        }
        self.capture_last_encounter_kind();
        self.current_encounter = None;
        self.encounter_phase = types::EncounterPhase::Scouting;
    }

    /// Abort a rest encounter — always results in PlayerWon.
    pub fn abort_rest_encounter(&mut self) {
        // Return rest cards from hand to discard
        for card in self.library.cards.iter_mut() {
            if matches!(card.kind, CardKind::Rest { .. }) && card.counts.hand > 0 {
                card.counts.discard += card.counts.hand;
                card.counts.hand = 0;
            }
        }
        self.record_encounter_finish(types::Discipline::Rest, EncounterOutcome::PlayerWon, 1);
        self.capture_last_encounter_kind();
        self.current_encounter = None;
        self.encounter_phase = types::EncounterPhase::Scouting;
    }
}
