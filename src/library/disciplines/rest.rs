use crate::library::types::{
    self, CardCounts, CardEffectCost, CardEffectKind, CardKind, EncounterKind, EncounterOutcome,
    EncounterState, RestEncounterState,
};
use crate::library::{GameState, Library};

pub(crate) fn register_rest_cards(lib: &mut Library, rng: &mut rand_pcg::Lcg64Xsh32) {
    // Rest PlayerCardEffect entries (GainTokens templates with RestToken + material costs).
    // Costs use percentage-of-gain: roll cap → roll gain% → roll cost% of gain.

    // 1. Stamina recovery — costs Fish (10-30%) + Plant (10-30%)
    let stamina_effect_id = lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::GainTokens {
                target: types::EffectTarget::OnSelf,
                token_type: types::TokenType::Stamina,
                cap_min: 400,
                cap_max: 600,
                gain_min_percent: 80,
                gain_max_percent: 100,
                costs: vec![
                    CardEffectCost {
                        cost_type: types::TokenType::Fish,
                        min_percent: 10,
                        max_percent: 30,
                    },
                    CardEffectCost {
                        cost_type: types::TokenType::Plant,
                        min_percent: 10,
                        max_percent: 30,
                    },
                ],
                duration: types::TokenLifecycle::PersistentCounter,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // 2. Health recovery — costs Fish (15-35%) + Plant (15-35%)
    let health_effect_id = lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::GainTokens {
                target: types::EffectTarget::OnSelf,
                token_type: types::TokenType::Health,
                cap_min: 300,
                cap_max: 500,
                gain_min_percent: 80,
                gain_max_percent: 100,
                costs: vec![
                    CardEffectCost {
                        cost_type: types::TokenType::Fish,
                        min_percent: 15,
                        max_percent: 35,
                    },
                    CardEffectCost {
                        cost_type: types::TokenType::Plant,
                        min_percent: 15,
                        max_percent: 35,
                    },
                ],
                duration: types::TokenLifecycle::PersistentCounter,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // 3. Mixed Stamina + Health — no RestToken cost (free play), lower gains
    let mixed_stamina_effect_id = lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::GainTokens {
                target: types::EffectTarget::OnSelf,
                token_type: types::TokenType::Stamina,
                cap_min: 200,
                cap_max: 350,
                gain_min_percent: 70,
                gain_max_percent: 90,
                costs: vec![],
                duration: types::TokenLifecycle::PersistentCounter,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    let mixed_health_effect_id = lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::GainTokens {
                target: types::EffectTarget::OnSelf,
                token_type: types::TokenType::Health,
                cap_min: 150,
                cap_max: 300,
                gain_min_percent: 70,
                gain_max_percent: 90,
                costs: vec![],
                duration: types::TokenLifecycle::PersistentCounter,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // Roll concrete rest cards referencing these effect templates.
    // Distribution: 2 stamina, 2 health, 1 mixed.
    // Stamina/health cards cost 1 rest token; mixed card costs 0.
    let effect_groups: Vec<(Vec<usize>, i64)> = vec![
        (vec![stamina_effect_id], 1),
        (vec![stamina_effect_id], 1),
        (vec![health_effect_id], 1),
        (vec![health_effect_id], 1),
        (vec![mixed_stamina_effect_id, mixed_health_effect_id], 0),
    ];

    for (effect_ids, rest_token_cost) in &effect_groups {
        let effects: Vec<types::ConcreteEffect> = effect_ids
            .iter()
            .map(|&eid| crate::library::game_state::roll_concrete_effect(rng, eid, lib))
            .collect();
        lib.add_card(
            CardKind::Rest {
                effects,
                rest_token_cost: *rest_token_cost,
            },
            CardCounts {
                library: 0,
                deck: 5,
                hand: 0,
                discard: 0,
            },
        );
    }

    // Register rest encounter card (~20% of encounter deck = hand: 4)
    lib.add_card(
        CardKind::Encounter {
            encounter_kind: EncounterKind::Rest {
                rest_def: types::RestDef {
                    rest_token_min: 1,
                    rest_token_max: 2,
                },
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 4,
            discard: 0,
        },
    );
}

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
        let (effects, rest_token_cost) = match &lib_card.kind {
            CardKind::Rest {
                effects,
                rest_token_cost,
            } => (effects.clone(), *rest_token_cost),
            _ => return Err("Cannot play a non-rest card in rest encounter".to_string()),
        };

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

        // Apply GainTokens effects
        for effect in &effects {
            let effect_kind = self.library.resolve_effect(effect.effect_id);
            if let Some(CardEffectKind::GainTokens {
                token_type,
                duration,
                ..
            }) = effect_kind
            {
                let token = types::Token {
                    token_type: token_type.clone(),
                    lifecycle: duration.clone(),
                };
                let entry = self.token_balances.entry(token).or_insert(0);
                *entry += effect.rolled_value;
                // Cap the token balance at rolled_cap
                if let Some(cap) = effect.rolled_cap {
                    if *entry > cap {
                        *entry = cap;
                    }
                }
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
        self.last_encounter_result = Some(EncounterOutcome::PlayerWon);
        self.encounter_results.push(EncounterOutcome::PlayerWon);
        // Return rest cards from hand to discard
        for card in self.library.cards.iter_mut() {
            if matches!(card.kind, CardKind::Rest { .. }) && card.counts.hand > 0 {
                card.counts.discard += card.counts.hand;
                card.counts.hand = 0;
            }
        }
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
        self.last_encounter_result = Some(EncounterOutcome::PlayerWon);
        self.encounter_results.push(EncounterOutcome::PlayerWon);
        self.current_encounter = None;
        self.encounter_phase = types::EncounterPhase::Scouting;
    }
}
