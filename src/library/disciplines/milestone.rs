use super::super::types::{self, *};
use super::super::Library;
use crate::library::game_state::{roll_best_concrete_effect, GameState};
use std::collections::HashMap;

fn milestone_insight_cost(tier: u32) -> i64 {
    100 * (1i64 << (tier - 1))
}

fn scale_factor(tier: u32) -> f64 {
    1.5f64.powi(tier as i32 - 1)
}

/// Find the base (non-milestone) encounter definition for a discipline.
fn find_base_encounter(lib: &Library, discipline: &Discipline) -> Option<EncounterKind> {
    lib.cards.iter().find_map(|card| {
        if let CardKind::Encounter { encounter_kind } = &card.kind {
            if matches!(encounter_kind, EncounterKind::Milestone { .. }) {
                return None;
            }
            if matches!(
                (discipline, encounter_kind),
                (Discipline::Combat, EncounterKind::Combat { .. })
                    | (Discipline::Mining, EncounterKind::Mining { .. })
                    | (Discipline::Herbalism, EncounterKind::Herbalism { .. })
                    | (Discipline::Woodcutting, EncounterKind::Woodcutting { .. })
                    | (Discipline::Fishing, EncounterKind::Fishing { .. })
            ) {
                return Some(encounter_kind.clone());
            }
        }
        None
    })
}

/// Build mapping from source-tier enemy effect IDs to target-tier effect IDs.
/// Effects are matched positionally: the Nth source-tier effect maps to the Nth target-tier effect.
fn tier_effect_mapping(
    lib: &Library,
    discipline: &Discipline,
    from_tier: u32,
    to_tier: u32,
) -> HashMap<usize, usize> {
    let from: Vec<usize> = lib
        .enemy_effects_for_discipline_and_tier(discipline, from_tier)
        .iter()
        .map(|(id, _)| *id)
        .collect();

    if from_tier == to_tier {
        return from.iter().map(|id| (*id, *id)).collect();
    }

    let to: Vec<usize> = lib
        .enemy_effects_for_discipline_and_tier(discipline, to_tier)
        .iter()
        .map(|(id, _)| *id)
        .collect();

    from.iter().zip(to.iter()).map(|(f, t)| (*f, *t)).collect()
}

/// Rebuild concrete effects using roll_best on mapped (tier-appropriate) effect IDs.
fn rebuild_effects_best(
    effects: &[ConcreteEffect],
    effect_map: &HashMap<usize, usize>,
    lib: &Library,
) -> Vec<ConcreteEffect> {
    effects
        .iter()
        .map(|e| {
            let mapped_id = effect_map.get(&e.effect_id).copied().unwrap_or(e.effect_id);
            roll_best_concrete_effect(mapped_id, lib)
        })
        .collect()
}

