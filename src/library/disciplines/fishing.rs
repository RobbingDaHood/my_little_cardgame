use crate::library::game_state::{roll_concrete_effect, roll_range};
use crate::library::types::{
    self, CardCounts, CardEffectKind, CardKind, ConcreteEffectCost, EncounterKind,
    EncounterOutcome, EncounterState,
};
use crate::library::{GameState, Library};
use std::collections::HashMap;

/// Compute the card_value for a fishing card based on its benefit effects.
/// FishingValue effects contribute their rolled_value; modifiers contribute a flat 200.
fn compute_fishing_card_value(effects: &[types::ConcreteEffect], lib: &Library) -> i64 {
    let mut total = 0i64;
    for e in effects {
        if let Some(kind) = lib.resolve_effect(e.effect_id) {
            match kind {
                CardEffectKind::FishingValue { .. } => total += e.rolled_value,
                CardEffectKind::GainTokens {
                    token_type:
                        types::TokenType::FishingRangeMin
                        | types::TokenType::FishingRangeMax
                        | types::TokenType::FishAmount,
                    ..
                } => {
                    total += 200;
                }
                CardEffectKind::GainTokens { .. } => total += e.rolled_value,
                _ => {}
            }
        }
    }
    total.max(200)
}

/// Set card_value and costs on a fishing card's effects.
/// Costs are given as pre-rolled absolute amounts, converted to percentages of card_value.
fn apply_fishing_costs(
    effects: &mut [types::ConcreteEffect],
    lib: &Library,
    absolute_costs: &[(types::TokenType, i64)],
) {
    let card_value = compute_fishing_card_value(effects, lib);
    for effect in effects.iter_mut() {
        effect.card_value = Some(card_value);
    }
    if !effects.is_empty() && !absolute_costs.is_empty() {
        effects[0].rolled_costs = absolute_costs
            .iter()
            .map(|(token_type, absolute)| ConcreteEffectCost {
                token_type: token_type.clone(),
                rolled_percent: if card_value > 0 {
                    (*absolute * 100 / card_value).max(1) as u32
                } else {
                    100
                },
            })
            .collect();
    }
}

