use crate::library::types::{
    self, CardCounts, CardEffectKind, CardKind, EncounterKind, EncounterOutcome, EncounterState,
    MiningEncounterState,
};
use crate::library::{GameState, Library};

use crate::library::game_state::roll_concrete_effect;

pub(crate) fn register_mining_cards(lib: &mut Library, rng: &mut rand_pcg::Lcg64Xsh32) {
    // ---- Mining EnemyCardEffect templates (ore hazards) ----

    let ore_light_small_id = lib.cards.len();
    lib.add_card(
        CardKind::EnemyCardEffect {
            kind: types::CardEffectKind::LoseTokens {
                target: types::EffectTarget::OnOpponent,
                token_type: types::TokenType::MiningLightLevel,
                min: 20,
                max: 40,
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
        rng,
        vec![types::Discipline::Mining],
    );

    let ore_light_medium_id = lib.cards.len();
    lib.add_card(
        CardKind::EnemyCardEffect {
            kind: types::CardEffectKind::LoseTokens {
                target: types::EffectTarget::OnOpponent,
                token_type: types::TokenType::MiningLightLevel,
                min: 40,
                max: 60,
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
        rng,
        vec![types::Discipline::Mining],
    );

    let ore_durability_medium_id = lib.cards.len();
    lib.add_card(
        CardKind::EnemyCardEffect {
            kind: types::CardEffectKind::LoseTokens {
                target: types::EffectTarget::OnOpponent,
                token_type: types::TokenType::Durability,
                min: 80,
                max: 120,
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
        rng,
        vec![types::Discipline::Mining],
    );

    let ore_durability_heavy_id = lib.cards.len();
    lib.add_card(
        CardKind::EnemyCardEffect {
            kind: types::CardEffectKind::LoseTokens {
                target: types::EffectTarget::OnOpponent,
                token_type: types::TokenType::Durability,
                min: 150,
                max: 250,
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
        rng,
        vec![types::Discipline::Mining],
    );

    let ore_health_id = lib.cards.len();
    lib.add_card(
        CardKind::EnemyCardEffect {
            kind: types::CardEffectKind::LoseTokens {
                target: types::EffectTarget::OnOpponent,
                token_type: types::TokenType::Health,
                min: 50,
                max: 100,
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
        rng,
        vec![types::Discipline::Mining],
    );

    // ---- Mining PlayerCardEffect templates ----

    let mining_power_gain_id = lib.cards.len();
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::GainTokens {
                target: types::EffectTarget::OnSelf,
                token_type: types::TokenType::MiningPower,
                cap_min: 300,
                cap_max: 1200,
                gain_min_percent: 100,
                gain_max_percent: 100,
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
        rng,
        vec![types::Discipline::Mining],
    );

    let mining_light_gain_id = lib.cards.len();
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::GainTokens {
                target: types::EffectTarget::OnSelf,
                token_type: types::TokenType::MiningLightLevel,
                cap_min: 200,
                cap_max: 400,
                gain_min_percent: 100,
                gain_max_percent: 100,
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
        rng,
        vec![types::Discipline::Mining],
    );

    let mining_stamina_gain_id = lib.cards.len();
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::GainTokens {
                target: types::EffectTarget::OnSelf,
                token_type: types::TokenType::Stamina,
                cap_min: 150,
                cap_max: 250,
                gain_min_percent: 100,
                gain_max_percent: 100,
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
        rng,
        vec![types::Discipline::Mining],
    );

    let mining_lumber_cost_id = lib.cards.len();
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::LoseTokens {
                target: types::EffectTarget::OnSelf,
                token_type: types::TokenType::Lumber,
                min: 10,
                max: 30,
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
        rng,
        vec![types::Discipline::Mining],
    );

    let mining_stamina_cost_id = lib.cards.len();
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::LoseTokens {
                target: types::EffectTarget::OnSelf,
                token_type: types::TokenType::Stamina,
                min: 50,
                max: 350,
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
        rng,
        vec![types::Discipline::Mining],
    );

    let mining_health_cost_id = lib.cards.len();
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::LoseTokens {
                target: types::EffectTarget::OnSelf,
                token_type: types::TokenType::Health,
                min: 100,
                max: 200,
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
        rng,
        vec![types::Discipline::Mining],
    );

    // ---- Player mining cards (rolled from templates) ----

    // Mining power card: high power, no cost
    lib.add_card(
        CardKind::Mining {
            effects: vec![roll_concrete_effect(rng, mining_power_gain_id, lib)],
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Mining],
    );

    // Balanced mining power card: moderate power
    lib.add_card(
        CardKind::Mining {
            effects: vec![roll_concrete_effect(rng, mining_power_gain_id, lib)],
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Mining],
    );

    // Light level card: restores light, costs lumber
    lib.add_card(
        CardKind::Mining {
            effects: vec![
                roll_concrete_effect(rng, mining_lumber_cost_id, lib),
                roll_concrete_effect(rng, mining_light_gain_id, lib),
            ],
        },
        CardCounts {
            library: 0,
            deck: 10,
            hand: 3,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Mining],
    );

    // Mining encounter: Iron Ore
    lib.add_card(
        CardKind::Encounter {
            encounter_kind: types::EncounterKind::Mining {
                mining_def: types::MiningDef {
                    initial_light_level: 300,
                    ore_deck: vec![
                        types::OreCard {
                            effects: vec![roll_concrete_effect(rng, ore_light_small_id, lib)],
                            counts: types::DeckCounts {
                                deck: 0,
                                hand: 6,
                                discard: 0,
                            },
                        },
                        types::OreCard {
                            effects: vec![
                                roll_concrete_effect(rng, ore_light_medium_id, lib),
                                roll_concrete_effect(rng, ore_durability_medium_id, lib),
                            ],
                            counts: types::DeckCounts {
                                deck: 0,
                                hand: 8,
                                discard: 0,
                            },
                        },
                        types::OreCard {
                            effects: vec![roll_concrete_effect(rng, ore_durability_heavy_id, lib)],
                            counts: types::DeckCounts {
                                deck: 0,
                                hand: 4,
                                discard: 0,
                            },
                        },
                        types::OreCard {
                            effects: vec![roll_concrete_effect(rng, ore_health_id, lib)],
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
        rng,
        vec![],
    );

    // High power mining card: costs stamina
    lib.add_card(
        CardKind::Mining {
            effects: vec![
                roll_concrete_effect(rng, mining_stamina_cost_id, lib),
                roll_concrete_effect(rng, mining_power_gain_id, lib),
            ],
        },
        CardCounts {
            library: 0,
            deck: 5,
            hand: 2,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Mining],
    );

    // High power + high cost
    lib.add_card(
        CardKind::Mining {
            effects: vec![
                roll_concrete_effect(rng, mining_stamina_cost_id, lib),
                roll_concrete_effect(rng, mining_power_gain_id, lib),
            ],
        },
        CardCounts {
            library: 0,
            deck: 3,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Mining],
    );

    // Very high power, highest cost
    lib.add_card(
        CardKind::Mining {
            effects: vec![
                roll_concrete_effect(rng, mining_stamina_cost_id, lib),
                roll_concrete_effect(rng, mining_power_gain_id, lib),
            ],
        },
        CardCounts {
            library: 0,
            deck: 2,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Mining],
    );

    // Large light level card: higher gain, higher lumber cost
    lib.add_card(
        CardKind::Mining {
            effects: vec![
                roll_concrete_effect(rng, mining_lumber_cost_id, lib),
                roll_concrete_effect(rng, mining_light_gain_id, lib),
            ],
        },
        CardCounts {
            library: 0,
            deck: 5,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Mining],
    );

    // Mining rest card: grants stamina, no power or light
    lib.add_card(
        CardKind::Mining {
            effects: vec![roll_concrete_effect(rng, mining_stamina_gain_id, lib)],
        },
        CardCounts {
            library: 0,
            deck: 3,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Mining],
    );

    // Stamina-cost starting mining card: above-average power
    lib.add_card(
        CardKind::Mining {
            effects: vec![
                roll_concrete_effect(rng, mining_stamina_cost_id, lib),
                roll_concrete_effect(rng, mining_power_gain_id, lib),
            ],
        },
        CardCounts {
            library: 1,
            deck: 1,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Mining],
    );

    // Health-cost starting mining card: highest power
    lib.add_card(
        CardKind::Mining {
            effects: vec![
                roll_concrete_effect(rng, mining_health_cost_id, lib),
                roll_concrete_effect(rng, mining_power_gain_id, lib),
            ],
        },
        CardCounts {
            library: 1,
            deck: 1,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Mining],
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
        let mut encounter_tokens = std::collections::HashMap::new();
        encounter_tokens.insert(
            types::Token::persistent(types::TokenType::MiningLightLevel),
            mining_def.initial_light_level,
        );
        encounter_tokens.insert(types::Token::persistent(types::TokenType::MiningYield), 0);

        let state = MiningEncounterState {
            round: 1,
            encounter_card_id,
            outcome: EncounterOutcome::Undecided,
            ore_deck,
            encounter_tokens,
        };
        self.current_encounter = Some(EncounterState::Mining(state));
        self.encounter_phase = types::EncounterPhase::InEncounter;
        Ok(())
    }

    /// Resolve a player mining card play against the current mining encounter.
    /// Reads effects from library: LoseTokens/OnSelf → costs, GainTokens/OnSelf → gains,
    /// Insight → insight tokens. Auto-resolves ore play, draws cards, checks encounter end.
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
        let effects = match &lib_card.kind {
            CardKind::Mining { effects } => effects.clone(),
            _ => return Err("Cannot play a non-mining card in mining encounter".to_string()),
        };

        // Extract and deduct pre-play costs from LoseTokens/OnSelf effects
        let costs = Self::extract_gathering_costs_from_effects(&effects, &self.library);
        Self::check_and_deduct_gathering_costs(&costs, &mut self.token_balances)?;

        // Process all effects via library templates
        for effect in &effects {
            let kind = match self.library.resolve_effect(effect.effect_id) {
                Some(resolved) => resolved,
                None => continue,
            };
            match &kind {
                CardEffectKind::GainTokens { token_type, .. } => {
                    match token_type {
                        types::TokenType::MiningPower => {
                            // yield += mining_power × light_level / 100
                            let encounter_tokens = match &mut self.current_encounter {
                                Some(EncounterState::Mining(m)) => &mut m.encounter_tokens,
                                _ => return Err("No active mining encounter".to_string()),
                            };
                            let light_key =
                                types::Token::persistent(types::TokenType::MiningLightLevel);
                            let light_level =
                                encounter_tokens.get(&light_key).copied().unwrap_or(0);
                            let yield_increase = effect.rolled_value * light_level / 100;
                            let yield_key = types::Token::persistent(types::TokenType::MiningYield);
                            let yield_val = encounter_tokens.entry(yield_key).or_insert(0);
                            *yield_val += yield_increase;
                        }
                        types::TokenType::MiningLightLevel => {
                            let encounter_tokens = match &mut self.current_encounter {
                                Some(EncounterState::Mining(m)) => &mut m.encounter_tokens,
                                _ => return Err("No active mining encounter".to_string()),
                            };
                            let light_key =
                                types::Token::persistent(types::TokenType::MiningLightLevel);
                            let light_val = encounter_tokens.entry(light_key).or_insert(0);
                            *light_val += effect.rolled_value;
                        }
                        _ => {
                            // Direct token addition to player balances (e.g., Stamina)
                            let entry =
                                types::token_entry_by_type(&mut self.token_balances, token_type);
                            *entry += effect.rolled_value;
                        }
                    }
                }
                CardEffectKind::Insight { .. } => {
                    let insight_type =
                        types::TokenType::insight_for_discipline(&types::Discipline::Mining);
                    let entry = types::token_entry_by_type(&mut self.token_balances, &insight_type);
                    *entry += effect.rolled_value;
                }
                // LoseTokens/OnSelf already handled above as costs
                _ => {}
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
        self.all_effects_hand_cards_unpayable(|k| match k {
            CardKind::Mining { effects } => Some(effects),
            _ => None,
        })
    }

    /// Ore plays a random card from hand, applying ConcreteEffect-based damages.
    /// Then draws a card from deck to hand.
    fn resolve_ore_play(&mut self, rng: &mut rand_pcg::Lcg64Xsh32) {
        let effects = {
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
            mining.ore_deck[played_idx].effects.clone()
        };

        for effect in &effects {
            let kind = match self.library.resolve_effect(effect.effect_id) {
                Some(resolved) => resolved,
                None => continue,
            };

            if let types::CardEffectKind::LoseTokens { token_type, .. } = &kind {
                let resolved_type = token_type.resolve_durability(&types::Discipline::Mining);
                let key = types::Token::persistent(resolved_type.clone());
                let damage = effect.rolled_value;
                match resolved_type {
                    types::TokenType::MiningLightLevel | types::TokenType::MiningYield => {
                        if let Some(EncounterState::Mining(m)) = &mut self.current_encounter {
                            let val = m.encounter_tokens.entry(key).or_insert(0);
                            *val = (*val - damage).max(0);
                        }
                    }
                    _ => {
                        let val = self.token_balances.entry(key).or_insert(0);
                        *val = (*val - damage).max(0);
                    }
                }
            }
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
        let mining_yield = match &self.current_encounter {
            Some(EncounterState::Mining(m)) if m.outcome == EncounterOutcome::Undecided => {
                let yield_key = types::Token::persistent(types::TokenType::MiningYield);
                m.encounter_tokens.get(&yield_key).copied().unwrap_or(0)
            }
            _ => return Err("No active mining encounter to conclude".to_string()),
        };

        let stamina_key = types::Token::persistent(types::TokenType::Stamina);
        let stamina = self.token_balances.get(&stamina_key).copied().unwrap_or(0);
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

    /// Finalize a mining encounter: record outcome. Encounter-scoped tokens are
    /// automatically dropped with the encounter state.
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
        self.check_player_death();
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