fn register_combat_milestone(
    lib: &mut Library,
    rng: &mut rand_pcg::Lcg64Xsh32,
    tier: u32,
    card_id: usize,
    previous_combatant: Option<&CombatantDef>,
) {
    let combatant_def = if let Some(prev) = previous_combatant {
        let effect_map = tier_effect_mapping(lib, &Discipline::Combat, tier - 1, tier);
        let tier_scale = scale_factor(tier) / scale_factor(tier - 1);

        let rebuild_deck = |deck: &[EnemyCardDef]| -> Vec<EnemyCardDef> {
            deck.iter()
                .map(|card_def| EnemyCardDef {
                    effects: rebuild_effects_best(&card_def.effects, &effect_map, lib),
                    counts: card_def.counts.clone(),
                })
                .collect()
        };

        let initial_tokens = prev
            .initial_tokens
            .iter()
            .map(|(k, v)| (k.clone(), (*v as f64 * tier_scale).round() as u64))
            .collect();

        CombatantDef {
            initial_tokens,
            attack_deck: rebuild_deck(&prev.attack_deck),
            defence_deck: rebuild_deck(&prev.defence_deck),
            resource_deck: rebuild_deck(&prev.resource_deck),
        }
    } else if let Some(EncounterKind::Combat {
        combatant_def: base,
    }) = find_base_encounter(lib, &Discipline::Combat)
    {
        let effect_map = tier_effect_mapping(lib, &Discipline::Combat, 1, tier);

        let rebuild_deck = |deck: &[EnemyCardDef]| -> Vec<EnemyCardDef> {
            deck.iter()
                .map(|card_def| EnemyCardDef {
                    effects: rebuild_effects_best(&card_def.effects, &effect_map, lib),
                    counts: card_def.counts.clone(),
                })
                .collect()
        };

        let initial_tokens = base
            .initial_tokens
            .iter()
            .map(|(k, v)| (k.clone(), (*v as f64 * scale_factor(tier)).round() as u64))
            .collect();

        CombatantDef {
            initial_tokens,
            attack_deck: rebuild_deck(&base.attack_deck),
            defence_deck: rebuild_deck(&base.defence_deck),
            resource_deck: rebuild_deck(&base.resource_deck),
        }
    } else {
        let enemy_hp = (3000.0 * scale_factor(tier)).round() as u64;
        CombatantDef {
            initial_tokens: HashMap::from([
                (Token::persistent(TokenType::Health), enemy_hp),
                (Token::persistent(TokenType::MaxHealth), enemy_hp),
            ]),
            attack_deck: vec![],
            defence_deck: vec![],
            resource_deck: vec![],
        }
    };

    let milestone_def = MilestoneDef {
        inner_encounter_kind: Box::new(EncounterKind::Combat { combatant_def }),
        discipline: Discipline::Combat,
        tier,
        insight_cost: milestone_insight_cost(tier),
    };

    lib.replace_card(
        card_id,
        CardKind::Encounter {
            encounter_kind: EncounterKind::Milestone { milestone_def },
        },
        CardCounts {
            library: 0,
            deck: 0,
            hand: 1,
            discard: 0,
        },
        rng,
        vec![Discipline::Combat],
        tier,
    );
}

fn register_mining_milestone(
    lib: &mut Library,
    rng: &mut rand_pcg::Lcg64Xsh32,
    tier: u32,
    card_id: usize,
    previous_mining: Option<&MiningDef>,
) {
    let (source, from_tier) = if let Some(prev) = previous_mining {
        (prev.clone(), tier - 1)
    } else if let Some(EncounterKind::Mining { mining_def }) =
        find_base_encounter(lib, &Discipline::Mining)
    {
        (mining_def, 1)
    } else {
        let empty = MiningDef {
            initial_light_level: (200.0 * scale_factor(tier)).round() as i64,
            ore_deck: vec![],
        };
        let milestone_def = MilestoneDef {
            inner_encounter_kind: Box::new(EncounterKind::Mining { mining_def: empty }),
            discipline: Discipline::Mining,
            tier,
            insight_cost: milestone_insight_cost(tier),
        };
        lib.replace_card(
            card_id,
            CardKind::Encounter {
                encounter_kind: EncounterKind::Milestone { milestone_def },
            },
            CardCounts {
                library: 0,
                deck: 0,
                hand: 1,
                discard: 0,
            },
            rng,
            vec![Discipline::Mining],
            tier,
        );
        return;
    };

    let effect_map = tier_effect_mapping(lib, &Discipline::Mining, from_tier, tier);
    let scale = if from_tier > 0 {
        scale_factor(tier) / scale_factor(from_tier)
    } else {
        scale_factor(tier)
    };

    let mining_def = MiningDef {
        initial_light_level: (source.initial_light_level as f64 * scale).round() as i64,
        ore_deck: source
            .ore_deck
            .iter()
            .map(|ore| OreCard {
                effects: rebuild_effects_best(&ore.effects, &effect_map, lib),
                counts: ore.counts.clone(),
            })
            .collect(),
    };

    let milestone_def = MilestoneDef {
        inner_encounter_kind: Box::new(EncounterKind::Mining { mining_def }),
        discipline: Discipline::Mining,
        tier,
        insight_cost: milestone_insight_cost(tier),
    };

    lib.replace_card(
        card_id,
        CardKind::Encounter {
            encounter_kind: EncounterKind::Milestone { milestone_def },
        },
        CardCounts {
            library: 0,
            deck: 0,
            hand: 1,
            discard: 0,
        },
        rng,
        vec![Discipline::Mining],
        tier,
    );
}