pub(crate) fn register_fishing_cards(lib: &mut Library, rng: &mut rand_pcg::Lcg64Xsh32) {
    // ---- Fishing EnemyCardEffect templates ----

    // Fish value effect (low): 50-150
    let fish_low_id = lib.add_card(
        CardKind::EnemyCardEffect {
            kind: types::CardEffectKind::GainTokens {
                target: types::EffectTarget::OnSelf,
                token_type: types::TokenType::Fish,
                cap_min: 50,
                cap_max: 150,
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
        vec![types::Discipline::Fishing],
    );

    // Fish value effect (medium): 200-400
    let fish_medium_id = lib.add_card(
        CardKind::EnemyCardEffect {
            kind: types::CardEffectKind::GainTokens {
                target: types::EffectTarget::OnSelf,
                token_type: types::TokenType::Fish,
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
        vec![types::Discipline::Fishing],
    );

    // Fish value effect (high): 400-600
    let fish_high_id = lib.add_card(
        CardKind::EnemyCardEffect {
            kind: types::CardEffectKind::GainTokens {
                target: types::EffectTarget::OnSelf,
                token_type: types::TokenType::Fish,
                cap_min: 400,
                cap_max: 600,
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
        vec![types::Discipline::Fishing],
    );

    // Fish value effect (very high): 600-800
    let fish_very_high_id = lib.add_card(
        CardKind::EnemyCardEffect {
            kind: types::CardEffectKind::GainTokens {
                target: types::EffectTarget::OnSelf,
                token_type: types::TokenType::Fish,
                cap_min: 600,
                cap_max: 800,
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
        vec![types::Discipline::Fishing],
    );

    // ---- Fishing PlayerCardEffect templates ----

    // Cost: Durability (dead entry — costs now stored as rolled_costs on benefit effects)
    let _fish_durability_cost_id = lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::LoseTokens {
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
        vec![types::Discipline::Fishing],
    );

    // Cost: Stamina (dead entry — costs now stored as rolled_costs on benefit effects)
    let _fish_stamina_cost_id = lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::LoseTokens {
                token_type: types::TokenType::Stamina,
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
        vec![types::Discipline::Fishing],
    );

    // Cost: Health (dead entry — costs now stored as rolled_costs on benefit effects)
    let _fish_health_cost_id = lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::LoseTokens {
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
        vec![types::Discipline::Fishing],
    );

    // Gain: FishingRangeMin (covers -150, 50)
    let fish_range_min_gain_id = lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::GainTokens {
                target: types::EffectTarget::OnSelf,
                token_type: types::TokenType::FishingRangeMin,
                cap_min: -150,
                cap_max: 50,
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
        vec![types::Discipline::Fishing],
    );

    // Gain: FishingRangeMax (covers -50, 150)
    let fish_range_max_gain_id = lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::GainTokens {
                target: types::EffectTarget::OnSelf,
                token_type: types::TokenType::FishingRangeMax,
                cap_min: -50,
                cap_max: 150,
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
        vec![types::Discipline::Fishing],
    );

    // Gain: FishAmount (covers -1, 1)
    let fish_amount_gain_id = lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::GainTokens {
                target: types::EffectTarget::OnSelf,
                token_type: types::TokenType::FishAmount,
                cap_min: -1,
                cap_max: 1,
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
        vec![types::Discipline::Fishing],
    );

    // Gain: Stamina (covers 200)
    let fish_stamina_gain_id = lib.add_card(
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
        vec![types::Discipline::Fishing],
    );

    // FishingValue: Low (50-200)
    let fish_value_low_id = lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::FishingValue {
                min: 50,
                max: 200,
                costs: vec![],
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Fishing],
    );

    // FishingValue: Medium (250-450)
    let fish_value_medium_id = lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::FishingValue {
                min: 250,
                max: 450,
                costs: vec![],
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Fishing],
    );

    // FishingValue: High (500-750)
    let fish_value_high_id = lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::FishingValue {
                min: 500,
                max: 750,
                costs: vec![],
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Fishing],
    );

    // ---- Player fishing cards (rolled from templates) ----

    // Low value fishing card: durability cost + 1 low value
    let mut effects = vec![roll_concrete_effect(rng, fish_value_low_id, lib)];
    let dur_cost = roll_range(rng, 50, 100);
    apply_fishing_costs(
        &mut effects,
        lib,
        &[(types::TokenType::Durability, dur_cost)],
    );
    lib.add_card(
        CardKind::Fishing { effects },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Fishing],
    );

    // Medium value fishing card: durability cost + 1 medium value
    let mut effects = vec![roll_concrete_effect(rng, fish_value_medium_id, lib)];
    let dur_cost = roll_range(rng, 50, 100);
    apply_fishing_costs(
        &mut effects,
        lib,
        &[(types::TokenType::Durability, dur_cost)],
    );
    lib.add_card(
        CardKind::Fishing { effects },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Fishing],
    );

    // High value fishing card: durability cost + 1 high value
    let mut effects = vec![roll_concrete_effect(rng, fish_value_high_id, lib)];
    let dur_cost = roll_range(rng, 50, 100);
    apply_fishing_costs(
        &mut effects,
        lib,
        &[(types::TokenType::Durability, dur_cost)],
    );
    lib.add_card(
        CardKind::Fishing { effects },
        CardCounts {
            library: 0,
            deck: 10,
            hand: 5,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Fishing],
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
                            effects: vec![roll_concrete_effect(rng, fish_low_id, lib)],
                            counts: types::DeckCounts {
                                deck: 0,
                                hand: 6,
                                discard: 0,
                            },
                        },
                        types::FishCard {
                            value: 300,
                            effects: vec![roll_concrete_effect(rng, fish_medium_id, lib)],
                            counts: types::DeckCounts {
                                deck: 0,
                                hand: 6,
                                discard: 0,
                            },
                        },
                        types::FishCard {
                            value: 500,
                            effects: vec![roll_concrete_effect(rng, fish_high_id, lib)],
                            counts: types::DeckCounts {
                                deck: 0,
                                hand: 4,
                                discard: 0,
                            },
                        },
                        types::FishCard {
                            value: 700,
                            effects: vec![roll_concrete_effect(rng, fish_very_high_id, lib)],
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
        rng,
        vec![],
    );

    // Widen range — reduces min value token
    let mut effects = vec![roll_concrete_effect(rng, fish_range_min_gain_id, lib)];
    let dur_cost = roll_range(rng, 50, 100);
    apply_fishing_costs(
        &mut effects,
        lib,
        &[(types::TokenType::Durability, dur_cost)],
    );
    lib.add_card(
        CardKind::Fishing { effects },
        CardCounts {
            library: 0,
            deck: 5,
            hand: 1,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Fishing],
    );

    // Widen range — increases max value token
    let mut effects = vec![roll_concrete_effect(rng, fish_range_max_gain_id, lib)];
    let dur_cost = roll_range(rng, 50, 100);
    apply_fishing_costs(
        &mut effects,
        lib,
        &[(types::TokenType::Durability, dur_cost)],
    );
    lib.add_card(
        CardKind::Fishing { effects },
        CardCounts {
            library: 0,
            deck: 5,
            hand: 1,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Fishing],
    );

    // Multi-value + narrows range (3 values)
    let mut effects = vec![
        roll_concrete_effect(rng, fish_range_min_gain_id, lib),
        roll_concrete_effect(rng, fish_range_max_gain_id, lib),
        roll_concrete_effect(rng, fish_value_low_id, lib),
        roll_concrete_effect(rng, fish_value_medium_id, lib),
        roll_concrete_effect(rng, fish_value_high_id, lib),
    ];
    let stam_cost = roll_range(rng, 100, 200);
    let dur_cost = roll_range(rng, 50, 100);
    apply_fishing_costs(
        &mut effects,
        lib,
        &[
            (types::TokenType::Stamina, stam_cost),
            (types::TokenType::Durability, dur_cost),
        ],
    );
    lib.add_card(
        CardKind::Fishing { effects },
        CardCounts {
            library: 0,
            deck: 3,
            hand: 1,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Fishing],
    );

    // Increase fish amount
    let mut effects = vec![roll_concrete_effect(rng, fish_amount_gain_id, lib)];
    let dur_cost = roll_range(rng, 50, 100);
    apply_fishing_costs(
        &mut effects,
        lib,
        &[(types::TokenType::Durability, dur_cost)],
    );
    lib.add_card(
        CardKind::Fishing { effects },
        CardCounts {
            library: 0,
            deck: 3,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Fishing],
    );

    // Multi-value but decreases fish amount
    let mut effects = vec![
        roll_concrete_effect(rng, fish_amount_gain_id, lib),
        roll_concrete_effect(rng, fish_value_low_id, lib),
        roll_concrete_effect(rng, fish_value_medium_id, lib),
        roll_concrete_effect(rng, fish_value_high_id, lib),
    ];
    let stam_cost = roll_range(rng, 100, 200);
    let dur_cost = roll_range(rng, 50, 100);
    apply_fishing_costs(
        &mut effects,
        lib,
        &[
            (types::TokenType::Stamina, stam_cost),
            (types::TokenType::Durability, dur_cost),
        ],
    );
    lib.add_card(
        CardKind::Fishing { effects },
        CardCounts {
            library: 0,
            deck: 3,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Fishing],
    );

    // Rest card — grants stamina, no values
    let mut effects = vec![roll_concrete_effect(rng, fish_stamina_gain_id, lib)];
    let dur_cost = roll_range(rng, 50, 100);
    apply_fishing_costs(
        &mut effects,
        lib,
        &[(types::TokenType::Durability, dur_cost)],
    );
    lib.add_card(
        CardKind::Fishing { effects },
        CardCounts {
            library: 0,
            deck: 3,
            hand: 1,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Fishing],
    );

    // Stamina cost card with multiple values (4 values: low, medium, high, high)
    let mut effects = vec![
        roll_concrete_effect(rng, fish_value_low_id, lib),
        roll_concrete_effect(rng, fish_value_medium_id, lib),
        roll_concrete_effect(rng, fish_value_high_id, lib),
        roll_concrete_effect(rng, fish_value_high_id, lib),
    ];
    let stam_cost = roll_range(rng, 100, 200);
    let dur_cost = roll_range(rng, 50, 100);
    apply_fishing_costs(
        &mut effects,
        lib,
        &[
            (types::TokenType::Stamina, stam_cost),
            (types::TokenType::Durability, dur_cost),
        ],
    );
    lib.add_card(
        CardKind::Fishing { effects },
        CardCounts {
            library: 0,
            deck: 3,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Fishing],
    );

    // Stamina-cost starting fishing card: 3 values (low, medium, high)
    let mut effects = vec![
        roll_concrete_effect(rng, fish_value_low_id, lib),
        roll_concrete_effect(rng, fish_value_medium_id, lib),
        roll_concrete_effect(rng, fish_value_high_id, lib),
    ];
    let stam_cost = roll_range(rng, 100, 200);
    let dur_cost = roll_range(rng, 50, 100);
    apply_fishing_costs(
        &mut effects,
        lib,
        &[
            (types::TokenType::Stamina, stam_cost),
            (types::TokenType::Durability, dur_cost),
        ],
    );
    lib.add_card(
        CardKind::Fishing { effects },
        CardCounts {
            library: 1,
            deck: 1,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Fishing],
    );

    // Health-cost starting fishing card: 5 values (low, medium, medium, high, high)
    let mut effects = vec![
        roll_concrete_effect(rng, fish_value_low_id, lib),
        roll_concrete_effect(rng, fish_value_medium_id, lib),
        roll_concrete_effect(rng, fish_value_medium_id, lib),
        roll_concrete_effect(rng, fish_value_high_id, lib),
        roll_concrete_effect(rng, fish_value_high_id, lib),
    ];
    let health_cost = roll_range(rng, 150, 200);
    let dur_cost = roll_range(rng, 50, 100);
    apply_fishing_costs(
        &mut effects,
        lib,
        &[
            (types::TokenType::Health, health_cost),
            (types::TokenType::Durability, dur_cost),
        ],
    );
    lib.add_card(
        CardKind::Fishing { effects },
        CardCounts {
            library: 1,
            deck: 1,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Fishing],
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
            encounter_tokens,
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

        // If card has no FishingValue effects, skip the fishing duel (utility-only card)
        if values.is_empty() {
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

        // Read current range and fish amount from encounter tokens
        let (valid_min, valid_max, fish_amount) = match &self.current_encounter {
            Some(EncounterState::Fishing(f)) => {
                let min_key = types::Token::persistent(types::TokenType::FishingRangeMin);
                let max_key = types::Token::persistent(types::TokenType::FishingRangeMax);
                let amount_key = types::Token::persistent(types::TokenType::FishAmount);
                (
                    f.encounter_tokens.get(&min_key).copied().unwrap_or(0),
                    f.encounter_tokens.get(&max_key).copied().unwrap_or(0),
                    f.encounter_tokens
                        .get(&amount_key)
                        .copied()
                        .unwrap_or(1)
                        .max(1),
                )
            }
            _ => return Err("No active fishing encounter".to_string()),
        };

        // Auto-resolve fish play: pick random fish card from hand
        let fish_value = Self::fish_play_random(rng, &mut self.current_encounter);

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
        self.all_effects_hand_cards_unpayable(|k| match k {
            CardKind::Fishing { effects } => Some(effects),
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

    /// Conclude a fishing encounter voluntarily: grant rewards if any accumulated.
    pub fn conclude_fishing_encounter(&mut self) -> Result<(), String> {
        match &self.current_encounter {
            Some(EncounterState::Fishing(f)) if f.outcome == EncounterOutcome::Undecided => {
                if f.rewards.values().all(|&v| v <= 0) {
                    return Err("No rewards accumulated; abort the encounter instead".to_string());
                }
            }
            _ => return Err("No active fishing encounter to conclude".to_string()),
        }
        self.finish_fishing_encounter(true);
        Ok(())
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
        let rounds = match &self.current_encounter {
            Some(EncounterState::Fishing(f)) => f.round,
            _ => 0,
        };
        self.record_encounter_finish(types::Discipline::Fishing, outcome, rounds);
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
