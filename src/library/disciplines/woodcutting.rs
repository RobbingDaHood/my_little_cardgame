use crate::library::types::{
    self, CardEffectKind, CardKind, EncounterKind, EncounterOutcome, EncounterState,
};
use crate::library::GameState;

/// Evaluate played woodcutting cards and return (pattern_name, reward_multiplier).
/// Poker-inspired patterns evaluated from config in priority order.
fn evaluate_best_pattern(
    played: &[types::PlayedWoodcuttingCard],
    patterns: &[crate::library::config::WoodcuttingPatternRule],
) -> (String, f64) {
    use std::collections::HashMap;
    use types::ChopType;

    // Count occurrences of each chop type
    let mut type_counts: HashMap<&ChopType, usize> = HashMap::new();
    for card in played {
        for ct in &card.chop_types {
            *type_counts.entry(ct).or_insert(0) += 1;
        }
    }

    // Collect all chop values (sorted) for straight detection
    let mut all_values: Vec<u32> = played
        .iter()
        .flat_map(|c| c.chop_values.iter().copied())
        .collect();
    all_values.sort();

    // Count value frequencies for value-based patterns
    let mut value_counts: HashMap<u32, usize> = HashMap::new();
    for &v in &all_values {
        *value_counts.entry(v).or_insert(0) += 1;
    }
    let mut freq_list: Vec<usize> = value_counts.values().copied().collect();
    freq_list.sort_unstable_by(|a, b| b.cmp(a));

    let max_type_count = type_counts.values().copied().max().unwrap_or(0);
    let distinct_types = type_counts.len();
    let longest_straight = longest_consecutive_run(&all_values);

    // Sorted frequency list for value-based patterns
    let mut sorted_type_counts: Vec<usize> = type_counts.values().copied().collect();
    sorted_type_counts.sort_unstable_by(|a, b| b.cmp(a));

    // Evaluate patterns from config (first match wins)
    let second_type_count = sorted_type_counts.get(1).copied().unwrap_or(0);
    let top_value_freq = freq_list.first().copied().unwrap_or(0);

    for pattern in patterns {
        let type_ok = pattern.min_type_count == 0 || max_type_count >= pattern.min_type_count;
        let straight_ok = pattern.min_straight == 0 || longest_straight >= pattern.min_straight;
        let distinct_ok =
            pattern.min_distinct_types == 0 || distinct_types >= pattern.min_distinct_types;
        let second_ok =
            pattern.second_type_min == 0 || second_type_count >= pattern.second_type_min;
        let value_freq_ok = pattern.value_freq_min == 0 || top_value_freq >= pattern.value_freq_min;

        if type_ok && straight_ok && distinct_ok && second_ok && value_freq_ok {
            return (pattern.name.clone(), pattern.multiplier);
        }
    }
    // Fallback if no pattern matches
    ("High Card".to_string(), 1.0)
}

/// Find the longest run of consecutive values in a sorted slice.
fn longest_consecutive_run(sorted_values: &[u32]) -> usize {
    if sorted_values.is_empty() {
        return 0;
    }
    let mut deduped: Vec<u32> = Vec::new();
    for &v in sorted_values {
        if deduped.last() != Some(&v) {
            deduped.push(v);
        }
    }
    let mut best = 1;
    let mut current = 1;
    for i in 1..deduped.len() {
        if deduped[i] == deduped[i - 1] + 1 {
            current += 1;
            if current > best {
                best = current;
            }
        } else {
            current = 1;
        }
    }
    best
}

impl GameState {
    /// Initialize a woodcutting pattern-matching encounter (no enemy deck).
    pub fn start_woodcutting_encounter(
        &mut self,
        encounter_card_id: usize,
        _rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let lib_card = self
            .library
            .get(encounter_card_id)
            .ok_or_else(|| format!("Card {} not found in Library", encounter_card_id))?
            .clone();
        let woodcutting_def = match &lib_card.kind {
            CardKind::Encounter {
                encounter_kind: EncounterKind::Woodcutting { woodcutting_def },
            } => woodcutting_def.clone(),
            _ => {
                return Err(format!(
                    "Card {} is not a woodcutting encounter",
                    encounter_card_id
                ))
            }
        };
        let state = types::WoodcuttingEncounterState {
            round: 1,
            encounter_card_id,
            outcome: EncounterOutcome::Undecided,
            played_cards: Vec::new(),
            max_plays: woodcutting_def.max_plays,
            pattern_name: None,
            pattern_multiplier: None,
            base_rewards: woodcutting_def.base_rewards,
        };
        self.current_encounter = Some(EncounterState::Woodcutting(state));
        self.encounter_phase = types::EncounterPhase::InEncounter;
        Ok(())
    }

    /// Resolve a player woodcutting card play: deduct durability, track card, check completion.
    pub fn resolve_player_woodcutting_card(
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
            CardKind::Woodcutting { effects, .. } => effects.clone(),
            _ => {
                return Err(
                    "Cannot play a non-woodcutting card in woodcutting encounter".to_string(),
                )
            }
        };

        // Extract costs from rolled_costs on effects and split pre/post-play
        let costs = Self::extract_gathering_costs_from_effects(&effects);
        let (pre_play_costs, post_play_costs) = types::split_token_amounts(&costs);
        Self::check_and_deduct_gathering_costs(&pre_play_costs, &mut self.token_balances)?;

