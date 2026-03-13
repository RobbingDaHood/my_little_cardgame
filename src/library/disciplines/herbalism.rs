use crate::library::types::{
    self, CardCounts, CardEffectKind, CardKind, EncounterKind, EncounterOutcome, EncounterState,
    HerbalismEncounterState,
};
use crate::library::{GameState, Library};
use std::collections::HashMap;

use crate::library::game_state::roll_concrete_effect;

pub(crate) fn register_herbalism_cards(lib: &mut Library, rng: &mut rand_pcg::Lcg64Xsh32) {
    // ---- Herbalism EnemyCardEffect templates ----

    // Plant passive effect: small gain (single-characteristic plants)
    let plant_small_id = lib.cards.len();
    lib.add_card(
        CardKind::EnemyCardEffect {
            kind: types::CardEffectKind::GainTokens {
                target: types::EffectTarget::OnSelf,
                token_type: types::TokenType::Plant,
                cap_min: 50,
                cap_max: 100,
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
        vec![types::Discipline::Herbalism],
    );

    // Plant passive effect: medium gain (dual-characteristic plants)
    let plant_medium_id = lib.cards.len();
    lib.add_card(
        CardKind::EnemyCardEffect {
            kind: types::CardEffectKind::GainTokens {
                target: types::EffectTarget::OnSelf,
                token_type: types::TokenType::Plant,
                cap_min: 100,
                cap_max: 200,
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
        vec![types::Discipline::Herbalism],
    );

    // ---- Herbalism PlayerCardEffect templates ----

    let durability_cost_id = lib.cards.len();
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::LoseTokens {
                target: types::EffectTarget::OnSelf,
                token_type: types::TokenType::Durability,
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
        vec![types::Discipline::Herbalism],
    );

    let stamina_cost_id = lib.cards.len();
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::LoseTokens {
                target: types::EffectTarget::OnSelf,
                token_type: types::TokenType::Stamina,
                min: 100,
                max: 150,
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
        vec![types::Discipline::Herbalism],
    );

    let health_cost_id = lib.cards.len();
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::LoseTokens {
                target: types::EffectTarget::OnSelf,
                token_type: types::TokenType::Health,
                min: 150,
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
        vec![types::Discipline::Herbalism],
    );

    let stamina_gain_id = lib.cards.len();
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::GainTokens {
                target: types::EffectTarget::OnSelf,
                token_type: types::TokenType::Stamina,
                cap_min: 200,
                cap_max: 200,
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
        vec![types::Discipline::Herbalism],
    );

    // ---- HerbalismMatch templates (one per unique match_mode) ----

    let match_or_fragile_id = lib.cards.len();
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::HerbalismMatch {
                match_mode: types::HerbalismMatchMode::Or {
                    types: vec![types::PlantCharacteristic::Fragile],
                },
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Herbalism],
    );

    let match_or_thorny_aromatic_id = lib.cards.len();
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::HerbalismMatch {
                match_mode: types::HerbalismMatchMode::Or {
                    types: vec![
                        types::PlantCharacteristic::Thorny,
                        types::PlantCharacteristic::Aromatic,
                    ],
                },
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Herbalism],
    );

    let match_or_bitter_luminous_fragile_id = lib.cards.len();
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::HerbalismMatch {
                match_mode: types::HerbalismMatchMode::Or {
                    types: vec![
                        types::PlantCharacteristic::Bitter,
                        types::PlantCharacteristic::Luminous,
                        types::PlantCharacteristic::Fragile,
                    ],
                },
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Herbalism],
    );

    let match_most_common_id = lib.cards.len();
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::HerbalismMatch {
                match_mode: types::HerbalismMatchMode::MostCommon {
                    limit: 1,
                    types: vec![],
                },
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Herbalism],
    );

    let match_least_common_id = lib.cards.len();
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::HerbalismMatch {
                match_mode: types::HerbalismMatchMode::LeastCommon {
                    limit: 1,
                    types: vec![],
                },
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Herbalism],
    );

    let match_and_fragile_thorny_id = lib.cards.len();
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::HerbalismMatch {
                match_mode: types::HerbalismMatchMode::And {
                    types: vec![
                        types::PlantCharacteristic::Fragile,
                        types::PlantCharacteristic::Thorny,
                    ],
                },
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Herbalism],
    );

    let match_or_empty_id = lib.cards.len();
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::HerbalismMatch {
                match_mode: types::HerbalismMatchMode::Or { types: vec![] },
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Herbalism],
    );

    let match_or_4chars_id = lib.cards.len();
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::HerbalismMatch {
                match_mode: types::HerbalismMatchMode::Or {
                    types: vec![
                        types::PlantCharacteristic::Fragile,
                        types::PlantCharacteristic::Thorny,
                        types::PlantCharacteristic::Aromatic,
                        types::PlantCharacteristic::Bitter,
                    ],
                },
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Herbalism],
    );

    let match_or_all5_id = lib.cards.len();
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::HerbalismMatch {
                match_mode: types::HerbalismMatchMode::Or {
                    types: vec![
                        types::PlantCharacteristic::Fragile,
                        types::PlantCharacteristic::Thorny,
                        types::PlantCharacteristic::Aromatic,
                        types::PlantCharacteristic::Bitter,
                        types::PlantCharacteristic::Luminous,
                    ],
                },
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Herbalism],
    );

    // ---- Player herbalism cards (rolled from templates) ----

    // Narrow herbalism card: targets 1 characteristic, durability cost
    lib.add_card(
        CardKind::Herbalism {
            effects: vec![
                roll_concrete_effect(rng, durability_cost_id, lib),
                roll_concrete_effect(rng, match_or_fragile_id, lib),
            ],
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Herbalism],
    );

    // Medium herbalism card: targets 2 characteristics
    lib.add_card(
        CardKind::Herbalism {
            effects: vec![
                roll_concrete_effect(rng, durability_cost_id, lib),
                roll_concrete_effect(rng, match_or_thorny_aromatic_id, lib),
            ],
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Herbalism],
    );

    // Broad herbalism card: targets 3 characteristics
    lib.add_card(
        CardKind::Herbalism {
            effects: vec![
                roll_concrete_effect(rng, durability_cost_id, lib),
                roll_concrete_effect(rng, match_or_bitter_luminous_fragile_id, lib),
            ],
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Herbalism],
    );

    // Herbalism encounter: Meadow Herb
    lib.add_card(
        CardKind::Encounter {
            encounter_kind: types::EncounterKind::Herbalism {
                herbalism_def: types::HerbalismDef {
                    plant_hand: vec![
                        types::PlantCard {
                            characteristics: vec![types::PlantCharacteristic::Fragile],
                            effects: vec![roll_concrete_effect(rng, plant_small_id, lib)],
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
                            effects: vec![roll_concrete_effect(rng, plant_medium_id, lib)],
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
                            effects: vec![roll_concrete_effect(rng, plant_medium_id, lib)],
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
                            effects: vec![roll_concrete_effect(rng, plant_medium_id, lib)],
                            counts: types::DeckCounts {
                                deck: 0,
                                hand: 1,
                                discard: 0,
                            },
                        },
                        types::PlantCard {
                            characteristics: vec![types::PlantCharacteristic::Luminous],
                            effects: vec![roll_concrete_effect(rng, plant_small_id, lib)],
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
        rng,
        vec![],
    );

    // MostCommon card — removes the most common characteristic (limit 1)
    lib.add_card(
        CardKind::Herbalism {
            effects: vec![
                roll_concrete_effect(rng, stamina_cost_id, lib),
                roll_concrete_effect(rng, durability_cost_id, lib),
                roll_concrete_effect(rng, match_most_common_id, lib),
            ],
        },
        CardCounts {
            library: 0,
            deck: 3,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Herbalism],
    );

    // LeastCommon card — removes the least common characteristic (limit 1)
    lib.add_card(
        CardKind::Herbalism {
            effects: vec![
                roll_concrete_effect(rng, stamina_cost_id, lib),
                roll_concrete_effect(rng, durability_cost_id, lib),
                roll_concrete_effect(rng, match_least_common_id, lib),
            ],
        },
        CardCounts {
            library: 0,
            deck: 3,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Herbalism],
    );

    // AND-based multi-type card — removes only plants matching ALL listed types
    lib.add_card(
        CardKind::Herbalism {
            effects: vec![
                roll_concrete_effect(rng, stamina_cost_id, lib),
                roll_concrete_effect(rng, durability_cost_id, lib),
                roll_concrete_effect(rng, match_and_fragile_thorny_id, lib),
            ],
        },
        CardCounts {
            library: 0,
            deck: 3,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Herbalism],
    );

    // Stamina rest card for herbalism
    lib.add_card(
        CardKind::Herbalism {
            effects: vec![
                roll_concrete_effect(rng, durability_cost_id, lib),
                roll_concrete_effect(rng, match_or_empty_id, lib),
                roll_concrete_effect(rng, stamina_gain_id, lib),
            ],
        },
        CardCounts {
            library: 0,
            deck: 3,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Herbalism],
    );

    // Stamina-cost starting herbalism card: matches 4 characteristics
    lib.add_card(
        CardKind::Herbalism {
            effects: vec![
                roll_concrete_effect(rng, stamina_cost_id, lib),
                roll_concrete_effect(rng, durability_cost_id, lib),
                roll_concrete_effect(rng, match_or_4chars_id, lib),
            ],
        },
        CardCounts {
            library: 1,
            deck: 1,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Herbalism],
    );

    // Health-cost starting herbalism card: matches ALL 5 characteristics
    lib.add_card(
        CardKind::Herbalism {
            effects: vec![
                roll_concrete_effect(rng, health_cost_id, lib),
                roll_concrete_effect(rng, durability_cost_id, lib),
                roll_concrete_effect(rng, match_or_all5_id, lib),
            ],
        },
        CardCounts {
            library: 1,
            deck: 1,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Herbalism],
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

        // Extract costs from LoseTokens/OnSelf effects and split into pre/post-play
        let costs = Self::extract_gathering_costs_from_effects(&effects, &self.library);
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
                CardEffectKind::HerbalismMatch { match_mode } => {
                    match_mode_opt = Some(match_mode.clone());
                }
                // LoseTokens/OnSelf already handled above as costs
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
}
