use crate::library::game_state::{roll_concrete_effect, roll_range};
use crate::library::types::{
    self, CardCounts, CardEffectKind, CardKind, ConcreteEffectCost, EncounterKind,
    EncounterOutcome, EncounterState,
};
use crate::library::{GameState, Library};
use std::collections::HashMap;
use std::collections::HashSet;

/// Compute the card_value for a woodcutting card based on its chop effects.
/// Formula: sum(chop_values) * unique_chop_types * 100
fn compute_woodcutting_card_value(effects: &[types::ConcreteEffect], lib: &Library) -> i64 {
    let mut sum_values = 0i64;
    let mut unique_types = HashSet::new();
    for e in effects {
        if let Some(CardEffectKind::WoodcuttingChop { chop_type, .. }) =
            lib.resolve_effect(e.effect_id)
        {
            sum_values += e.rolled_value;
            unique_types.insert(chop_type.clone());
        }
    }
    if unique_types.is_empty() {
        return 200; // Default for rest cards (no chops)
    }
    sum_values * unique_types.len() as i64 * 100
}

/// Set card_value and costs on a woodcutting card's effects.
/// Costs are given as pre-rolled absolute amounts, converted to percentages of card_value.
fn apply_woodcutting_costs(
    effects: &mut [types::ConcreteEffect],
    lib: &Library,
    absolute_costs: &[(types::TokenType, i64)],
) {
    let card_value = compute_woodcutting_card_value(effects, lib);
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
                is_absolute: false,
            })
            .collect();
    }
}

