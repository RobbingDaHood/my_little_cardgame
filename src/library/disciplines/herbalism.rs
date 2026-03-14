use crate::library::types::{
    self, CardEffectKind, CardKind, EncounterKind, EncounterOutcome, EncounterState,
    HerbalismEncounterState,
};
use crate::library::GameState;

impl GameState {
    /// Initialize an herbalism gathering encounter from a Library Encounter card.
    pub fn start_herbalism_encounter(
        &mut self,
        encounter_card_id: usize,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let lib_card = self
            .library
            .get(encounter_card_id)
            .ok_or_else(|| format!("Card {} not found in Library", encounter_card_id))?
            .clone();
        let herbalism_def = match &lib_card.kind {
            CardKind::Encounter {
                encounter_kind: EncounterKind::Herbalism { herbalism_def },
            } => herbalism_def.clone(),
            _ => {
                return Err(format!(
                    "Card {} is not an herbalism encounter",
                    encounter_card_id
                ))
            }
        };
        let mut plant_hand = herbalism_def.plant_hand;
        crate::library::game_state::deck_shuffle_hand(rng, &mut plant_hand);
        let state = HerbalismEncounterState {
            round: 1,
            encounter_card_id,
            outcome: EncounterOutcome::Undecided,
            plant_hand,
            rewards: herbalism_def.rewards,
        };
        self.current_encounter = Some(EncounterState::Herbalism(state));
        self.encounter_phase = types::EncounterPhase::InEncounter;
        Ok(())
    }

    /// Resolve a player herbalism card play against the current herbalism encounter.
    /// Applies durability cost, removes plant cards sharing ≥1 characteristic,
    /// checks win (exactly 1 remaining) / loss (0 remaining or durability depleted).
    pub fn resolve_player_herbalism_card(
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
            CardKind::Herbalism { effects } => effects.clone(),
            _ => return Err("Cannot play a non-herbalism card in herbalism encounter".to_string()),
        };

        // Extract costs from rolled_costs on effects and split into pre/post-play
        let costs = Self::extract_gathering_costs_from_effects(&effects);
        let (pre_play_costs, post_play_costs) = types::split_token_amounts(&costs);
        if !pre_play_costs.is_empty() {
            Self::check_and_deduct_gathering_costs(&pre_play_costs, &mut self.token_balances)?;
        }

        // Apply durability costs (depletes encounter, doesn't reject card)
        let mut durability_depleted = false;
        for cost in &post_play_costs {
            let resolved_type = cost
                .token_type
                .resolve_durability(&types::Discipline::Herbalism);
            let key = types::Token::persistent(resolved_type);
            let durability = self.token_balances.entry(key).or_insert(0);
            *durability = (*durability - cost.amount).max(0);
            if *durability <= 0 {
                durability_depleted = true;
            }
        }

        if durability_depleted {
            self.finish_herbalism_encounter(false);
            return Ok(());
        }

        // Process all effects via library templates
        let mut match_mode_opt: Option<types::HerbalismMatchMode> = None;
        for effect in &effects {
            let kind = match self.library.resolve_effect(effect.effect_id) {
                Some(resolved) => resolved,
                None => continue,
            };
            match &kind {
                CardEffectKind::GainTokens { token_type, .. } => {
                    let entry = types::token_entry_by_type(&mut self.token_balances, token_type);
                    *entry += effect.rolled_value;
                }
                CardEffectKind::Insight { .. } => {
                    let insight_type =
                        types::TokenType::insight_for_discipline(&types::Discipline::Herbalism);
                    let entry = types::token_entry_by_type(&mut self.token_balances, &insight_type);
                    *entry += effect.rolled_value;
                }
                CardEffectKind::HerbalismMatch { match_mode, .. } => {
                    match_mode_opt = Some(match_mode.clone());
                }
                // Costs handled via rolled_costs above
                _ => {}
            }
        }

        // Remove plant cards based on match mode
        if let Some(match_mode) = match_mode_opt {
            let herbalism = match &mut self.current_encounter {
                Some(EncounterState::Herbalism(h)) => h,
                _ => return Err("No active herbalism encounter".to_string()),
            };

            match &match_mode {
                types::HerbalismMatchMode::Or {
                    types: target_types,
                } => {
                    for plant_card in &mut herbalism.plant_hand {
                        if plant_card.counts.hand == 0 {
                            continue;
                        }
                        let shares_characteristic = plant_card
                            .characteristics
                            .iter()
                            .any(|c| target_types.contains(c));
                        if shares_characteristic {
                            plant_card.counts.hand = 0;
                        }
                    }
                }
                types::HerbalismMatchMode::And {
                    types: target_types,
                } => {
                    for plant_card in &mut herbalism.plant_hand {
                        if plant_card.counts.hand == 0 {
                            continue;
                        }
                        let has_all = target_types
                            .iter()
                            .all(|c| plant_card.characteristics.contains(c));
                        if has_all {
                            plant_card.counts.hand = 0;
                        }
                    }
                }
                types::HerbalismMatchMode::MostCommon { limit, .. } => {
                    let targets = Self::herbalism_most_common_characteristics(
                        &herbalism.plant_hand,
                        rng,
                        *limit,
                    );
                    for plant_card in &mut herbalism.plant_hand {
                        if plant_card.counts.hand == 0 {
                            continue;
                        }
                        let shares = plant_card
                            .characteristics
                            .iter()
                            .any(|c| targets.contains(c));
                        if shares {
                            plant_card.counts.hand = 0;
                        }
                    }
                }
                types::HerbalismMatchMode::LeastCommon { limit, .. } => {
                    let targets = Self::herbalism_least_common_characteristics(
                        &herbalism.plant_hand,
                        rng,
                        *limit,
                    );
                    for plant_card in &mut herbalism.plant_hand {
                        if plant_card.counts.hand == 0 {
                            continue;
                        }
                        let shares = plant_card
                            .characteristics
                            .iter()
                            .any(|c| targets.contains(c));
                        if shares {
                            plant_card.counts.hand = 0;
                        }
                    }
                }
            }

            herbalism.round += 1;
        }

