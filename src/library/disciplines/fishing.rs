use crate::library::types::{
    self, CardEffectKind, CardKind, EncounterKind, EncounterOutcome, EncounterState,
};
use crate::library::GameState;

impl GameState {
    /// Initialize a fishing gathering encounter from a Library Encounter card.
    pub fn start_fishing_encounter(
        &mut self,
        encounter_card_id: usize,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let lib_card = self
            .library
            .get(encounter_card_id)
            .ok_or_else(|| format!("Card {} not found in Library", encounter_card_id))?
            .clone();
        let fishing_def = match &lib_card.kind {
            CardKind::Encounter {
                encounter_kind: EncounterKind::Fishing { fishing_def },
            } => fishing_def.clone(),
            _ => {
                return Err(format!(
                    "Card {} is not a fishing encounter",
                    encounter_card_id
                ))
            }
        };
        let mut fish_deck = fishing_def.fish_deck;
        crate::library::game_state::deck_shuffle_hand(rng, &mut fish_deck);
        // Initialize encounter-scoped tokens
        let mut encounter_tokens = std::collections::HashMap::new();
        encounter_tokens.insert(
            types::Token::persistent(types::TokenType::FishingRangeMin),
            fishing_def.valid_range_min,
        );
        encounter_tokens.insert(
            types::Token::persistent(types::TokenType::FishingRangeMax),
            fishing_def.valid_range_max,
        );
        encounter_tokens.insert(types::Token::persistent(types::TokenType::FishAmount), 1);
        let mut state = types::FishingEncounterState {
            round: 1,
            encounter_card_id,
            outcome: EncounterOutcome::Undecided,
            turns_won: 0,
            max_turns: fishing_def.max_turns,
            win_turns_needed: fishing_def.win_turns_needed,
            valid_range_min: fishing_def.valid_range_min,
            valid_range_max: fishing_def.valid_range_max,
            fish_deck,
            current_fish_value: None,
            rewards: fishing_def.rewards,
            encounter_tokens,
        };
        // Draw the first fish so it is visible before the player acts.
        state.current_fish_value = Self::draw_fish_from_deck(rng, &mut state.fish_deck);
        self.current_encounter = Some(EncounterState::Fishing(state));
        self.encounter_phase = types::EncounterPhase::InEncounter;

        // Refill fishing hand to max before each encounter so that the
        // "last card play has no draw" edge-case doesn't drain the hand
        // over many encounters.
        self.refill_fishing_hand(rng);

        Ok(())
    }

    /// Resolve a player fishing card play: apply effects, check range, track wins.
    pub fn resolve_player_fishing_card(
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
            CardKind::Fishing { effects } => effects.clone(),
            _ => return Err("Cannot play a non-fishing card in fishing encounter".to_string()),
        };

        // Extract costs from rolled_costs on effects and split pre/post-play
        let costs = Self::extract_gathering_costs_from_effects(&effects);
        let (pre_play_costs, post_play_costs) = types::split_token_amounts(&costs);
        if !pre_play_costs.is_empty() {
            Self::check_and_deduct_gathering_costs(&pre_play_costs, &mut self.token_balances)?;
        }

        // Deduct durability costs (depletes encounter, doesn't reject card)
        let mut durability_depleted = false;
        for cost in &post_play_costs {
            let resolved_type = cost
                .token_type
                .resolve_durability(&types::Discipline::Fishing);
            let key = types::Token::persistent(resolved_type);
            let durability = self.token_balances.entry(key).or_insert(0);
            *durability = (*durability - cost.amount).max(0);
            if *durability <= 0 {
                durability_depleted = true;
            }
        }

        if durability_depleted {
            self.finish_fishing_encounter(false);
            return Ok(());
        }

        // Process all effects via library templates
        let mut values: Vec<i64> = Vec::new();
        for effect in &effects {
            let kind = match self.library.resolve_effect(effect.effect_id) {
                Some(resolved) => resolved,
                None => continue,
            };
            match &kind {
                CardEffectKind::GainTokens { token_type, .. } => match token_type {
                    types::TokenType::FishingRangeMin
                    | types::TokenType::FishingRangeMax
                    | types::TokenType::FishAmount => {
                        if let Some(EncounterState::Fishing(f)) = &mut self.current_encounter {
                            let key = types::Token::persistent(token_type.clone());
                            let entry = f.encounter_tokens.entry(key).or_insert(0);
                            *entry += effect.rolled_value;
                        }
                    }
                    _ => {
                        let entry =
                            types::token_entry_by_type(&mut self.token_balances, token_type);
                        *entry += effect.rolled_value;
                    }
                },
                CardEffectKind::Insight { .. } => {
                    let insight_type =
                        types::TokenType::insight_for_discipline(&types::Discipline::Fishing);
                    let entry = types::token_entry_by_type(&mut self.token_balances, &insight_type);
                    *entry += effect.rolled_value;
                }
                CardEffectKind::FishingValue { .. } => {
                    values.push(effect.rolled_value);
                }
                // Costs handled via rolled_costs above
                _ => {}
            }
        }