pub(crate) fn register_woodcutting_cards(lib: &mut Library, rng: &mut rand_pcg::Lcg64Xsh32) {
    // ---- Woodcutting PlayerCardEffect templates ----

    // Dead entries: former LoseTokens/OnSelf cost templates. Kept to preserve card indices.
    let _wc_durability_cost_id = lib.add_card(
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
        vec![types::Discipline::Woodcutting],
    );

    let _wc_stamina_cost_id = lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::LoseTokens {
                token_type: types::TokenType::Stamina,
                min: 100,
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
        vec![types::Discipline::Woodcutting],
    );

    let _wc_health_cost_id = lib.add_card(
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
        vec![types::Discipline::Woodcutting],
    );

    // Gain: Stamina
    let wc_stamina_gain_id = lib.add_card(
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
        vec![types::Discipline::Woodcutting],
    );

    // Chop: LightChop (values 1-3)
    let wc_light_chop_id = lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::WoodcuttingChop {
                chop_type: types::ChopType::LightChop,
                min_value: 1,
                max_value: 3,
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
        vec![types::Discipline::Woodcutting],
    );

    // Chop: HeavyChop (values 3-7)
    let wc_heavy_chop_id = lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::WoodcuttingChop {
                chop_type: types::ChopType::HeavyChop,
                min_value: 3,
                max_value: 7,
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
        vec![types::Discipline::Woodcutting],
    );

    // Chop: MediumChop (values 3-6)
    let wc_medium_chop_id = lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::WoodcuttingChop {
                chop_type: types::ChopType::MediumChop,
                min_value: 3,
                max_value: 6,
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
        vec![types::Discipline::Woodcutting],
    );

    // Chop: PrecisionChop (values 7-9)
    let wc_precision_chop_id = lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::WoodcuttingChop {
                chop_type: types::ChopType::PrecisionChop,
                min_value: 7,
                max_value: 9,
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
        vec![types::Discipline::Woodcutting],
    );

    // Chop: SplitChop (values 4-8)
    let wc_split_chop_id = lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::WoodcuttingChop {
                chop_type: types::ChopType::SplitChop,
                min_value: 4,
                max_value: 8,
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
        vec![types::Discipline::Woodcutting],
    );

    // ---- Woodcutting player cards ----

    // Card 1: LightChop, cost=Durability
    let mut effects = vec![roll_concrete_effect(rng, wc_light_chop_id, lib)];
    let dur_cost = roll_range(rng, 50, 100);
    apply_woodcutting_costs(
        &mut effects,
        lib,
        &[(types::TokenType::Durability, dur_cost)],
    );
    lib.add_card(
        CardKind::Woodcutting { effects },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 2,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Woodcutting],
    );

    // Card 2: HeavyChop, cost=Durability
    let mut effects = vec![roll_concrete_effect(rng, wc_heavy_chop_id, lib)];
    let dur_cost = roll_range(rng, 50, 100);
    apply_woodcutting_costs(
        &mut effects,
        lib,
        &[(types::TokenType::Durability, dur_cost)],
    );
    lib.add_card(
        CardKind::Woodcutting { effects },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 1,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Woodcutting],
    );

    // Card 3: MediumChop, cost=Durability
    let mut effects = vec![roll_concrete_effect(rng, wc_medium_chop_id, lib)];
    let dur_cost = roll_range(rng, 50, 100);
    apply_woodcutting_costs(
        &mut effects,
        lib,
        &[(types::TokenType::Durability, dur_cost)],
    );
    lib.add_card(
        CardKind::Woodcutting { effects },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 1,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Woodcutting],
    );

    // Card 4: PrecisionChop, cost=Durability
    let mut effects = vec![roll_concrete_effect(rng, wc_precision_chop_id, lib)];
    let dur_cost = roll_range(rng, 50, 100);
    apply_woodcutting_costs(
        &mut effects,
        lib,
        &[(types::TokenType::Durability, dur_cost)],
    );
    lib.add_card(
        CardKind::Woodcutting { effects },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 1,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Woodcutting],
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
        rng,
        vec![],
    );

    // Card 5: HeavyChop+LightChop, cost=Stamina + Durability
    let mut effects = vec![
        roll_concrete_effect(rng, wc_heavy_chop_id, lib),
        roll_concrete_effect(rng, wc_light_chop_id, lib),
    ];
    let stam_cost = roll_range(rng, 100, 250);
    let dur_cost = roll_range(rng, 50, 100);
    apply_woodcutting_costs(
        &mut effects,
        lib,
        &[
            (types::TokenType::Stamina, stam_cost),
            (types::TokenType::Durability, dur_cost),
        ],
    );
    lib.add_card(
        CardKind::Woodcutting { effects },
        CardCounts {
            library: 0,
            deck: 5,
            hand: 1,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Woodcutting],
    );

    // Card 6: SplitChop, cost=Durability
    let mut effects = vec![roll_concrete_effect(rng, wc_split_chop_id, lib)];
    let dur_cost = roll_range(rng, 50, 100);
    apply_woodcutting_costs(
        &mut effects,
        lib,
        &[(types::TokenType::Durability, dur_cost)],
    );
    lib.add_card(
        CardKind::Woodcutting { effects },
        CardCounts {
            library: 0,
            deck: 10,
            hand: 1,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Woodcutting],
    );

    // Card 7: LightChop+MediumChop, cost=Durability
    let mut effects = vec![
        roll_concrete_effect(rng, wc_light_chop_id, lib),
        roll_concrete_effect(rng, wc_medium_chop_id, lib),
    ];
    let dur_cost = roll_range(rng, 50, 100);
    apply_woodcutting_costs(
        &mut effects,
        lib,
        &[(types::TokenType::Durability, dur_cost)],
    );
    lib.add_card(
        CardKind::Woodcutting { effects },
        CardCounts {
            library: 0,
            deck: 5,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Woodcutting],
    );

    // Card 8: Heavy+Medium+Precision, cost=Stamina + Durability
    let mut effects = vec![
        roll_concrete_effect(rng, wc_heavy_chop_id, lib),
        roll_concrete_effect(rng, wc_medium_chop_id, lib),
        roll_concrete_effect(rng, wc_precision_chop_id, lib),
    ];
    let stam_cost = roll_range(rng, 100, 250);
    let dur_cost = roll_range(rng, 50, 100);
    apply_woodcutting_costs(
        &mut effects,
        lib,
        &[
            (types::TokenType::Stamina, stam_cost),
            (types::TokenType::Durability, dur_cost),
        ],
    );
    lib.add_card(
        CardKind::Woodcutting { effects },
        CardCounts {
            library: 0,
            deck: 3,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Woodcutting],
    );

    // Card 9: Light+Heavy+Medium+Split, cost=Stamina + Durability
    let mut effects = vec![
        roll_concrete_effect(rng, wc_light_chop_id, lib),
        roll_concrete_effect(rng, wc_heavy_chop_id, lib),
        roll_concrete_effect(rng, wc_medium_chop_id, lib),
        roll_concrete_effect(rng, wc_split_chop_id, lib),
    ];
    let stam_cost = roll_range(rng, 100, 250);
    let dur_cost = roll_range(rng, 50, 100);
    apply_woodcutting_costs(
        &mut effects,
        lib,
        &[
            (types::TokenType::Stamina, stam_cost),
            (types::TokenType::Durability, dur_cost),
        ],
    );
    lib.add_card(
        CardKind::Woodcutting { effects },
        CardCounts {
            library: 0,
            deck: 2,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Woodcutting],
    );

    // Card 10: Rest card (no chops), cost=Durability, gains=Stamina
    let mut rest_effects = vec![roll_concrete_effect(rng, wc_stamina_gain_id, lib)];
    {
        let card_value = 200i64; // Fixed value for rest cards
        for effect in &mut rest_effects {
            effect.card_value = Some(card_value);
        }
        let dur_cost = roll_range(rng, 50, 100);
        let percent = if card_value > 0 {
            (dur_cost * 100 / card_value).max(1) as u32
        } else {
            100
        };
        rest_effects[0].rolled_costs = vec![ConcreteEffectCost {
            token_type: types::TokenType::Durability,
            rolled_percent: percent,
            is_absolute: false,
        }];
    }
    lib.add_card(
        CardKind::Woodcutting {
            effects: rest_effects,
        },
        CardCounts {
            library: 0,
            deck: 3,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Woodcutting],
    );

    // Card 11: Precision+Medium, cost=Stamina + Durability
    let mut effects = vec![
        roll_concrete_effect(rng, wc_precision_chop_id, lib),
        roll_concrete_effect(rng, wc_medium_chop_id, lib),
    ];
    let stam_cost = roll_range(rng, 100, 250);
    let dur_cost = roll_range(rng, 50, 100);
    apply_woodcutting_costs(
        &mut effects,
        lib,
        &[
            (types::TokenType::Stamina, stam_cost),
            (types::TokenType::Durability, dur_cost),
        ],
    );
    lib.add_card(
        CardKind::Woodcutting { effects },
        CardCounts {
            library: 1,
            deck: 1,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Woodcutting],
    );

    // Card 12: Heavy+Precision+Medium, cost=Health + Durability
    let mut effects = vec![
        roll_concrete_effect(rng, wc_heavy_chop_id, lib),
        roll_concrete_effect(rng, wc_precision_chop_id, lib),
        roll_concrete_effect(rng, wc_medium_chop_id, lib),
    ];
    let health_cost = roll_range(rng, 150, 200);
    let dur_cost = roll_range(rng, 50, 100);
    apply_woodcutting_costs(
        &mut effects,
        lib,
        &[
            (types::TokenType::Health, health_cost),
            (types::TokenType::Durability, dur_cost),
        ],
    );
    lib.add_card(
        CardKind::Woodcutting { effects },
        CardCounts {
            library: 1,
            deck: 1,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![types::Discipline::Woodcutting],
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
        self.all_effects_hand_cards_unpayable(|k| match k {
            CardKind::Woodcutting { effects, .. } => Some(effects),
            _ => None,
        })
    }

    /// Finalize a woodcutting encounter: grant pattern-scaled rewards on win.
    /// Conclude a woodcutting encounter voluntarily: grant rewards if any accumulated.
    pub fn conclude_woodcutting_encounter(&mut self) -> Result<(), String> {
        match &self.current_encounter {
            Some(EncounterState::Woodcutting(w)) if w.outcome == EncounterOutcome::Undecided => {
                if w.base_rewards.values().all(|&v| v <= 0) {
                    return Err("No rewards accumulated; abort the encounter instead".to_string());
                }
            }
            _ => return Err("No active woodcutting encounter to conclude".to_string()),
        }
        self.finish_woodcutting_encounter(true);
        Ok(())
    }

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
        let rounds = match &self.current_encounter {
            Some(EncounterState::Woodcutting(w)) => w.round,
            _ => 0,
        };
        self.record_encounter_finish(types::Discipline::Woodcutting, outcome, rounds);
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