fn register_herbalism_milestone(
    lib: &mut Library,
    rng: &mut rand_pcg::Lcg64Xsh32,
    tier: u32,
    card_id: usize,
    previous_herbalism: Option<&HerbalismDef>,
) {
    let (source, from_tier) = if let Some(prev) = previous_herbalism {
        (prev.clone(), tier - 1)
    } else if let Some(EncounterKind::Herbalism { herbalism_def }) =
        find_base_encounter(lib, &Discipline::Herbalism)
    {
        (herbalism_def, 1)
    } else {
        let empty = HerbalismDef {
            plant_hand: vec![],
            rewards: HashMap::new(),
        };
        let milestone_def = MilestoneDef {
            inner_encounter_kind: Box::new(EncounterKind::Herbalism {
                herbalism_def: empty,
            }),
            discipline: Discipline::Herbalism,
            tier,
            insight_cost: milestone_insight_cost(tier),
        };
        lib.replace_card(
            card_id,
            CardKind::Encounter {
                encounter_kind: EncounterKind::Milestone { milestone_def },
            },
            CardCounts {
                library: 0,
                deck: 0,
                hand: 1,
                discard: 0,
            },
            rng,
            vec![Discipline::Herbalism],
            tier,
        );
        return;
    };

    let effect_map = tier_effect_mapping(lib, &Discipline::Herbalism, from_tier, tier);
    let scale = if from_tier > 0 {
        scale_factor(tier) / scale_factor(from_tier)
    } else {
        scale_factor(tier)
    };

    let plant_hand = source
        .plant_hand
        .iter()
        .map(|plant| PlantCard {
            characteristics: plant.characteristics.clone(),
            effects: rebuild_effects_best(&plant.effects, &effect_map, lib),
            counts: DeckCounts {
                deck: plant.counts.deck,
                hand: (plant.counts.hand as f64 * scale).round().max(1.0) as u32,
                discard: plant.counts.discard,
            },
        })
        .collect();

    let rewards = source
        .rewards
        .iter()
        .map(|(k, v)| (k.clone(), (*v as f64 * scale).round() as i64))
        .collect();

    let herbalism_def = HerbalismDef {
        plant_hand,
        rewards,
    };

    let milestone_def = MilestoneDef {
        inner_encounter_kind: Box::new(EncounterKind::Herbalism { herbalism_def }),
        discipline: Discipline::Herbalism,
        tier,
        insight_cost: milestone_insight_cost(tier),
    };

    lib.replace_card(
        card_id,
        CardKind::Encounter {
            encounter_kind: EncounterKind::Milestone { milestone_def },
        },
        CardCounts {
            library: 0,
            deck: 0,
            hand: 1,
            discard: 0,
        },
        rng,
        vec![Discipline::Herbalism],
        tier,
    );
}

fn register_woodcutting_milestone(
    lib: &mut Library,
    rng: &mut rand_pcg::Lcg64Xsh32,
    tier: u32,
    card_id: usize,
    previous_woodcutting: Option<&WoodcuttingDef>,
) {
    let woodcutting_def = if let Some(prev) = previous_woodcutting {
        let tier_scale = scale_factor(tier) / scale_factor(tier - 1);
        let max_plays = std::cmp::max(3, prev.max_plays.saturating_sub(1));
        let base_rewards = prev
            .base_rewards
            .iter()
            .map(|(k, v)| (k.clone(), (*v as f64 * tier_scale).round() as i64))
            .collect();
        WoodcuttingDef {
            max_plays,
            base_rewards,
        }
    } else if let Some(EncounterKind::Woodcutting { woodcutting_def }) =
        find_base_encounter(lib, &Discipline::Woodcutting)
    {
        let max_plays = std::cmp::max(
            3,
            woodcutting_def
                .max_plays
                .saturating_sub(tier.saturating_sub(1)),
        );
        let base_rewards = woodcutting_def
            .base_rewards
            .iter()
            .map(|(k, v)| (k.clone(), (*v as f64 * scale_factor(tier)).round() as i64))
            .collect();
        WoodcuttingDef {
            max_plays,
            base_rewards,
        }
    } else {
        let max_plays = std::cmp::max(3, 8u32.saturating_sub(tier));
        let mut base_rewards = HashMap::new();
        base_rewards.insert(
            Token::persistent(TokenType::Lumber),
            (50.0 * scale_factor(tier)).round() as i64,
        );
        base_rewards.insert(
            Token::persistent(TokenType::WoodcuttingInsight),
            (10.0 * scale_factor(tier)).round() as i64,
        );
        WoodcuttingDef {
            max_plays,
            base_rewards,
        }
    };

    let milestone_def = MilestoneDef {
        inner_encounter_kind: Box::new(EncounterKind::Woodcutting { woodcutting_def }),
        discipline: Discipline::Woodcutting,
        tier,
        insight_cost: milestone_insight_cost(tier),
    };

    lib.replace_card(
        card_id,
        CardKind::Encounter {
            encounter_kind: EncounterKind::Milestone { milestone_def },
        },
        CardCounts {
            library: 0,
            deck: 0,
            hand: 1,
            discard: 0,
        },
        rng,
        vec![Discipline::Woodcutting],
        tier,
    );
}

