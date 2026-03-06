use crate::library::types::{
    self, CardCounts, CardKind, EncounterKind, EncounterOutcome, EncounterState,
    MiningEncounterState,
};
use crate::library::{GameState, Library};

pub(crate) fn register_mining_cards(lib: &mut Library, _rng: &mut rand_pcg::Lcg64Xsh32) {
    // Mining power card: high power, no cost
    lib.add_card(
        CardKind::Mining {
            mining_effect: types::MiningCardEffect {
                costs: vec![],
                gains: vec![types::TokenAmount {
                    token_type: types::TokenType::MiningPower,
                    amount: 500,
                    cap: None,
                }],
            },
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
    );

    // Balanced mining power card: moderate power
    lib.add_card(
        CardKind::Mining {
            mining_effect: types::MiningCardEffect {
                costs: vec![],
                gains: vec![types::TokenAmount {
                    token_type: types::TokenType::MiningPower,
                    amount: 300,
                    cap: None,
                }],
            },
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
    );

    // Light level card: restores light, costs lumber
    lib.add_card(
        CardKind::Mining {
            mining_effect: types::MiningCardEffect {
                costs: vec![types::TokenAmount {
                    token_type: types::TokenType::Lumber,
                    amount: 15,
                    cap: None,
                }],
                gains: vec![types::TokenAmount {
                    token_type: types::TokenType::MiningLightLevel,
                    amount: 200,
                    cap: Some(500),
                }],
            },
        },
        CardCounts {
            library: 0,
            deck: 10,
            hand: 3,
            discard: 0,
        },
    );

    // Mining encounter: Iron Ore
    lib.add_card(
        CardKind::Encounter {
            encounter_kind: types::EncounterKind::Mining {
                mining_def: types::MiningDef {
                    initial_light_level: 300,
                    ore_deck: vec![
                        types::OreCard {
                            damages: vec![types::TokenAmount {
                                token_type: types::TokenType::MiningLightLevel,
                                amount: 30,
                                cap: None,
                            }],
                            counts: types::DeckCounts {
                                deck: 0,
                                hand: 6,
                                discard: 0,
                            },
                        },
                        types::OreCard {
                            damages: vec![
                                types::TokenAmount {
                                    token_type: types::TokenType::MiningLightLevel,
                                    amount: 50,
                                    cap: None,
                                },
                                types::TokenAmount {
                                    token_type: types::TokenType::MiningDurability,
                                    amount: 100,
                                    cap: None,
                                },
                            ],
                            counts: types::DeckCounts {
                                deck: 0,
                                hand: 8,
                                discard: 0,
                            },
                        },
                        types::OreCard {
                            damages: vec![types::TokenAmount {
                                token_type: types::TokenType::MiningDurability,
                                amount: 200,
                                cap: None,
                            }],
                            counts: types::DeckCounts {
                                deck: 0,
                                hand: 4,
                                discard: 0,
                            },
                        },
                        types::OreCard {
                            damages: vec![types::TokenAmount {
                                token_type: types::TokenType::Health,
                                amount: 75,
                                cap: None,
                            }],
                            counts: types::DeckCounts {
                                deck: 0,
                                hand: 2,
                                discard: 0,
                            },
                        },
                    ],
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

    // High power mining card: costs stamina
    lib.add_card(
        CardKind::Mining {
            mining_effect: types::MiningCardEffect {
                costs: vec![types::TokenAmount {
                    token_type: types::TokenType::Stamina,
                    amount: 100,
                    cap: None,
                }],
                gains: vec![types::TokenAmount {
                    token_type: types::TokenType::MiningPower,
                    amount: 800,
                    cap: None,
                }],
            },
        },
        CardCounts {
            library: 0,
            deck: 5,
            hand: 2,
            discard: 0,
        },
    );

    // High power + high cost
    lib.add_card(
        CardKind::Mining {
            mining_effect: types::MiningCardEffect {
                costs: vec![types::TokenAmount {
                    token_type: types::TokenType::Stamina,
                    amount: 200,
                    cap: None,
                }],
                gains: vec![types::TokenAmount {
                    token_type: types::TokenType::MiningPower,
                    amount: 600,
                    cap: None,
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

    // Very high power, highest cost
    lib.add_card(
        CardKind::Mining {
            mining_effect: types::MiningCardEffect {
                costs: vec![types::TokenAmount {
                    token_type: types::TokenType::Stamina,
                    amount: 300,
                    cap: None,
                }],
                gains: vec![types::TokenAmount {
                    token_type: types::TokenType::MiningPower,
                    amount: 1000,
                    cap: None,
                }],
            },
        },
        CardCounts {
            library: 0,
            deck: 2,
            hand: 0,
            discard: 0,
        },
    );

    // Large light level card: higher gain, higher lumber cost, higher cap
    lib.add_card(
        CardKind::Mining {
            mining_effect: types::MiningCardEffect {
                costs: vec![types::TokenAmount {
                    token_type: types::TokenType::Lumber,
                    amount: 25,
                    cap: None,
                }],
                gains: vec![types::TokenAmount {
                    token_type: types::TokenType::MiningLightLevel,
                    amount: 350,
                    cap: Some(600),
                }],
            },
        },
        CardCounts {
            library: 0,
            deck: 5,
            hand: 0,
            discard: 0,
        },
    );

    // Mining rest card: grants stamina, no power or light
    lib.add_card(
        CardKind::Mining {
            mining_effect: types::MiningCardEffect {
                costs: vec![],
                gains: vec![types::TokenAmount {
                    token_type: types::TokenType::Stamina,
                    amount: 200,
                    cap: None,
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
    /// Initialize a mining gathering encounter from a Library Encounter card.
    pub fn start_mining_encounter(
        &mut self,
        encounter_card_id: usize,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let lib_card = self
            .library
            .get(encounter_card_id)
            .ok_or_else(|| format!("Card {} not found in Library", encounter_card_id))?
            .clone();
        let mining_def = match &lib_card.kind {
            CardKind::Encounter {
                encounter_kind: EncounterKind::Mining { mining_def },
            } => mining_def.clone(),
            _ => {
                return Err(format!(
                    "Card {} is not a mining encounter",
                    encounter_card_id
                ))
            }
        };
        let mut ore_deck = mining_def.ore_deck.clone();
        crate::library::game_state::deck_shuffle_hand(rng, &mut ore_deck);

        // Initialize encounter-scoped tokens
        let light_key = types::Token::persistent(types::TokenType::MiningLightLevel);
        self.token_balances
            .insert(light_key, mining_def.initial_light_level);
        let yield_key = types::Token::persistent(types::TokenType::MiningYield);
        self.token_balances.insert(yield_key, 0);
        let power_key = types::Token::persistent(types::TokenType::MiningPower);
        self.token_balances.insert(power_key, 0);

        let state = MiningEncounterState {
            round: 1,
            encounter_card_id,
            outcome: EncounterOutcome::Undecided,
            ore_deck,
        };
        self.current_encounter = Some(EncounterState::Mining(state));
        self.encounter_phase = types::EncounterPhase::InEncounter;
        Ok(())
    }

    /// Resolve a player mining card play against the current mining encounter.
    /// Processes token-based gains (MiningPower→yield, MiningLightLevel→light with cap),
    /// auto-resolves ore play, draws cards, and checks encounter end.
    pub fn resolve_player_mining_card(
        &mut self,
        card_id: usize,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let lib_card = self
            .library
            .get(card_id)
            .ok_or_else(|| format!("Card {} not found in Library", card_id))?
            .clone();
        let mining_effect = match &lib_card.kind {
            CardKind::Mining { mining_effect } => mining_effect.clone(),
            _ => return Err("Cannot play a non-mining card in mining encounter".to_string()),
        };

        // Check and deduct pre-play costs (stamina, lumber etc.)
        Self::check_and_deduct_gathering_costs(&mining_effect.costs, &mut self.token_balances)?;

        // Process gains by token type
        for gain in &mining_effect.gains {
            match gain.token_type {
                types::TokenType::MiningPower => {
                    // yield += mining_power × light_level / 100
                    let light_key = types::Token::persistent(types::TokenType::MiningLightLevel);
                    let light_level = self.token_balances.get(&light_key).copied().unwrap_or(0);
                    let yield_increase = gain.amount * light_level / 100;
                    let yield_key = types::Token::persistent(types::TokenType::MiningYield);
                    let yield_val = self.token_balances.entry(yield_key).or_insert(0);
                    *yield_val += yield_increase;
                }
                types::TokenType::MiningLightLevel => {
                    let light_key = types::Token::persistent(types::TokenType::MiningLightLevel);
                    let light_val = self.token_balances.entry(light_key).or_insert(0);
                    if let Some(cap) = gain.cap {
                        let capped_gain = (cap - *light_val).max(0).min(gain.amount);
                        *light_val += capped_gain;
                    } else {
                        *light_val += gain.amount;
                    }
                }
                _ => {
                    // Direct token addition (e.g., Stamina)
                    let entry =
                        types::token_entry_by_type(&mut self.token_balances, &gain.token_type);
                    *entry += gain.amount;
                }
            }
        }

        // Auto-resolve ore play
        self.resolve_ore_play(rng);

        // Player draws a mining card
        self.draw_player_mining_card(rng);

        // Check autoloss: if all mining hand cards are unpayable, player loses
        if self.current_encounter.is_some() && self.all_mining_hand_cards_unpayable() {
            self.finish_mining_encounter(false);
        }

        Ok(())
    }

    /// Check if all mining hand cards are unpayable (pre-play costs unaffordable).
    fn all_mining_hand_cards_unpayable(&self) -> bool {
        self.all_gathering_hand_cards_unpayable(|k| match k {
            CardKind::Mining { mining_effect } => Some(&mining_effect.costs),
            _ => None,
        })
    }

    /// Ore plays a random card from hand, applying token-based damages.
    /// Then draws a card from deck to hand.
    fn resolve_ore_play(&mut self, rng: &mut rand_pcg::Lcg64Xsh32) {
        let damages = {
            let mining = match &mut self.current_encounter {
                Some(EncounterState::Mining(m)) => m,
                _ => return,
            };
            let played_idx =
                match crate::library::game_state::deck_play_random(rng, &mut mining.ore_deck) {
                    Some(idx) => idx,
                    None => return,
                };
            mining.round += 1;
            mining.ore_deck[played_idx].damages.clone()
        };

        // Apply each damage to player tokens
        for damage in &damages {
            let key = types::Token::persistent(damage.token_type.clone());
            let val = self.token_balances.entry(key).or_insert(0);
            *val = (*val - damage.amount).max(0);
        }

        // Ore draws a card
        if let Some(EncounterState::Mining(mining)) = &mut self.current_encounter {
            crate::library::game_state::deck_draw_random(rng, &mut mining.ore_deck);
        }

        // Check if player durability is depleted
        let durability_key = types::Token::persistent(types::TokenType::MiningDurability);
        let durability = self
            .token_balances
            .get(&durability_key)
            .copied()
            .unwrap_or(0);
        if durability <= 0 {
            self.finish_mining_encounter(false);
        }
    }

    /// Conclude a mining encounter voluntarily: reward = min(stamina, yield) ore tokens.
    pub fn conclude_mining_encounter(&mut self) -> Result<(), String> {
        match &self.current_encounter {
            Some(EncounterState::Mining(m)) if m.outcome == EncounterOutcome::Undecided => {}
            _ => return Err("No active mining encounter to conclude".to_string()),
        }

        let stamina_key = types::Token::persistent(types::TokenType::Stamina);
        let yield_key = types::Token::persistent(types::TokenType::MiningYield);
        let stamina = self.token_balances.get(&stamina_key).copied().unwrap_or(0);
        let mining_yield = self.token_balances.get(&yield_key).copied().unwrap_or(0);
        let reward = stamina.min(mining_yield);

        // Deduct stamina cost
        if let Some(s) = self.token_balances.get_mut(&stamina_key) {
            *s -= reward;
        }

        // Grant ore reward
        let ore_key = types::Token::persistent(types::TokenType::Ore);
        let ore = self.token_balances.entry(ore_key).or_insert(0);
        *ore += reward;

        self.finish_mining_encounter(true);
        Ok(())
    }

    /// Finalize a mining encounter: clean up encounter-scoped tokens, record outcome.
    pub(crate) fn finish_mining_encounter(&mut self, is_win: bool) {
        let outcome = if is_win {
            EncounterOutcome::PlayerWon
        } else {
            EncounterOutcome::PlayerLost
        };
        self.last_encounter_result = Some(outcome.clone());
        self.encounter_results.push(outcome);
        self.current_encounter = None;
        self.encounter_phase = types::EncounterPhase::Scouting;

        // Clean up encounter-scoped tokens
        self.token_balances.insert(
            types::Token::persistent(types::TokenType::MiningLightLevel),
            0,
        );
        self.token_balances
            .insert(types::Token::persistent(types::TokenType::MiningYield), 0);
        self.token_balances
            .insert(types::Token::persistent(types::TokenType::MiningPower), 0);
    }

    /// Draw one player mining card from deck to hand, recycling discard if needed.
    fn draw_player_mining_card(&mut self, rng: &mut rand_pcg::Lcg64Xsh32) {
        self.draw_player_cards_of_kind(
            1,
            |k| matches!(k, CardKind::Mining { .. }),
            rng,
            Some(types::TokenType::MiningMaxHand),
        );
    }
}