        // Check win/loss based on remaining plant cards
        let remaining = match &self.current_encounter {
            Some(EncounterState::Herbalism(h)) => {
                h.plant_hand.iter().filter(|c| c.counts.hand > 0).count()
            }
            _ => return Err("No active herbalism encounter".to_string()),
        };

        if remaining == 1 {
            self.finish_herbalism_encounter(true);
        } else if remaining == 0 {
            self.finish_herbalism_encounter(false);
        } else {
            // Draw 1 herbalism card for player
            self.draw_player_herbalism_card(rng);

            // Check autoloss: if all herbalism hand cards are unpayable, player loses
            if self.current_encounter.is_some() && self.all_herbalism_hand_cards_unpayable() {
                self.finish_herbalism_encounter(false);
            }
        }

        Ok(())
    }

    /// Check if all herbalism hand cards are unpayable (pre-play costs unaffordable).
    fn all_herbalism_hand_cards_unpayable(&self) -> bool {
        self.all_effects_hand_cards_unpayable(|k| match k {
            CardKind::Herbalism { effects } => Some(effects),
            _ => None,
        })
    }

    fn herbalism_most_common_characteristics(
        plant_hand: &[types::PlantCard],
        rng: &mut rand_pcg::Lcg64Xsh32,
        limit: u32,
    ) -> Vec<types::PlantCharacteristic> {
        use rand::RngCore;
        use std::collections::HashMap;
        let mut counts: HashMap<types::PlantCharacteristic, u32> = HashMap::new();
        for card in plant_hand {
            if card.counts.hand == 0 {
                continue;
            }
            for c in &card.characteristics {
                *counts.entry(c.clone()).or_insert(0) += 1;
            }
        }
        let mut sorted: Vec<_> = counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| (rng.next_u64() % 2).cmp(&1)));
        sorted
            .into_iter()
            .take(limit as usize)
            .map(|(c, _)| c)
            .collect()
    }

    fn herbalism_least_common_characteristics(
        plant_hand: &[types::PlantCard],
        rng: &mut rand_pcg::Lcg64Xsh32,
        limit: u32,
    ) -> Vec<types::PlantCharacteristic> {
        use rand::RngCore;
        use std::collections::HashMap;
        let mut counts: HashMap<types::PlantCharacteristic, u32> = HashMap::new();
        for card in plant_hand {
            if card.counts.hand == 0 {
                continue;
            }
            for c in &card.characteristics {
                *counts.entry(c.clone()).or_insert(0) += 1;
            }
        }
        let mut sorted: Vec<_> = counts.into_iter().collect();
        sorted.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| (rng.next_u64() % 2).cmp(&1)));
        sorted
            .into_iter()
            .take(limit as usize)
            .map(|(c, _)| c)
            .collect()
    }

    /// Conclude an herbalism encounter voluntarily: grant rewards if any accumulated.
    pub fn conclude_herbalism_encounter(&mut self) -> Result<(), String> {
        match &self.current_encounter {
            Some(EncounterState::Herbalism(h)) if h.outcome == EncounterOutcome::Undecided => {
                if h.rewards.values().all(|&v| v <= 0) {
                    return Err("No rewards accumulated; abort the encounter instead".to_string());
                }
            }
            _ => return Err("No active herbalism encounter to conclude".to_string()),
        }
        self.finish_herbalism_encounter(true);
        Ok(())
    }

    fn finish_herbalism_encounter(&mut self, is_win: bool) {
        if is_win {
            let rewards = match &self.current_encounter {
                Some(EncounterState::Herbalism(h)) => h.rewards.clone(),
                _ => return,
            };
            for (token, amount) in &rewards {
                let entry = self.token_balances.entry(token.clone()).or_insert(0);
                *entry += amount;
            }
        }
        let outcome = if is_win {
            EncounterOutcome::PlayerWon
        } else {
            EncounterOutcome::PlayerLost
        };
        let rounds = match &self.current_encounter {
            Some(EncounterState::Herbalism(h)) => h.round,
            _ => 0,
        };
        self.record_encounter_finish(types::Discipline::Herbalism, outcome, rounds);
        self.current_encounter = None;
        self.encounter_phase = types::EncounterPhase::Scouting;
    }

    /// Draw one player herbalism card from deck to hand, recycling discard if needed.
    fn draw_player_herbalism_card(&mut self, rng: &mut rand_pcg::Lcg64Xsh32) {
        self.draw_player_cards_of_kind(
            1,
            |k| matches!(k, CardKind::Herbalism { .. }),
            rng,
            Some(types::TokenType::HerbalismMaxHand),
        );
    }
}