fn register_fishing_milestone(
    lib: &mut Library,
    rng: &mut rand_pcg::Lcg64Xsh32,
    tier: u32,
    card_id: usize,
    previous_fishing: Option<&FishingDef>,
) {
    let (source, from_tier) = if let Some(prev) = previous_fishing {
        (prev.clone(), tier - 1)
    } else if let Some(EncounterKind::Fishing { fishing_def }) =
        find_base_encounter(lib, &Discipline::Fishing)
    {
        (fishing_def, 1)
    } else {
        let empty = FishingDef {
            valid_range_min: -60,
            valid_range_max: 60,
            max_turns: 10,
            win_turns_needed: 6,
            fish_deck: vec![],
            rewards: HashMap::new(),
        };
        let milestone_def = MilestoneDef {
            inner_encounter_kind: Box::new(EncounterKind::Fishing { fishing_def: empty }),
            discipline: Discipline::Fishing,
            tier,
            insight_cost: milestone_insight_cost(tier),
        };
        lib.replace_card(
            card_id,
            CardKind::Encounter {
                encounter_kind: EncounterKind::Milestone { milestone_def },
            },
            CardCounts {
                library: 0,
                deck: 0,
                hand: 1,
                discard: 0,
            },
            rng,
            vec![Discipline::Fishing],
            tier,
        );
        return;
    };

    let effect_map = tier_effect_mapping(lib, &Discipline::Fishing, from_tier, tier);
    let scale = if from_tier > 0 {
        scale_factor(tier) / scale_factor(from_tier)
    } else {
        scale_factor(tier)
    };

    let source_span = (source.valid_range_max - source.valid_range_min) / 2;
    let range_span = std::cmp::max(20, source_span - 10);

    let fish_deck = source
        .fish_deck
        .iter()
        .map(|fish| FishCard {
            value: (fish.value as f64 * scale).round() as i64,
            effects: rebuild_effects_best(&fish.effects, &effect_map, lib),
            counts: fish.counts.clone(),
        })
        .collect();

    let rewards = source
        .rewards
        .iter()
        .map(|(k, v)| (k.clone(), (*v as f64 * scale).round() as i64))
        .collect();

    let fishing_def = FishingDef {
        valid_range_min: -range_span,
        valid_range_max: range_span,
        max_turns: (source.max_turns as f64 * scale).round() as u32,
        win_turns_needed: (source.win_turns_needed as f64 * scale).round() as u32,
        fish_deck,
        rewards,
    };

    let milestone_def = MilestoneDef {
        inner_encounter_kind: Box::new(EncounterKind::Fishing { fishing_def }),
        discipline: Discipline::Fishing,
        tier,
        insight_cost: milestone_insight_cost(tier),
    };

    lib.replace_card(
        card_id,
        CardKind::Encounter {
            encounter_kind: EncounterKind::Milestone { milestone_def },
        },
        CardCounts {
            library: 0,
            deck: 0,
            hand: 1,
            discard: 0,
        },
        rng,
        vec![Discipline::Fishing],
        tier,
    );
}