        // If card has no FishingValue effects, skip the fishing duel (utility-only card).
        // The turn is consumed but no win/loss is recorded — the opportunity cost
        // of not dueling is the only penalty.
        if values.is_empty() {
            let (all_turns_used, enough_wins) = {
                let fishing = match &mut self.current_encounter {
                    Some(EncounterState::Fishing(f)) => f,
                    _ => return Err("No active fishing encounter".to_string()),
                };
                fishing.round += 1;
                let enough_wins = fishing.turns_won >= fishing.win_turns_needed as i32;
                let all_turns_used = (fishing.round - 1) as u32 >= fishing.max_turns;
                (all_turns_used, enough_wins)
            };
            if enough_wins {
                self.grant_fishing_rewards();
                self.finish_fishing_encounter(true);
            } else if all_turns_used {
                self.finish_fishing_encounter(false);
            } else {
                self.draw_player_fishing_card(rng);
                Self::advance_fish_for_next_round(rng, &mut self.current_encounter);

                // Check autoloss: if all fishing hand cards are unpayable, player loses
                if self.current_encounter.is_some() && self.all_fishing_hand_cards_unpayable() {
                    self.finish_fishing_encounter(false);
                }
            }
            return Ok(());
        }

        // Read current range from encounter tokens
        let (valid_min, valid_max) = match &self.current_encounter {
            Some(EncounterState::Fishing(f)) => {
                let min_key = types::Token::persistent(types::TokenType::FishingRangeMin);
                let max_key = types::Token::persistent(types::TokenType::FishingRangeMax);
                (
                    f.encounter_tokens.get(&min_key).copied().unwrap_or(0),
                    f.encounter_tokens.get(&max_key).copied().unwrap_or(0),
                )
            }
            _ => return Err("No active fishing encounter".to_string()),
        };

        // Use the pre-drawn fish value for this round
        let fish_value = match &self.current_encounter {
            Some(EncounterState::Fishing(f)) => f.current_fish_value.unwrap_or(0),
            _ => 0,
        };

        // Choose the best player value (the one that wins if possible)
        let best_value = values
            .iter()
            .filter_map(|&v| {
                let result = (v - fish_value).max(0);
                if result >= valid_min && result <= valid_max {
                    Some((v, result))
                } else {
                    None
                }
            })
            .min_by_key(|&(_, result)| (result - valid_min).abs())
            .map(|(v, _)| v)
            .unwrap_or(values[0]);

        let result = (best_value - fish_value).max(0);
        let win_turns_needed = match &self.current_encounter {
            Some(EncounterState::Fishing(f)) => f.win_turns_needed,
            _ => return Err("No active fishing encounter".to_string()),
        };
        let turn_won = result >= valid_min && result <= valid_max;

        // Update encounter state
        let (all_turns_used, enough_wins) = {
            let fishing = match &mut self.current_encounter {
                Some(EncounterState::Fishing(f)) => f,
                _ => return Err("No active fishing encounter".to_string()),
            };
            if turn_won {
                fishing.turns_won += 1;
            }
            fishing.round += 1;
            // Sync range fields from tokens for display
            fishing.valid_range_min = valid_min;
            fishing.valid_range_max = valid_max;
            let enough_wins = fishing.turns_won >= win_turns_needed as i32;
            let all_turns_used = (fishing.round - 1) as u32 >= fishing.max_turns;
            (all_turns_used, enough_wins)
        };

        if enough_wins {
            self.grant_fishing_rewards();
            self.finish_fishing_encounter(true);
        } else if all_turns_used {
            self.finish_fishing_encounter(false);
        } else {
            self.draw_player_fishing_card(rng);
            Self::advance_fish_for_next_round(rng, &mut self.current_encounter);

            // Check autoloss: if all fishing hand cards are unpayable, player loses
            if self.current_encounter.is_some() && self.all_fishing_hand_cards_unpayable() {
                self.finish_fishing_encounter(false);
            }
        }

