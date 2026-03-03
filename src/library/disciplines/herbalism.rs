use crate::library::types::{
    self, CardCounts, CardKind, EncounterKind, EncounterOutcome, EncounterState,
    HerbalismEncounterState,
};
use crate::library::{GameState, Library};
use std::collections::HashMap;

pub(crate) fn register_herbalism_cards(lib: &mut Library, _rng: &mut rand_pcg::Lcg64Xsh32) {
    // Narrow herbalism card: targets 1 characteristic, low durability cost
    lib.add_card(
        CardKind::Herbalism {
            herbalism_effect: types::HerbalismCardEffect {
                costs: vec![types::GatheringCost {
                    cost_type: types::TokenType::HerbalismDurability,
                    amount: 100,
                }],
                match_mode: types::HerbalismMatchMode::Or {
                    types: vec![types::PlantCharacteristic::Fragile],
                },
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

    // Medium herbalism card: targets 2 characteristics
    lib.add_card(
        CardKind::Herbalism {
            herbalism_effect: types::HerbalismCardEffect {
                costs: vec![types::GatheringCost {
                    cost_type: types::TokenType::HerbalismDurability,
                    amount: 100,
                }],
                match_mode: types::HerbalismMatchMode::Or {
                    types: vec![
                        types::PlantCharacteristic::Thorny,
                        types::PlantCharacteristic::Aromatic,
                    ],
                },
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

    // Broad herbalism card: targets 3 characteristics
    lib.add_card(
        CardKind::Herbalism {
            herbalism_effect: types::HerbalismCardEffect {
                costs: vec![types::GatheringCost {
                    cost_type: types::TokenType::HerbalismDurability,
                    amount: 100,
                }],
                match_mode: types::HerbalismMatchMode::Or {
                    types: vec![
                        types::PlantCharacteristic::Bitter,
                        types::PlantCharacteristic::Luminous,
                        types::PlantCharacteristic::Fragile,
                    ],
                },
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

    // Herbalism encounter: Meadow Herb
    lib.add_card(
        CardKind::Encounter {
            encounter_kind: types::EncounterKind::Herbalism {
                herbalism_def: types::HerbalismDef {
                    plant_hand: vec![
                        types::PlantCard {
                            characteristics: vec![types::PlantCharacteristic::Fragile],
                            counts: types::DeckCounts {
                                deck: 0,
                                hand: 1,
                                discard: 0,
                            },
                        },
                        types::PlantCard {
                            characteristics: vec![
                                types::PlantCharacteristic::Thorny,
                                types::PlantCharacteristic::Aromatic,
                            ],
                            counts: types::DeckCounts {
                                deck: 0,
                                hand: 1,
                                discard: 0,
                            },
                        },
                        types::PlantCard {
                            characteristics: vec![
                                types::PlantCharacteristic::Bitter,
                                types::PlantCharacteristic::Luminous,
                            ],
                            counts: types::DeckCounts {
                                deck: 0,
                                hand: 1,
                                discard: 0,
                            },
                        },
                        types::PlantCard {
                            characteristics: vec![
                                types::PlantCharacteristic::Fragile,
                                types::PlantCharacteristic::Thorny,
                            ],
                            counts: types::DeckCounts {
                                deck: 0,
                                hand: 1,
                                discard: 0,
                            },
                        },
                        types::PlantCard {
                            characteristics: vec![types::PlantCharacteristic::Luminous],
                            counts: types::DeckCounts {
                                deck: 0,
                                hand: 1,
                                discard: 0,
                            },
                        },
                    ],
                    rewards: HashMap::from([(
                        types::Token::persistent(types::TokenType::Plant),
                        500,
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

    // MostCommon card — removes the most common characteristic (limit 1)
    lib.add_card(
        CardKind::Herbalism {
            herbalism_effect: types::HerbalismCardEffect {
                costs: vec![
                    types::GatheringCost {
                        cost_type: types::TokenType::Stamina,
                        amount: 150,
                    },
                    types::GatheringCost {
                        cost_type: types::TokenType::HerbalismDurability,
                        amount: 100,
                    },
                ],
                match_mode: types::HerbalismMatchMode::MostCommon {
                    limit: 1,
                    types: vec![],
                },
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

    // LeastCommon card — removes the least common characteristic (limit 1)
    lib.add_card(
        CardKind::Herbalism {
            herbalism_effect: types::HerbalismCardEffect {
                costs: vec![
                    types::GatheringCost {
                        cost_type: types::TokenType::Stamina,
                        amount: 150,
                    },
                    types::GatheringCost {
                        cost_type: types::TokenType::HerbalismDurability,
                        amount: 100,
                    },
                ],
                match_mode: types::HerbalismMatchMode::LeastCommon {
                    limit: 1,
                    types: vec![],
                },
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

    // AND-based multi-type card — removes only plants matching ALL listed types
    lib.add_card(
        CardKind::Herbalism {
            herbalism_effect: types::HerbalismCardEffect {
                costs: vec![
                    types::GatheringCost {
                        cost_type: types::TokenType::Stamina,
                        amount: 100,
                    },
                    types::GatheringCost {
                        cost_type: types::TokenType::HerbalismDurability,
                        amount: 100,
                    },
                ],
                match_mode: types::HerbalismMatchMode::And {
                    types: vec![
                        types::PlantCharacteristic::Fragile,
                        types::PlantCharacteristic::Thorny,
                    ],
                },
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

    // Stamina rest card for herbalism
    lib.add_card(
        CardKind::Herbalism {
            herbalism_effect: types::HerbalismCardEffect {
                costs: vec![types::GatheringCost {
                    cost_type: types::TokenType::HerbalismDurability,
                    amount: 50,
                }],
                match_mode: types::HerbalismMatchMode::Or { types: vec![] },
                gains: vec![types::GatheringCost {
                    cost_type: types::TokenType::Stamina,
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
        Self::plant_shuffle_hand(rng, &mut plant_hand);
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
        let herbalism_effect = match &lib_card.kind {
            CardKind::Herbalism { herbalism_effect } => herbalism_effect.clone(),
            _ => return Err("Cannot play a non-herbalism card in herbalism encounter".to_string()),
        };

        // Split costs into pre-play (reject if unaffordable) and post-play (durability)
        let (pre_play_costs, post_play_costs) =
            types::split_gathering_costs(&herbalism_effect.costs);
        if !pre_play_costs.is_empty() {
            Self::check_and_deduct_gathering_costs(&pre_play_costs, &mut self.token_balances)?;
        }

        // Apply durability costs (depletes encounter, doesn't reject card)
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
            self.finish_herbalism_encounter(false);
            return Ok(());
        }

        // Apply gains
        for gain in &herbalism_effect.gains {
            let entry = types::token_entry_by_type(&mut self.token_balances, &gain.cost_type);
            *entry += gain.amount;
        }

        // Remove plant cards based on match mode
        {
            let herbalism = match &mut self.current_encounter {
                Some(EncounterState::Herbalism(h)) => h,
                _ => return Err("No active herbalism encounter".to_string()),
            };

            match &herbalism_effect.match_mode {
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
        self.all_gathering_hand_cards_unpayable(|k| match k {
            CardKind::Herbalism { herbalism_effect } => Some(&herbalism_effect.costs),
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
        self.last_encounter_result = Some(outcome.clone());
        self.encounter_results.push(outcome);
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

    /// Shuffle plant hand: move all to deck, redraw to original hand size.
    fn plant_shuffle_hand(rng: &mut rand_pcg::Lcg64Xsh32, deck: &mut [types::PlantCard]) {
        let target_hand: u32 = deck.iter().map(|c| c.counts.hand).sum();
        for card in deck.iter_mut() {
            card.counts.deck += card.counts.hand;
            card.counts.hand = 0;
        }
        for _ in 0..target_hand {
            Self::plant_draw_random(rng, deck);
        }
    }

    /// Draw one random plant card from deck to hand, recycling discard if needed.
    fn plant_draw_random(rng: &mut rand_pcg::Lcg64Xsh32, deck: &mut [types::PlantCard]) {
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