/// Generate 50%-improved versions of every existing PlayerCardEffect and EnemyCardEffect
/// for the given discipline at the given tier. Tags new effects with tier+1.
/// Returns the IDs of newly created effect cards.
pub(crate) fn generate_milestone_reward_effects(
    lib: &mut Library,
    rng: &mut rand_pcg::Lcg64Xsh32,
    discipline: &Discipline,
    tier: u32,
) -> Vec<usize> {
    let next_tier = tier + 1;

    // Scale player effects at this tier
    let player_effects: Vec<(usize, CardEffectKind)> = lib
        .card_effects_for_discipline_and_tier(discipline, tier)
        .iter()
        .filter_map(|(id, card)| match &card.kind {
            CardKind::PlayerCardEffect { kind } => Some((*id, kind.clone())),
            _ => None,
        })
        .collect();

    let mut new_ids = Vec::new();
    for (_old_id, kind) in &player_effects {
        let improved = scale_card_effect_kind(kind, 1.5);
        let new_id = lib.add_card_with_tier(
            CardKind::PlayerCardEffect { kind: improved },
            CardCounts {
                library: 1,
                deck: 0,
                hand: 0,
                discard: 0,
            },
            rng,
            vec![discipline.clone()],
            next_tier,
        );
        new_ids.push(new_id);
    }

    // Scale enemy effects at this tier
    let enemy_effects: Vec<(usize, CardEffectKind)> = lib
        .enemy_effects_for_discipline_and_tier(discipline, tier)
        .iter()
        .filter_map(|(id, card)| match &card.kind {
            CardKind::EnemyCardEffect { kind } => Some((*id, kind.clone())),
            _ => None,
        })
        .collect();

    for (_old_id, kind) in &enemy_effects {
        let improved = scale_card_effect_kind(kind, 1.5);
        let new_id = lib.add_card_with_tier(
            CardKind::EnemyCardEffect { kind: improved },
            CardCounts {
                library: 1,
                deck: 0,
                hand: 0,
                discard: 0,
            },
            rng,
            vec![discipline.clone()],
            next_tier,
        );
        new_ids.push(new_id);
    }

    new_ids
}

fn scale_card_effect_kind(kind: &CardEffectKind, factor: f64) -> CardEffectKind {
    match kind {
        CardEffectKind::GainTokens {
            target,
            token_type,
            cap_min,
            cap_max,
            gain_min_percent,
            gain_max_percent,
            costs,
            duration,
        } => CardEffectKind::GainTokens {
            target: target.clone(),
            token_type: token_type.clone(),
            cap_min: ((*cap_min as f64) * factor).round() as i64,
            cap_max: ((*cap_max as f64) * factor).round() as i64,
            gain_min_percent: *gain_min_percent,
            gain_max_percent: *gain_max_percent,
            costs: costs.clone(),
            duration: duration.clone(),
        },
        CardEffectKind::LoseTokens {
            token_type,
            min,
            max,
            costs,
            duration,
        } => CardEffectKind::LoseTokens {
            token_type: token_type.clone(),
            min: ((*min as f64) * factor).round() as i64,
            max: ((*max as f64) * factor).round() as i64,
            costs: costs.clone(),
            duration: duration.clone(),
        },
        CardEffectKind::DrawCards {
            attack,
            defence,
            resource,
        } => CardEffectKind::DrawCards {
            attack: attack + 1,
            defence: *defence,
            resource: *resource,
        },
        CardEffectKind::Insight { min, max } => CardEffectKind::Insight {
            min: ((*min as f64) * factor).round() as i64,
            max: ((*max as f64) * factor).round() as i64,
        },
        CardEffectKind::WoodcuttingChop {
            chop_type,
            min_value,
            max_value,
            costs,
        } => CardEffectKind::WoodcuttingChop {
            chop_type: chop_type.clone(),
            min_value: ((*min_value as f64) * factor).round() as u32,
            max_value: ((*max_value as f64) * factor).round() as u32,
            costs: costs.clone(),
        },
        CardEffectKind::HerbalismMatch { match_mode, costs } => CardEffectKind::HerbalismMatch {
            match_mode: match_mode.clone(),
            costs: costs.clone(),
        },
        CardEffectKind::FishingValue { min, max, costs } => CardEffectKind::FishingValue {
            min: ((*min as f64) * factor).round() as i64,
            max: ((*max as f64) * factor).round() as i64,
            costs: costs.clone(),
        },
        CardEffectKind::CraftingReduction {
            token_type,
            min,
            max,
            costs,
        } => CardEffectKind::CraftingReduction {
            token_type: token_type.clone(),
            min: ((*min as f64) * factor).round() as i64,
            max: ((*max as f64) * factor).round() as i64,
            costs: costs.clone(),
        },
        CardEffectKind::ResearchProbe { symbols, costs } => CardEffectKind::ResearchProbe {
            symbols: symbols.clone(),
            costs: costs.clone(),
        },
        CardEffectKind::ResearchInterference { kind } => {
            CardEffectKind::ResearchInterference { kind: kind.clone() }
        }
    }
}