        Ok(())
    }

    /// Check if all fishing hand cards are unpayable (pre-play costs unaffordable).
    fn all_fishing_hand_cards_unpayable(&self) -> bool {
        self.all_effects_hand_cards_unpayable(|k| match k {
            CardKind::Fishing { effects } => Some(effects),
            _ => None,
        })
    }

    /// Draw a fish card from the deck, returning its value if one was drawn.
    fn draw_fish_from_deck(
        rng: &mut rand_pcg::Lcg64Xsh32,
        fish_deck: &mut [types::FishCard],
    ) -> Option<i64> {
        crate::library::game_state::deck_play_random(rng, fish_deck).map(|idx| fish_deck[idx].value)
    }

    /// Draw the next fish for the upcoming round and store it in the encounter state.
    fn advance_fish_for_next_round(
        rng: &mut rand_pcg::Lcg64Xsh32,
        encounter: &mut Option<EncounterState>,
    ) {
        if let Some(EncounterState::Fishing(f)) = encounter {
            f.current_fish_value = Self::draw_fish_from_deck(rng, &mut f.fish_deck);
        }
    }

    /// Conclude a fishing encounter voluntarily.
    /// Grants rewards only if the player has accumulated enough wins;
    /// otherwise the encounter counts as a loss.
    pub fn conclude_fishing_encounter(&mut self) -> Result<(), String> {
        let is_win = match &self.current_encounter {
            Some(EncounterState::Fishing(f)) if f.outcome == EncounterOutcome::Undecided => {
                f.turns_won >= f.win_turns_needed as i32
            }
            _ => return Err("No active fishing encounter to conclude".to_string()),
        };
        if is_win {
            self.grant_fishing_rewards();
        }
        self.finish_fishing_encounter(is_win);
        Ok(())
    }

    /// Grant fishing rewards scaled by FishAmount from the encounter's reward table.
    fn grant_fishing_rewards(&mut self) {
        let (rewards, fish_amount) = match &self.current_encounter {
            Some(EncounterState::Fishing(f)) => {
                let amount_key = types::Token::persistent(types::TokenType::FishAmount);
                let amount = f
                    .encounter_tokens
                    .get(&amount_key)
                    .copied()
                    .unwrap_or(1)
                    .max(1);
                (f.rewards.clone(), amount)
            }
            _ => return,
        };
        for (token, base_amount) in &rewards {
            let entry = self.token_balances.entry(token.clone()).or_insert(0);
            *entry += base_amount * fish_amount;
        }
    }

    fn finish_fishing_encounter(&mut self, is_win: bool) {
        let outcome = if is_win {
            EncounterOutcome::PlayerWon
        } else {
            EncounterOutcome::PlayerLost
        };
        let rounds = match &self.current_encounter {
            Some(EncounterState::Fishing(f)) => f.round,
            _ => 0,
        };
        self.record_encounter_finish(types::Discipline::Fishing, outcome, rounds);
        self.capture_last_encounter_kind();
        self.current_encounter = None;
        self.encounter_phase = types::EncounterPhase::Scouting;
    }

    fn draw_player_fishing_card(&mut self, rng: &mut rand_pcg::Lcg64Xsh32) {
        self.draw_player_cards_of_kind(
            1,
            |k| matches!(k, CardKind::Fishing { .. }),
            rng,
            Some(types::TokenType::FishingMaxHand),
        );
    }

    /// Draw fishing cards until the hand reaches FishingMaxHand.
    fn refill_fishing_hand(&mut self, rng: &mut rand_pcg::Lcg64Xsh32) {
        let max_hand =
            types::token_balance_by_type(&self.token_balances, &types::TokenType::FishingMaxHand);
        let current_hand: i64 = self
            .library
            .cards
            .iter()
            .filter(|c| matches!(c.kind, CardKind::Fishing { .. }))
            .map(|c| c.counts.hand as i64)
            .sum();
        let deficit = max_hand - current_hand;
        if deficit > 0 {
            self.draw_player_cards_of_kind(
                deficit as u32,
                |k| matches!(k, CardKind::Fishing { .. }),
                rng,
                Some(types::TokenType::FishingMaxHand),
            );
        }
    }
}
