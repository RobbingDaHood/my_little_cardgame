use crate::library::types::{
    self, CardCounts, CardKind, EncounterKind, EncounterOutcome, EncounterState,
};
use crate::library::{GameState, Library};
use std::collections::HashMap;

pub(crate) fn register_woodcutting_cards(lib: &mut Library, _rng: &mut rand_pcg::Lcg64Xsh32) {
    // LightChop card: value 2
    lib.add_card(
        CardKind::Woodcutting {
            woodcutting_effect: types::WoodcuttingCardEffect {
                chop_types: vec![types::ChopType::LightChop],
                chop_values: vec![2],
                costs: vec![types::TokenAmount {
                    token_type: types::TokenType::WoodcuttingDurability,
                    amount: 100,
                }],
                gains: vec![],
            },
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 2,
            discard: 0,
        },
    );

    // HeavyChop card: value 5
    lib.add_card(
        CardKind::Woodcutting {
            woodcutting_effect: types::WoodcuttingCardEffect {
                chop_types: vec![types::ChopType::HeavyChop],
                chop_values: vec![5],
                costs: vec![types::TokenAmount {
                    token_type: types::TokenType::WoodcuttingDurability,
                    amount: 100,
                }],
                gains: vec![],
            },
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 1,
            discard: 0,
        },
    );

    // MediumChop card: value 3
    lib.add_card(
        CardKind::Woodcutting {
            woodcutting_effect: types::WoodcuttingCardEffect {
                chop_types: vec![types::ChopType::MediumChop],
                chop_values: vec![3],
                costs: vec![types::TokenAmount {
                    token_type: types::TokenType::WoodcuttingDurability,
                    amount: 100,
                }],
                gains: vec![],
            },
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 1,
            discard: 0,
        },
    );

    // PrecisionChop card: value 7
    lib.add_card(
        CardKind::Woodcutting {
            woodcutting_effect: types::WoodcuttingCardEffect {
                chop_types: vec![types::ChopType::PrecisionChop],
                chop_values: vec![7],
                costs: vec![types::TokenAmount {
                    token_type: types::TokenType::WoodcuttingDurability,
                    amount: 100,
                }],
                gains: vec![],
            },
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 1,
            discard: 0,
        },
    );

    // Woodcutting encounter: Oak Tree
    lib.add_card(
        CardKind::Encounter {
            encounter_kind: types::EncounterKind::Woodcutting {
                woodcutting_def: types::WoodcuttingDef {
                    max_plays: 8,
                    base_rewards: HashMap::from([(
                        types::Token::persistent(types::TokenType::Lumber),
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

    // Cost Woodcutting card: HeavyChop+LightChop combo, costs stamina
    lib.add_card(
        CardKind::Woodcutting {
            woodcutting_effect: types::WoodcuttingCardEffect {
                chop_types: vec![types::ChopType::HeavyChop, types::ChopType::LightChop],
                chop_values: vec![5, 3],
                costs: vec![
                    types::TokenAmount {
                        token_type: types::TokenType::Stamina,
                        amount: 100,
                    },
                    types::TokenAmount {
                        token_type: types::TokenType::WoodcuttingDurability,
                        amount: 100,
                    },
                ],
                gains: vec![],
            },
        },
        CardCounts {
            library: 0,
            deck: 5,
            hand: 1,
            discard: 0,
        },
    );

    // SplitChop value 4
    lib.add_card(
        CardKind::Woodcutting {
            woodcutting_effect: types::WoodcuttingCardEffect {
                chop_types: vec![types::ChopType::SplitChop],
                chop_values: vec![4],
                costs: vec![types::TokenAmount {
                    token_type: types::TokenType::WoodcuttingDurability,
                    amount: 100,
                }],
                gains: vec![],
            },
        },
        CardCounts {
            library: 0,
            deck: 10,
            hand: 1,
            discard: 0,
        },
    );

    // LightChop+MediumChop, values 1,6
    lib.add_card(
        CardKind::Woodcutting {
            woodcutting_effect: types::WoodcuttingCardEffect {
                chop_types: vec![types::ChopType::LightChop, types::ChopType::MediumChop],
                chop_values: vec![1, 6],
                costs: vec![types::TokenAmount {
                    token_type: types::TokenType::WoodcuttingDurability,
                    amount: 100,
                }],
                gains: vec![],
            },
        },
        CardCounts {
            library: 0,
            deck: 5,
            hand: 0,
            discard: 0,
        },
    );

    // Cost card: 3 types, 3 values, moderate stamina cost
    lib.add_card(
        CardKind::Woodcutting {
            woodcutting_effect: types::WoodcuttingCardEffect {
                chop_types: vec![
                    types::ChopType::HeavyChop,
                    types::ChopType::MediumChop,
                    types::ChopType::PrecisionChop,
                ],
                chop_values: vec![3, 5, 7],
                costs: vec![
                    types::TokenAmount {
                        token_type: types::TokenType::Stamina,
                        amount: 150,
                    },
                    types::TokenAmount {
                        token_type: types::TokenType::WoodcuttingDurability,
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

    // Cost card: 4 types, 4 values, higher stamina cost
    lib.add_card(
        CardKind::Woodcutting {
            woodcutting_effect: types::WoodcuttingCardEffect {
                chop_types: vec![
                    types::ChopType::LightChop,
                    types::ChopType::HeavyChop,
                    types::ChopType::MediumChop,
                    types::ChopType::SplitChop,
                ],
                chop_values: vec![2, 4, 6, 8],
                costs: vec![
                    types::TokenAmount {
                        token_type: types::TokenType::Stamina,
                        amount: 250,
                    },
                    types::TokenAmount {
                        token_type: types::TokenType::WoodcuttingDurability,
                        amount: 100,
                    },
                ],
                gains: vec![],
            },
        },
        CardCounts {
            library: 0,
            deck: 2,
            hand: 0,
            discard: 0,
        },
    );

    // Woodcutting rest card: grants stamina, no chops
    lib.add_card(
        CardKind::Woodcutting {
            woodcutting_effect: types::WoodcuttingCardEffect {
                chop_types: vec![],
                chop_values: vec![],
                costs: vec![types::TokenAmount {
                    token_type: types::TokenType::WoodcuttingDurability,
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
            hand: 0,
            discard: 0,
        },
    );
}

/// Evaluate played woodcutting cards and return (pattern_name, reward_multiplier).
/// Poker-inspired patterns adapted for 8 cards using ChopType counts.
fn evaluate_best_pattern(played: &[types::PlayedWoodcuttingCard]) -> (String, f64) {
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

    // Evaluate patterns from best to worst
    // Eight of a Kind: all 8 cards same type
    if max_type_count >= 8 {
        return ("Eight of a Kind".to_string(), 55.0);
    }
    // Seven of a Kind
    if max_type_count >= 7 {
        return ("Seven of a Kind".to_string(), 12.0);
    }
    // Perfect Straight: 8 sequential values
    if longest_straight >= 8 {
        return ("Perfect Straight".to_string(), 3.0);
    }
    // Six of a Kind
    if max_type_count >= 6 {
        return ("Six of a Kind".to_string(), 4.0);
    }
    // Long Straight: 6-7 sequential values
    if longest_straight >= 6 {
        return ("Long Straight".to_string(), 1.5);
    }
    // Full Set: all 5 chop types present
    if distinct_types >= 5 {
        return ("Full Set".to_string(), 1.0);
    }
    // Five of a Kind
    if max_type_count >= 5 {
        return ("Five of a Kind".to_string(), 2.0);
    }
    // Four of a Kind with Pair: 4+ of one type plus 2+ of another
    if max_type_count >= 4 && sorted_type_counts.len() >= 2 && sorted_type_counts[1] >= 2 {
        return ("Full House".to_string(), 1.5);
    }
    // Four of a Kind
    if max_type_count >= 4 {
        return ("Four of a Kind".to_string(), 10.0);
    }
    // Short Straight: 4-5 sequential values
    if longest_straight >= 4 {
        return ("Short Straight".to_string(), 2.5);
    }
    // Two Pair Types: 2 types with 3+ each
    if sorted_type_counts.len() >= 2 && sorted_type_counts[0] >= 3 && sorted_type_counts[1] >= 3 {
        return ("Two Pair Types".to_string(), 2.0);
    }
    // Value Quads: 4+ of same value
    if freq_list.first().copied().unwrap_or(0) >= 4 {
        return ("Value Quads".to_string(), 12.0);
    }
    // Triple of a Kind
    if max_type_count >= 3 {
        return ("Triple".to_string(), 2.0);
    }
    // Value Triples
    if freq_list.first().copied().unwrap_or(0) >= 3 {
        return ("Value Triple".to_string(), 19.0);
    }
    // Pair (2+ of a type)
    if max_type_count >= 2 {
        return ("Pair".to_string(), 4.5);
    }
    // High Card (fallback)
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
        let woodcutting_effect = match &lib_card.kind {
            CardKind::Woodcutting { woodcutting_effect } => woodcutting_effect.clone(),
            _ => {
                return Err(
                    "Cannot play a non-woodcutting card in woodcutting encounter".to_string(),
                )
            }
        };

        // Split costs into pre-play (reject if unaffordable) and post-play (durability)
        let (pre_play_costs, post_play_costs) =
            types::split_token_amounts(&woodcutting_effect.costs);
        Self::check_and_deduct_gathering_costs(&pre_play_costs, &mut self.token_balances)?;

        // Apply gains
        for gain in &woodcutting_effect.gains {
            let entry = types::token_entry_by_type(&mut self.token_balances, &gain.token_type);
            *entry += gain.amount;
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
            self.finish_woodcutting_encounter(false);
            return Ok(());
        }

        // Track the played card
        let played = types::PlayedWoodcuttingCard {
            card_id,
            chop_types: woodcutting_effect.chop_types,
            chop_values: woodcutting_effect.chop_values,
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

        if all_played {
            // Evaluate pattern and finish as win
            let (pattern_name, multiplier) = {
                let woodcutting = match &self.current_encounter {
                    Some(EncounterState::Woodcutting(w)) => w,
                    _ => return Err("No active woodcutting encounter".to_string()),
                };
                evaluate_best_pattern(&woodcutting.played_cards)
            };
            if let Some(EncounterState::Woodcutting(w)) = &mut self.current_encounter {
                w.pattern_name = Some(pattern_name);
                w.pattern_multiplier = Some(multiplier);
            }
            self.finish_woodcutting_encounter(true);
        } else {
            self.draw_player_woodcutting_card(rng);

            // Check autoloss: if all woodcutting hand cards are unpayable, player loses
            if self.current_encounter.is_some() && self.all_woodcutting_hand_cards_unpayable() {
                self.finish_woodcutting_encounter(false);
            }
        }

        Ok(())
    }

    /// Check if all woodcutting hand cards are unpayable (pre-play costs unaffordable).
    fn all_woodcutting_hand_cards_unpayable(&self) -> bool {
        self.all_gathering_hand_cards_unpayable(|k| match k {
            CardKind::Woodcutting { woodcutting_effect } => Some(&woodcutting_effect.costs),
            _ => None,
        })
    }

    /// Finalize a woodcutting encounter: grant pattern-scaled rewards on win.
    fn finish_woodcutting_encounter(&mut self, is_win: bool) {
        if is_win {
            let (base_rewards, multiplier) = match &self.current_encounter {
                Some(EncounterState::Woodcutting(w)) => {
                    (w.base_rewards.clone(), w.pattern_multiplier.unwrap_or(1.0))
                }
                _ => return,
            };
            for (token, amount) in &base_rewards {
                let scaled = (*amount as f64 * multiplier).round() as i64;
                let entry = self.token_balances.entry(token.clone()).or_insert(0);
                *entry += scaled;
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