/// Replace the defeated milestone card in-place with a next-tier milestone encounter.
pub(crate) fn generate_next_tier_milestone_encounter(
    lib: &mut Library,
    rng: &mut rand_pcg::Lcg64Xsh32,
    discipline: &Discipline,
    current_tier: u32,
    defeated_encounter: Option<&EncounterKind>,
    card_id: usize,
) {
    let next_tier = current_tier + 1;
    match (discipline, defeated_encounter) {
        (Discipline::Combat, Some(EncounterKind::Combat { combatant_def })) => {
            register_combat_milestone(lib, rng, next_tier, card_id, Some(combatant_def));
        }
        (Discipline::Combat, _) => {
            register_combat_milestone(lib, rng, next_tier, card_id, None);
        }
        (Discipline::Mining, Some(EncounterKind::Mining { mining_def })) => {
            register_mining_milestone(lib, rng, next_tier, card_id, Some(mining_def));
        }
        (Discipline::Mining, _) => {
            register_mining_milestone(lib, rng, next_tier, card_id, None);
        }
        (Discipline::Herbalism, Some(EncounterKind::Herbalism { herbalism_def })) => {
            register_herbalism_milestone(lib, rng, next_tier, card_id, Some(herbalism_def));
        }
        (Discipline::Herbalism, _) => {
            register_herbalism_milestone(lib, rng, next_tier, card_id, None);
        }
        (Discipline::Woodcutting, Some(EncounterKind::Woodcutting { woodcutting_def })) => {
            register_woodcutting_milestone(lib, rng, next_tier, card_id, Some(woodcutting_def));
        }
        (Discipline::Woodcutting, _) => {
            register_woodcutting_milestone(lib, rng, next_tier, card_id, None);
        }
        (Discipline::Fishing, Some(EncounterKind::Fishing { fishing_def })) => {
            register_fishing_milestone(lib, rng, next_tier, card_id, Some(fishing_def));
        }
        (Discipline::Fishing, _) => {
            register_fishing_milestone(lib, rng, next_tier, card_id, None);
        }
        _ => {}
    }
}

// ---- GameState methods for milestone encounters ----

impl GameState {
    /// Start a milestone encounter: deduct MilestoneInsight, delegate to inner discipline.
    pub fn start_milestone_encounter(
        &mut self,
        encounter_card_id: usize,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let lib_card = self
            .library
            .get(encounter_card_id)
            .ok_or_else(|| format!("Card {} not found in Library", encounter_card_id))?
            .clone();

        let milestone_def = match &lib_card.kind {
            CardKind::Encounter {
                encounter_kind: EncounterKind::Milestone { milestone_def },
            } => milestone_def.clone(),
            _ => {
                return Err(format!(
                    "Card {} is not a milestone encounter",
                    encounter_card_id
                ))
            }
        };

        // Deduct MilestoneInsight cost
        let insight_key = Token::persistent(TokenType::MilestoneInsight);
        let balance = self.token_balances.get(&insight_key).copied().unwrap_or(0);
        if balance < milestone_def.insight_cost {
            return Err(format!(
                "Insufficient MilestoneInsight: have {}, need {}",
                balance, milestone_def.insight_cost
            ));
        }
        *self.token_balances.entry(insight_key).or_insert(0) -= milestone_def.insight_cost;

        // Start the inner encounter
        let discipline = milestone_def.discipline.clone();
        let tier = milestone_def.tier;
        match milestone_def.inner_encounter_kind.as_ref() {
            EncounterKind::Combat { combatant_def } => {
                self.start_combat_inner(encounter_card_id, combatant_def.clone(), rng)?;
            }
            EncounterKind::Mining { mining_def } => {
                self.start_mining_inner(encounter_card_id, mining_def.clone(), rng)?;
            }
            EncounterKind::Herbalism { herbalism_def } => {
                self.start_herbalism_inner(encounter_card_id, herbalism_def.clone(), rng)?;
            }
            EncounterKind::Woodcutting { woodcutting_def } => {
                self.start_woodcutting_inner(encounter_card_id, woodcutting_def.clone(), rng)?;
            }
            EncounterKind::Fishing { fishing_def } => {
                self.start_fishing_inner(encounter_card_id, fishing_def.clone(), rng)?;
            }
            _ => return Err("Unsupported milestone inner encounter type".to_string()),
        }

        // Wrap the inner state in MilestoneEncounterState
        let inner_state = self
            .current_encounter
            .take()
            .ok_or("Inner encounter failed to initialize")?;

        self.current_encounter = Some(EncounterState::Milestone(MilestoneEncounterState {
            encounter_card_id,
            inner_state: Box::new(inner_state),
            discipline,
            tier,
            outcome: EncounterOutcome::Undecided,
        }));
        self.encounter_phase = types::EncounterPhase::InEncounter;
        Ok(())
    }

