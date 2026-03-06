use crate::library::types::{
    self, CardCounts, CardKind, EncounterKind, EncounterOutcome, EncounterState,
};
use crate::library::{GameState, Library};
use std::collections::HashMap;

pub(crate) fn register_fishing_cards(lib: &mut Library, _rng: &mut rand_pcg::Lcg64Xsh32) {
    // Low value fishing card
    lib.add_card(
        CardKind::Fishing {
            fishing_effect: types::FishingCardEffect {
                values: vec![200],
                costs: vec![types::TokenAmount {
                    token_type: types::TokenType::FishingDurability,
                    amount: 100,
                }],
                gains: vec![],
            },
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
    );

    // Medium value fishing card
    lib.add_card(
        CardKind::Fishing {
            fishing_effect: types::FishingCardEffect {
                values: vec![400],
                costs: vec![types::TokenAmount {
                    token_type: types::TokenType::FishingDurability,
                    amount: 100,
                }],
                gains: vec![],
            },
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
    );

    // High value fishing card
    lib.add_card(
        CardKind::Fishing {
            fishing_effect: types::FishingCardEffect {
                values: vec![700],
                costs: vec![types::TokenAmount {
                    token_type: types::TokenType::FishingDurability,
                    amount: 100,
                }],
                gains: vec![],
            },
        },
        CardCounts {
            library: 0,
            deck: 10,
            hand: 5,
            discard: 0,
        },
    );

    // Fishing encounter: River Spot
    lib.add_card(
        CardKind::Encounter {
            encounter_kind: types::EncounterKind::Fishing {
                fishing_def: types::FishingDef {
                    valid_range_min: 100,
                    valid_range_max: 300,
                    max_turns: 8,
                    win_turns_needed: 4,
                    fish_deck: vec![
                        types::FishCard {
                            value: 100,
                            counts: types::DeckCounts {
                                deck: 0,
                                hand: 6,
                                discard: 0,
                            },
                        },
                        types::FishCard {
                            value: 300,
                            counts: types::DeckCounts {
                                deck: 0,
                                hand: 6,
                                discard: 0,
                            },
                        },
                        types::FishCard {
                            value: 500,
                            counts: types::DeckCounts {
                                deck: 0,
                                hand: 4,
                                discard: 0,
                            },
                        },
                        types::FishCard {
                            value: 700,
                            counts: types::DeckCounts {
                                deck: 0,
                                hand: 2,
                                discard: 0,
                            },
                        },
                    ],
                    rewards: HashMap::from([(
                        types::Token::persistent(types::TokenType::Fish),
                        1000,
                    )]),
                },
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 3,
            discard: 0,
        },
    );

    // Widen range — reduces min value token
    lib.add_card(
        CardKind::Fishing {
            fishing_effect: types::FishingCardEffect {
                values: vec![],
                costs: vec![types::TokenAmount {
                    token_type: types::TokenType::FishingDurability,
                    amount: 100,
                }],
                gains: vec![types::TokenAmount {
                    token_type: types::TokenType::FishingRangeMin,
                    amount: -150,
                }],
            },
        },
        CardCounts {
            library: 0,
            deck: 5,
            hand: 1,
            discard: 0,
        },
    );

    // Widen range — increases max value token
    lib.add_card(
        CardKind::Fishing {
            fishing_effect: types::FishingCardEffect {
                values: vec![],
                costs: vec![types::TokenAmount {
                    token_type: types::TokenType::FishingDurability,
                    amount: 100,
                }],
                gains: vec![types::TokenAmount {
                    token_type: types::TokenType::FishingRangeMax,
                    amount: 150,
                }],
            },
        },
        CardCounts {
            library: 0,
            deck: 5,
            hand: 1,
            discard: 0,
        },
    );

    // Cost card — narrows range but has multiple values (3 values)
    lib.add_card(
        CardKind::Fishing {
            fishing_effect: types::FishingCardEffect {
                values: vec![100, 350, 600],
                costs: vec![
                    types::TokenAmount {
                        token_type: types::TokenType::Stamina,
                        amount: 150,
                    },
                    types::TokenAmount {
                        token_type: types::TokenType::FishingDurability,
                        amount: 100,
                    },
                ],
                gains: vec![
                    types::TokenAmount {
                        token_type: types::TokenType::FishingRangeMin,
                        amount: 50,
                    },
                    types::TokenAmount {
                        token_type: types::TokenType::FishingRangeMax,
                        amount: -50,
                    },
                ],
            },
        },
        CardCounts {
            library: 0,
            deck: 3,
            hand: 1,
            discard: 0,
        },
    );

    // Increase fish amount
    lib.add_card(
        CardKind::Fishing {
            fishing_effect: types::FishingCardEffect {
                values: vec![],
                costs: vec![types::TokenAmount {
                    token_type: types::TokenType::FishingDurability,
                    amount: 100,
                }],
                gains: vec![types::TokenAmount {
                    token_type: types::TokenType::FishAmount,
                    amount: 1,
                }],
            },
        },
        CardCounts {
            library: 0,
            deck: 3,
            hand: 0,
            discard: 0,
        },
    );

    // Multi-value but decreases fish amount
    lib.add_card(
        CardKind::Fishing {
            fishing_effect: types::FishingCardEffect {
                values: vec![150, 400, 650],
                costs: vec![
                    types::TokenAmount {
                        token_type: types::TokenType::Stamina,
                        amount: 100,
                    },
                    types::TokenAmount {
                        token_type: types::TokenType::FishingDurability,
                        amount: 100,
                    },
                ],
                gains: vec![types::TokenAmount {
                    token_type: types::TokenType::FishAmount,
                    amount: -1,
                }],
            },
        },
        CardCounts {
            library: 0,
            deck: 3,
            hand: 0,
            discard: 0,
        },
    );

    // Rest card — grants stamina, no values
    lib.add_card(
        CardKind::Fishing {
            fishing_effect: types::FishingCardEffect {
                values: vec![],
                costs: vec![types::TokenAmount {
                    token_type: types::TokenType::FishingDurability,
                    amount: 50,
                }],
                gains: vec![types::TokenAmount {
                    token_type: types::TokenType::Stamina,
                    amount: 200,
                }],
            },
        },
        CardCounts {
            library: 0,
            deck: 3,
            hand: 1,
            discard: 0,
        },
    );

    // Stamina cost card with multiple values
    lib.add_card(
        CardKind::Fishing {
            fishing_effect: types::FishingCardEffect {
                values: vec![50, 250, 500, 750],
                costs: vec![
                    types::TokenAmount {
                        token_type: types::TokenType::Stamina,
                        amount: 200,
                    },
                    types::TokenAmount {
                        token_type: types::TokenType::FishingDurability,
                        amount: 100,
                    },
                ],
                gains: vec![],
            },
        },
        CardCounts {
            library: 0,
            deck: 3,
            hand: 0,
            discard: 0,
        },
    );
}

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
        let (pre_play_costs, post_play_costs) = types::split_token_amounts(&fishing_effect.costs);
        if !pre_play_costs.is_empty() {
            Self::check_and_deduct_gathering_costs(&pre_play_costs, &mut self.token_balances)?;
        }

        // Deduct durability costs (depletes encounter, doesn't reject card)
        let mut durability_depleted = false;
        for cost in &post_play_costs {
            let key = types::Token::persistent(cost.token_type.clone());
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
            let entry = types::token_entry_by_type(&mut self.token_balances, &gain.token_type);
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
        self.all_gathering_hand_cards_unpayable(|k| match k {
            CardKind::Fishing { fishing_effect } => Some(&fishing_effect.costs),
            _ => None,
        })
    }

    fn fish_play_random(
        rng: &mut rand_pcg::Lcg64Xsh32,
        encounter: &mut Option<EncounterState>,
    ) -> i64 {
        let fish_deck = match encounter {
            Some(EncounterState::Fishing(f)) => &mut f.fish_deck,
            _ => return 0,
        };
        match crate::library::game_state::deck_play_random(rng, fish_deck) {
            Some(idx) => {
                let fish_deck = match encounter {
                    Some(EncounterState::Fishing(f)) => &f.fish_deck,
                    _ => return 0,
                };
                fish_deck[idx].value
            }
            None => 0,
        }
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
}
