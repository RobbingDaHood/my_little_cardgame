use crate::library::types::{
    self, CardKind, CombatEncounterState, ConcreteEffect, EncounterKind, EncounterOutcome,
    EncounterState,
};
use crate::library::{GameState, Library};
use std::collections::HashMap;

/// Apply card effects to combat using concrete rolled values.
/// Only processes GainTokens and LoseTokens effects; DrawCards effects are handled separately.
fn apply_card_effects(
    effects: &[ConcreteEffect],
    is_player: bool,
    player_tokens: &mut HashMap<types::Token, i64>,
    combat: &mut CombatEncounterState,
    library: &Library,
) {
    for effect in effects {
        let kind = match library.resolve_effect(effect.effect_id) {
            Some(resolved) => resolved,
            None => continue,
        };

        let (target_tokens, token_type, is_loss) = match &kind {
            types::CardEffectKind::GainTokens {
                target, token_type, ..
            } => {
                let tokens = match (target, is_player) {
                    (types::EffectTarget::OnSelf, true)
                    | (types::EffectTarget::OnOpponent, false) => &mut *player_tokens,
                    (types::EffectTarget::OnOpponent, true)
                    | (types::EffectTarget::OnSelf, false) => &mut combat.enemy_tokens,
                };
                (tokens, token_type, false)
            }
            types::CardEffectKind::LoseTokens { token_type, .. } => {
                let tokens = if is_player {
                    &mut combat.enemy_tokens
                } else {
                    &mut *player_tokens
                };
                (tokens, token_type, true)
            }
            types::CardEffectKind::DrawCards { .. } => continue,
            types::CardEffectKind::Insight { .. } => {
                let entry =
                    types::token_entry_by_type(player_tokens, &types::TokenType::CombatInsight);
                *entry += effect.rolled_value;
                continue;
            }
            // Discipline-specific effects are not used in combat
            types::CardEffectKind::WoodcuttingChop { .. }
            | types::CardEffectKind::HerbalismMatch { .. }
            | types::CardEffectKind::FishingValue { .. }
            | types::CardEffectKind::CraftingReduction { .. }
            | types::CardEffectKind::ResearchProbe { .. } => continue,
        };

        if is_loss {
            let damage = effect.rolled_value;
            if *token_type == types::TokenType::Health {
                // Dodge absorbs first (timing-based, expires after Defending phase)
                let dodge = target_tokens
                    .get(&types::Token::dodge())
                    .copied()
                    .unwrap_or(0);
                let dodge_absorbed = dodge.min(damage);
                target_tokens.insert(types::Token::dodge(), (dodge - dodge_absorbed).max(0));
                let after_dodge = damage - dodge_absorbed;
                // Shield absorbs next (persists for encounter, blocks 1:1)
                let shield_key = types::Token::persistent(types::TokenType::Shield);
                let shield = target_tokens.get(&shield_key).copied().unwrap_or(0);
                let shield_absorbed = shield.min(after_dodge);
                target_tokens.insert(shield_key, (shield - shield_absorbed).max(0));
                let remaining_damage = after_dodge - shield_absorbed;
                if remaining_damage > 0 {
                    let health = target_tokens
                        .entry(types::Token::persistent(types::TokenType::Health))
                        .or_insert(0);
                    *health = (*health - remaining_damage).max(0);
                }
            } else {
                let entry = target_tokens
                    .entry(types::Token::persistent(token_type.clone()))
                    .or_insert(0);
                *entry = (*entry - damage).max(0);
            }
        } else {
            // GainTokens: gain = cap * gain_percent / 100, cap limits the gain not the total
            let grant_amount = match (effect.rolled_cap, effect.rolled_gain_percent) {
                (Some(cap), Some(pct)) => cap * pct as i64 / 100,
                _ => effect.rolled_value,
            };
            let entry = target_tokens
                .entry(types::Token::persistent(token_type.clone()))
                .or_insert(0);
            *entry = (*entry + grant_amount).max(0);
        }
    }
}

/// Check if combat has ended (either side at 0 health).
fn check_combat_end(player_tokens: &HashMap<types::Token, i64>, combat: &mut CombatEncounterState) {
    let player_health = player_tokens
        .get(&types::Token::persistent(types::TokenType::Health))
        .copied()
        .unwrap_or(0);
    let enemy_health = combat
        .enemy_tokens
        .get(&types::Token::persistent(types::TokenType::Health))
        .copied()
        .unwrap_or(0);

    if enemy_health <= 0 || player_health <= 0 {
        combat.outcome = if enemy_health <= 0 && player_health > 0 {
            EncounterOutcome::PlayerWon
        } else if player_health <= 0 && enemy_health > 0 {
            EncounterOutcome::PlayerLost
        } else {
            EncounterOutcome::PlayerWon // Draw defaults to player
        };
    }
}