    /// Delegate card play to the inner encounter within a milestone.
    pub fn resolve_milestone_play_card(
        &mut self,
        card_id: usize,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        // Extract the inner state, resolve the play, then re-wrap
        let milestone = match self.current_encounter.take() {
            Some(EncounterState::Milestone(m)) => m,
            other => {
                self.current_encounter = other;
                return Err("Not in a milestone encounter".to_string());
            }
        };

        // Temporarily set the inner state as current
        self.current_encounter = Some(*milestone.inner_state);

        // Delegate to the appropriate discipline's play resolution
        let result = match &self.current_encounter {
            Some(EncounterState::Combat(_)) => {
                self.resolve_player_card(card_id, rng)?;
                if self.current_encounter.is_some() {
                    self.resolve_enemy_play(rng)?;
                    if self.current_encounter.is_some() {
                        self.advance_combat_phase()?;
                    }
                }
                Ok(())
            }
            Some(EncounterState::Mining(_)) => self.resolve_player_mining_card(card_id, rng),
            Some(EncounterState::Herbalism(_)) => self.resolve_player_herbalism_card(card_id, rng),
            Some(EncounterState::Woodcutting(_)) => {
                self.resolve_player_woodcutting_card(card_id, rng)
            }
            Some(EncounterState::Fishing(_)) => self.resolve_player_fishing_card(card_id, rng),
            _ => Err("Unsupported inner encounter type for milestone".to_string()),
        };

        // Check if inner encounter ended (combat sets current_encounter = None on finish)
        let inner_finished = self.current_encounter.is_none()
            || self
                .current_encounter
                .as_ref()
                .map(|e| e.is_finished())
                .unwrap_or(false);

        if inner_finished {
            // The inner discipline's finish logic already ran (including
            // setting encounter_phase to Scouting and recording metrics).
            // We need to override the behaviour for milestones.
            let inner_outcome = if self.current_encounter.is_none() {
                // Combat clears current_encounter on finish; check last_encounter_result
                self.encounter_results
                    .last()
                    .cloned()
                    .unwrap_or(EncounterOutcome::PlayerLost)
            } else {
                self.current_encounter
                    .as_ref()
                    .map(|e| e.outcome().clone())
                    .unwrap_or(EncounterOutcome::PlayerLost)
            };

            self.handle_milestone_finish(
                milestone.encounter_card_id,
                milestone.discipline,
                milestone.tier,
                inner_outcome,
                rng,
            );
        } else {
            // Re-wrap the inner state
            let inner_state = self.current_encounter.take().ok_or("Inner state lost")?;
            self.current_encounter = Some(EncounterState::Milestone(MilestoneEncounterState {
                encounter_card_id: milestone.encounter_card_id,
                inner_state: Box::new(inner_state),
                discipline: milestone.discipline,
                tier: milestone.tier,
                outcome: EncounterOutcome::Undecided,
            }));
            self.encounter_phase = types::EncounterPhase::InEncounter;
        }

        result
    }

    fn handle_milestone_finish(
        &mut self,
        encounter_card_id: usize,
        discipline: Discipline,
        tier: u32,
        outcome: EncounterOutcome,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) {
        self.current_encounter = None;

        if outcome == EncounterOutcome::PlayerWon {
            // Extract the defeated milestone's inner encounter before any mutation
            let defeated_inner =
                self.library
                    .get(encounter_card_id)
                    .and_then(|card| match &card.kind {
                        CardKind::Encounter {
                            encounter_kind: EncounterKind::Milestone { milestone_def },
                        } => Some(milestone_def.inner_encounter_kind.as_ref().clone()),
                        _ => None,
                    });

            // Generate reward effects (scaled to next tier)
            generate_milestone_reward_effects(&mut self.library, rng, &discipline, tier);

            // Replace the card in-place with the next-tier milestone
            generate_next_tier_milestone_encounter(
                &mut self.library,
                rng,
                &discipline,
                tier,
                defeated_inner.as_ref(),
                encounter_card_id,
            );

            self.encounter_phase = types::EncounterPhase::NoEncounter;
        } else {
            // Loss: return encounter card to hand, go to NoEncounter
            let _ = self.library.return_to_hand(encounter_card_id);
            self.encounter_phase = types::EncounterPhase::NoEncounter;
        }

        self.check_player_death();
    }

