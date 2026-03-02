use crate::library::types::{self, CardKind, EncounterKind, EncounterOutcome, EncounterState};
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
        Self::fish_shuffle_hand(rng, &mut fish_deck);
        // Initialize fishing tokens (encounter-scoped, stored in player token_balances)
        self.token_balances.insert(
            types::Token::persistent(types::TokenType::FishingRangeMin),
            fishing_def.valid_range_min,
        );
        self.token_balances.insert(
            types::Token::persistent(types::TokenType::FishingRangeMax),
            fishing_def.valid_range_max,
        );
        self.token_balances
            .insert(types::Token::persistent(types::TokenType::FishAmount), 1);
        let state = types::FishingEncounterState {
            round: 1,
            encounter_card_id,
            outcome: EncounterOutcome::Undecided,
            turns_won: 0,
            max_turns: fishing_def.max_turns,
            win_turns_needed: fishing_def.win_turns_needed,
            valid_range_min: fishing_def.valid_range_min,
            valid_range_max: fishing_def.valid_range_max,
            fish_deck,
            rewards: fishing_def.rewards,
        };
        self.current_encounter = Some(EncounterState::Fishing(state));
        self.encounter_phase = types::EncounterPhase::InEncounter;
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
        let fishing_effect = match &lib_card.kind {
            CardKind::Fishing { fishing_effect } => fishing_effect.clone(),
            _ => return Err("Cannot play a non-fishing card in fishing encounter".to_string()),
        };

        // Split costs into pre-play (reject if unaffordable) and post-play (durability)
        let (pre_play_costs, post_play_costs) = types::split_gathering_costs(&fishing_effect.costs);
        if !pre_play_costs.is_empty() {
            Self::check_and_deduct_gathering_costs(&pre_play_costs, &mut self.token_balances)?;
        }

        // Deduct durability costs (depletes encounter, doesn't reject card)
        let mut durability_depleted = false;
        for cost in &post_play_costs {
            let key = types::Token::persistent(cost.cost_type.clone());
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

        // Apply gains
        for gain in &fishing_effect.gains {
            let entry = types::token_entry_by_type(&mut self.token_balances, &gain.cost_type);
            *entry += gain.amount;
        }

        // If card has no values, skip the fishing duel (utility-only card)
        if fishing_effect.values.is_empty() {
            // Still advance the round
            let (all_turns_used, enough_wins) = {
                let fishing = match &mut self.current_encounter {
                    Some(EncounterState::Fishing(f)) => f,
                    _ => return Err("No active fishing encounter".to_string()),
                };
                fishing.round += 1;
                let enough_wins = fishing.turns_won >= fishing.win_turns_needed;
                let all_turns_used = (fishing.round - 1) as u32 >= fishing.max_turns;
                (all_turns_used, enough_wins)
            };
            if enough_wins {
                self.finish_fishing_encounter(true);
            } else if all_turns_used {
                self.finish_fishing_encounter(false);
            } else {
                self.draw_player_fishing_card(rng);

                // Check autoloss: if all fishing hand cards are unpayable, player loses
                if self.current_encounter.is_some() && self.all_fishing_hand_cards_unpayable() {
                    self.finish_fishing_encounter(false);
                }
            }
            return Ok(());
        }

        // Read current range from tokens
        let valid_min =
            types::token_balance_by_type(&self.token_balances, &types::TokenType::FishingRangeMin);
        let valid_max =
            types::token_balance_by_type(&self.token_balances, &types::TokenType::FishingRangeMax);

        // Determine fish amount for this turn
        let fish_amount =
            types::token_balance_by_type(&self.token_balances, &types::TokenType::FishAmount)
                .max(1);

        // Auto-resolve fish play: pick random fish card from hand
        let fish_value = Self::fish_play_random(rng, &mut self.current_encounter);

        // Choose the best player value (the one that wins if possible)
        let best_value = fishing_effect
            .values
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
            .unwrap_or(fishing_effect.values[0]);

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
                fishing.turns_won += fish_amount as u32;
            }
            fishing.round += 1;
            // Sync range fields from tokens for display
            fishing.valid_range_min = valid_min;
            fishing.valid_range_max = valid_max;
            let enough_wins = fishing.turns_won >= win_turns_needed;
            let all_turns_used = (fishing.round - 1) as u32 >= fishing.max_turns;
            (all_turns_used, enough_wins)
        };

        if enough_wins {
            self.finish_fishing_encounter(true);
        } else if all_turns_used {
            self.finish_fishing_encounter(false);
        } else {
            self.draw_player_fishing_card(rng);

            // Check autoloss: if all fishing hand cards are unpayable, player loses
            if self.current_encounter.is_some() && self.all_fishing_hand_cards_unpayable() {
                self.finish_fishing_encounter(false);
            }
        }

        Ok(())
    }

    /// Check if all fishing hand cards are unpayable (pre-play costs unaffordable).
    fn all_fishing_hand_cards_unpayable(&self) -> bool {
        let hand_cards: Vec<_> = self
            .library
            .cards
            .iter()
            .filter(|c| c.counts.hand > 0 && matches!(c.kind, CardKind::Fishing { .. }))
            .collect();
        if hand_cards.is_empty() {
            return false;
        }
        hand_cards.iter().all(|card| {
            let costs = match &card.kind {
                CardKind::Fishing { fishing_effect } => &fishing_effect.costs,
                _ => return false,
            };
            let (pre_play_costs, _) = types::split_gathering_costs(costs);
            if pre_play_costs.is_empty() {
                return false;
            }
            Self::preview_gathering_costs(&pre_play_costs, &self.token_balances).is_err()
        })
    }

    fn fish_play_random(
        rng: &mut rand_pcg::Lcg64Xsh32,
        encounter: &mut Option<EncounterState>,
    ) -> i64 {
        use rand::RngCore;
        let fish_deck = match encounter {
            Some(EncounterState::Fishing(f)) => &mut f.fish_deck,
            _ => return 0,
        };
        let total_hand: u32 = fish_deck.iter().map(|c| c.counts.hand).sum();
        if total_hand == 0 {
            // Recycle discard to hand
            let total_discard: u32 = fish_deck.iter().map(|c| c.counts.discard).sum();
            if total_discard == 0 {
                return 0;
            }
            for card in fish_deck.iter_mut() {
                card.counts.hand += card.counts.discard;
                card.counts.discard = 0;
            }
        }
        let total_hand: u32 = fish_deck.iter().map(|c| c.counts.hand).sum();
        if total_hand == 0 {
            return 0;
        }
        let mut pick = (rng.next_u64() as u32) % total_hand;
        for card in fish_deck.iter_mut() {
            if pick < card.counts.hand {
                card.counts.hand -= 1;
                card.counts.discard += 1;
                return card.value;
            }
            pick -= card.counts.hand;
        }
        0
    }

    fn finish_fishing_encounter(&mut self, is_win: bool) {
        if is_win {
            let rewards = match &self.current_encounter {
                Some(EncounterState::Fishing(f)) => f.rewards.clone(),
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
        self.last_encounter_result = Some(outcome.clone());
        self.encounter_results.push(outcome);
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

    fn fish_shuffle_hand(rng: &mut rand_pcg::Lcg64Xsh32, deck: &mut [types::FishCard]) {
        let target_hand: u32 = deck.iter().map(|c| c.counts.hand).sum();
        for card in deck.iter_mut() {
            card.counts.deck += card.counts.hand;
            card.counts.hand = 0;
        }
        for _ in 0..target_hand {
            Self::fish_draw_random(rng, deck);
        }
    }

    fn fish_draw_random(rng: &mut rand_pcg::Lcg64Xsh32, deck: &mut [types::FishCard]) {
        use rand::RngCore;
        let total_deck: u32 = deck.iter().map(|c| c.counts.deck).sum();
        if total_deck == 0 {
            let total_discard: u32 = deck.iter().map(|c| c.counts.discard).sum();
            if total_discard == 0 {
                return;
            }
            for card in deck.iter_mut() {
                card.counts.deck += card.counts.discard;
                card.counts.discard = 0;
            }
        }
        let total_deck: u32 = deck.iter().map(|c| c.counts.deck).sum();
        if total_deck == 0 {
            return;
        }
        let mut pick = (rng.next_u64() as u32) % total_deck;
        for card in deck.iter_mut() {
            if pick < card.counts.deck {
                card.counts.deck -= 1;
                card.counts.hand += 1;
                return;
            }
            pick -= card.counts.deck;
        }
    }
}