impl GameState {
    /// Initialize combat from a Library Encounter card.
    pub fn start_combat(
        &mut self,
        encounter_card_id: usize,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let lib_card = self
            .library
            .get(encounter_card_id)
            .ok_or_else(|| format!("Card {} not found in Library", encounter_card_id))?
            .clone();
        let combatant_def = match &lib_card.kind {
            CardKind::Encounter {
                encounter_kind: EncounterKind::Combat { combatant_def },
            } => combatant_def.clone(),
            _ => {
                return Err(format!(
                    "Card {} is not a combat encounter",
                    encounter_card_id
                ))
            }
        };
        let mut enemy_attack_deck = combatant_def.attack_deck.clone();
        let mut enemy_defence_deck = combatant_def.defence_deck.clone();
        let mut enemy_resource_deck = combatant_def.resource_deck.clone();
        crate::library::game_state::deck_shuffle_hand(rng, &mut enemy_attack_deck);
        crate::library::game_state::deck_shuffle_hand(rng, &mut enemy_defence_deck);
        crate::library::game_state::deck_shuffle_hand(rng, &mut enemy_resource_deck);
        let snapshot = CombatEncounterState {
            round: 1,
            phase: types::CombatPhase::Defending,
            enemy_tokens: combatant_def
                .initial_tokens
                .iter()
                .map(|(k, v)| (k.clone(), *v as i64))
                .collect(),
            encounter_card_id,
            outcome: EncounterOutcome::Undecided,
            enemy_attack_deck,
            enemy_defence_deck,
            enemy_resource_deck,
        };
        self.current_encounter = Some(EncounterState::Combat(snapshot));
        self.encounter_phase = types::EncounterPhase::InEncounter;
        Ok(())
    }

    /// Resolve a player card play against the current combat encounter.
    pub fn resolve_player_card(
        &mut self,
        card_id: usize,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let combat = match &mut self.current_encounter {
            Some(EncounterState::Combat(c)) => c,
            _ => return Err("No active combat".to_string()),
        };
        let lib_card = self
            .library
            .get(card_id)
            .ok_or_else(|| format!("Card {} not found in Library", card_id))?
            .clone();
        let effects = match &lib_card.kind {
            CardKind::Attack { effects }
            | CardKind::Defence { effects }
            | CardKind::Resource { effects } => effects.clone(),
            _ => return Err("Cannot play a non-action card".to_string()),
        };
        // Pre-check: if no effect on the card can be paid, reject the play
        let any_payable = effects.iter().any(|effect| {
            if effect.rolled_costs.is_empty() {
                return true; // costless effect is always playable
            }
            Self::preview_costs(std::slice::from_ref(effect), &self.token_balances).is_ok()
        });
        if !any_payable && !effects.is_empty() {
            return Err("Cannot play card: no effect costs can be paid".to_string());
        }
        // Multi-effect evaluation: each effect is evaluated independently.
        // A previous effect can grant tokens that a later effect needs.
        // If an effect's cost cannot be paid, it is skipped (partial success).
        let (mut atk_draws, mut def_draws, mut res_draws) = (0u32, 0u32, 0u32);
        for effect in &effects {
            // Try to pay cost for this single effect
            if Self::check_and_deduct_costs(std::slice::from_ref(effect), &mut self.token_balances)
                .is_err()
            {
                continue;
            }
            if let Some(types::CardEffectKind::DrawCards {
                attack,
                defence,
                resource,
            }) = self.library.resolve_effect(effect.effect_id)
            {
                atk_draws += attack;
                def_draws += defence;
                res_draws += resource;
            }
            apply_card_effects(
                std::slice::from_ref(effect),
                true,
                &mut self.token_balances,
                combat,
                &self.library,
            );
        }
        check_combat_end(&self.token_balances, combat);
        if combat.outcome == EncounterOutcome::Undecided {
            // Check autoloss: if all hand cards are unpayable, player loses
            if self.all_combat_hand_cards_unpayable() {
                let combat = match &mut self.current_encounter {
                    Some(EncounterState::Combat(c)) => c,
                    _ => return Ok(()),
                };
                combat.outcome = EncounterOutcome::PlayerLost;
            }
        }
        let (outcome, rounds) = match &self.current_encounter {
            Some(EncounterState::Combat(c)) => (c.outcome.clone(), c.round),
            _ => (EncounterOutcome::Undecided, 0),
        };
        if outcome != EncounterOutcome::Undecided {
            if outcome == EncounterOutcome::PlayerWon {
                let entry = types::token_entry_by_type(
                    &mut self.token_balances,
                    &types::TokenType::MilestoneInsight,
                );
                *entry += 100;
            }
            self.record_encounter_finish(types::Discipline::Combat, outcome, rounds);
            self.current_encounter = None;
            self.encounter_phase = types::EncounterPhase::Scouting;
            self.check_player_death();
        }
        self.draw_player_cards_by_type(atk_draws, def_draws, res_draws, rng);
        Ok(())
    }