    /// Abort a milestone encounter (treated as loss).
    pub fn abort_milestone_encounter(&mut self) {
        if let Some(EncounterState::Milestone(m)) = &self.current_encounter {
            let enc_id = m.encounter_card_id;
            let discipline = m.discipline.clone();
            let rounds = m.inner_state.round();
            self.record_encounter_finish(discipline, EncounterOutcome::PlayerLost, rounds);
            self.current_encounter = None;
            let _ = self.library.return_to_hand(enc_id);
            self.encounter_phase = types::EncounterPhase::NoEncounter;
            self.check_player_death();
        }
    }

    // ---- Inner encounter start helpers (bypass card lookup, use def directly) ----

    fn start_combat_inner(
        &mut self,
        encounter_card_id: usize,
        combatant_def: CombatantDef,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        use crate::library::game_state::deck_shuffle_hand;

        let mut attack_deck = combatant_def.attack_deck;
        let mut defence_deck = combatant_def.defence_deck;
        let mut resource_deck = combatant_def.resource_deck;
        deck_shuffle_hand(rng, &mut attack_deck);
        deck_shuffle_hand(rng, &mut defence_deck);
        deck_shuffle_hand(rng, &mut resource_deck);

        let state = CombatEncounterState {
            round: 1,
            phase: CombatPhase::Defending,
            enemy_tokens: combatant_def
                .initial_tokens
                .iter()
                .map(|(k, v)| (k.clone(), *v as i64))
                .collect(),
            encounter_card_id,
            outcome: EncounterOutcome::Undecided,
            enemy_attack_deck: attack_deck,
            enemy_defence_deck: defence_deck,
            enemy_resource_deck: resource_deck,
        };
        self.current_encounter = Some(EncounterState::Combat(state));
        self.encounter_phase = types::EncounterPhase::InEncounter;
        Ok(())
    }

    fn start_mining_inner(
        &mut self,
        encounter_card_id: usize,
        mining_def: MiningDef,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        use crate::library::game_state::deck_shuffle_hand;

        let mut ore_deck = mining_def.ore_deck;
        deck_shuffle_hand(rng, &mut ore_deck);

        let mut encounter_tokens = HashMap::new();
        encounter_tokens.insert(
            Token::persistent(TokenType::MiningLightLevel),
            mining_def.initial_light_level,
        );
        encounter_tokens.insert(Token::persistent(TokenType::MiningYield), 0);

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

    fn start_herbalism_inner(
        &mut self,
        encounter_card_id: usize,
        herbalism_def: HerbalismDef,
        _rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let state = HerbalismEncounterState {
            round: 1,
            encounter_card_id,
            outcome: EncounterOutcome::Undecided,
            plant_hand: herbalism_def.plant_hand,
            rewards: herbalism_def.rewards,
        };
        self.current_encounter = Some(EncounterState::Herbalism(state));
        self.encounter_phase = types::EncounterPhase::InEncounter;
        Ok(())
    }

    fn start_woodcutting_inner(
        &mut self,
        encounter_card_id: usize,
        woodcutting_def: WoodcuttingDef,
        _rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let state = WoodcuttingEncounterState {
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

    fn start_fishing_inner(
        &mut self,
        encounter_card_id: usize,
        fishing_def: FishingDef,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        use crate::library::game_state::deck_shuffle_hand;

        let mut fish_deck = fishing_def.fish_deck;
        deck_shuffle_hand(rng, &mut fish_deck);

        let mut encounter_tokens = HashMap::new();
        encounter_tokens.insert(
            Token::persistent(TokenType::FishingRangeMin),
            fishing_def.valid_range_min,
        );
        encounter_tokens.insert(
            Token::persistent(TokenType::FishingRangeMax),
            fishing_def.valid_range_max,
        );

        let state = FishingEncounterState {
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
}