        // Process all effects via library templates
        let mut chop_types = Vec::new();
        let mut chop_values = Vec::new();
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
                        types::TokenType::insight_for_discipline(&types::Discipline::Woodcutting);
                    let entry = types::token_entry_by_type(&mut self.token_balances, &insight_type);
                    *entry += effect.rolled_value;
                }
                CardEffectKind::WoodcuttingChop { chop_type, .. } => {
                    chop_types.push(chop_type.clone());
                    chop_values.push(effect.rolled_value as u32);
                }
                // Costs handled via rolled_costs above
                _ => {}
            }
        }

        // Deduct durability costs (depletes encounter, doesn't reject card)
        let mut durability_depleted = false;
        for cost in &post_play_costs {
            let resolved_type = cost
                .token_type
                .resolve_durability(&types::Discipline::Woodcutting);
            let key = types::Token::persistent(resolved_type);
            let durability = self.token_balances.entry(key).or_insert(0);
            *durability = (*durability - cost.amount).max(0);
            if *durability <= 0 {
                durability_depleted = true;
            }
        }

        if durability_depleted {
            self.finish_woodcutting_encounter(false);
            return Ok(());
        }

        // Track the played card
        let played = types::PlayedWoodcuttingCard {
            card_id,
            chop_types,
            chop_values,
        };

        let all_played = {
            let woodcutting = match &mut self.current_encounter {
                Some(EncounterState::Woodcutting(w)) => w,
                _ => return Err("No active woodcutting encounter".to_string()),
            };
            woodcutting.played_cards.push(played);
            woodcutting.round += 1;
            woodcutting.played_cards.len() as u32 >= woodcutting.max_plays
        };

        self.draw_player_woodcutting_card(rng);

        if all_played {
            self.evaluate_and_grant_woodcutting_rewards();
            self.finish_woodcutting_encounter(true);
        } else {
            // Check autoloss: if all woodcutting hand cards are unpayable, player loses
            if self.current_encounter.is_some() && self.all_woodcutting_hand_cards_unpayable() {
                self.finish_woodcutting_encounter(false);
            }
        }

        Ok(())
    }

    /// Check if all woodcutting hand cards are unpayable (pre-play costs unaffordable).
    fn all_woodcutting_hand_cards_unpayable(&self) -> bool {
        self.all_effects_hand_cards_unpayable(|k| match k {
            CardKind::Woodcutting { effects, .. } => Some(effects),
            _ => None,
        })
    }

    /// Conclude a woodcutting encounter voluntarily: evaluate pattern and grant rewards.
    pub fn conclude_woodcutting_encounter(&mut self) -> Result<(), String> {
        match &self.current_encounter {
            Some(EncounterState::Woodcutting(w)) if w.outcome == EncounterOutcome::Undecided => {
                if w.base_rewards.values().all(|&v| v <= 0) {
                    return Err("No rewards accumulated; abort the encounter instead".to_string());
                }
            }
            _ => return Err("No active woodcutting encounter to conclude".to_string()),
        }
        self.evaluate_and_grant_woodcutting_rewards();
        self.finish_woodcutting_encounter(true);
        Ok(())
    }

    /// Evaluate the best pattern from played cards and grant scaled rewards.
    fn evaluate_and_grant_woodcutting_rewards(&mut self) {
        let (pattern_name, multiplier) = match &self.current_encounter {
            Some(EncounterState::Woodcutting(w)) if !w.played_cards.is_empty() => {
                evaluate_best_pattern(&w.played_cards, &self.library.woodcutting_patterns)
            }
            _ => return,
        };
        let base_rewards = match &self.current_encounter {
            Some(EncounterState::Woodcutting(w)) => w.base_rewards.clone(),
            _ => return,
        };
        if let Some(EncounterState::Woodcutting(w)) = &mut self.current_encounter {
            w.pattern_name = Some(pattern_name);
            w.pattern_multiplier = Some(multiplier);
        }
        for (token, amount) in &base_rewards {
            let scaled = (*amount as f64 * multiplier).round() as i64;
            let entry = self.token_balances.entry(token.clone()).or_insert(0);
            *entry += scaled;
        }
    }

    fn finish_woodcutting_encounter(&mut self, is_win: bool) {
        let outcome = if is_win {
            EncounterOutcome::PlayerWon
        } else {
            EncounterOutcome::PlayerLost
        };
        let rounds = match &self.current_encounter {
            Some(EncounterState::Woodcutting(w)) => w.round,
            _ => 0,
        };
        self.record_encounter_finish(types::Discipline::Woodcutting, outcome, rounds);
        self.capture_last_encounter_kind();
        self.current_encounter = None;
        self.encounter_phase = types::EncounterPhase::Scouting;
    }

    /// Draw one player woodcutting card from deck to hand, recycling discard if needed.
    fn draw_player_woodcutting_card(&mut self, rng: &mut rand_pcg::Lcg64Xsh32) {
        self.draw_player_cards_of_kind(
            1,
            |k| matches!(k, CardKind::Woodcutting { .. }),
            rng,
            Some(types::TokenType::WoodcuttingMaxHand),
        );
    }
}