    /// Check if all combat hand cards (attack, defence, resource) are unpayable.
    /// A card is "unpayable" if ALL of its effects have costs that can't be afforded.
    /// A card with any costless effect is always playable.
    fn all_combat_hand_cards_unpayable(&self) -> bool {
        let hand_cards: Vec<_> = self
            .library
            .cards
            .iter()
            .filter(|c| {
                c.counts.hand > 0
                    && matches!(
                        c.kind,
                        CardKind::Attack { .. }
                            | CardKind::Defence { .. }
                            | CardKind::Resource { .. }
                    )
            })
            .collect();
        if hand_cards.is_empty() {
            return false;
        }
        hand_cards.iter().all(|card| {
            let effects = match &card.kind {
                CardKind::Attack { effects }
                | CardKind::Defence { effects }
                | CardKind::Resource { effects } => effects,
                _ => return false,
            };
            if effects.is_empty() {
                return false;
            }
            // Card is unpayable if ALL effects have costs and none can be afforded
            effects.iter().all(|effect| {
                if effect.rolled_costs.is_empty() {
                    return false; // costless effect → card is playable
                }
                Self::preview_costs(std::slice::from_ref(effect), &self.token_balances).is_err()
            })
        })
    }

    /// Resolve an enemy card play from hand in the current combat phase.
    /// Played cards move to discard. DrawCards effects trigger per-type enemy draws.
    pub fn resolve_enemy_play(&mut self, rng: &mut rand_pcg::Lcg64Xsh32) -> Result<(), String> {
        let combat = match &self.current_encounter {
            Some(EncounterState::Combat(c)) => c,
            _ => return Err("No active combat".to_string()),
        };
        let phase = combat.phase.clone();

        let combat = match &mut self.current_encounter {
            Some(EncounterState::Combat(c)) => c,
            _ => return Err("No active combat".to_string()),
        };
        let deck = match phase {
            types::CombatPhase::Attacking => &mut combat.enemy_attack_deck,
            types::CombatPhase::Defending => &mut combat.enemy_defence_deck,
            types::CombatPhase::Resourcing => &mut combat.enemy_resource_deck,
        };

        if let Some(card_idx) = crate::library::game_state::deck_play_random(rng, deck) {
            let effects = deck[card_idx].effects.clone();

            let (mut atk_draws, mut def_draws, mut res_draws) = (0u32, 0u32, 0u32);
            for effect in &effects {
                if let Some(types::CardEffectKind::DrawCards {
                    attack,
                    defence,
                    resource,
                }) = self.library.resolve_effect(effect.effect_id)
                {
                    atk_draws += attack;
                    def_draws += defence;
                    res_draws += resource;
                }
            }

            apply_card_effects(
                &effects,
                false,
                &mut self.token_balances,
                combat,
                &self.library,
            );
            check_combat_end(&self.token_balances, combat);

            // Handle enemy draws per deck type
            if combat.outcome == EncounterOutcome::Undecided {
                Self::enemy_draw_n(rng, &mut combat.enemy_attack_deck, atk_draws);
                Self::enemy_draw_n(rng, &mut combat.enemy_defence_deck, def_draws);
                Self::enemy_draw_n(rng, &mut combat.enemy_resource_deck, res_draws);
            }

            if combat.outcome != EncounterOutcome::Undecided {
                if combat.outcome == EncounterOutcome::PlayerWon {
                    let entry = types::token_entry_by_type(
                        &mut self.token_balances,
                        &types::TokenType::MilestoneInsight,
                    );
                    *entry += 100;
                }
                let outcome = combat.outcome.clone();
                let rounds = combat.round;
                self.record_encounter_finish(types::Discipline::Combat, outcome, rounds);
                self.current_encounter = None;
                self.encounter_phase = types::EncounterPhase::Scouting;
                self.check_player_death();
            }
        }
        Ok(())
    }

    /// Draw `count` random cards from a single enemy deck to hand, recycling discard if needed.
    fn enemy_draw_n(rng: &mut rand_pcg::Lcg64Xsh32, deck: &mut [types::EnemyCardDef], count: u32) {
        for _ in 0..count {
            crate::library::game_state::deck_draw_random(rng, deck);
        }
    }

    /// Advance combat phase to next (Defending → Attacking → Resourcing → Defending).
    pub fn advance_combat_phase(&mut self) -> Result<(), String> {
        let combat = match &mut self.current_encounter {
            Some(EncounterState::Combat(c)) => c,
            _ => return Err("No active combat".to_string()),
        };
        combat.phase = combat.phase.next();
        Ok(())
    }
}
