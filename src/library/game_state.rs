use super::action_log::ActionLog;
use super::types::{
    ActionEntry, ActionPayload, CardCounts, CardKind, CombatEncounterState, ConcreteEffect,
    ConcreteEffectCost, EncounterKind, EncounterOutcome, EncounterState, HerbalismEncounterState,
    MiningEncounterState,
};
use super::Library;
use std::collections::HashMap;
use std::sync::atomic::Ordering;

fn roll_range(rng: &mut rand_pcg::Lcg64Xsh32, min: i64, max: i64) -> i64 {
    use rand::RngCore;
    if min == max {
        return min;
    }
    let (lo, hi) = if min < max { (min, max) } else { (max, min) };
    let range = (hi - lo + 1) as u64;
    lo + (rng.next_u64() % range) as i64
}

fn roll_range_u32(rng: &mut rand_pcg::Lcg64Xsh32, min: u32, max: u32) -> u32 {
    use rand::RngCore;
    if min == max {
        return min;
    }
    let (lo, hi) = if min < max { (min, max) } else { (max, min) };
    let range = (hi - lo + 1) as u64;
    lo + (rng.next_u64() % range) as u32
}

fn roll_concrete_effect(
    rng: &mut rand_pcg::Lcg64Xsh32,
    effect_id: usize,
    library: &Library,
) -> ConcreteEffect {
    let kind = library.resolve_effect(effect_id);
    let (rolled_value, rolled_costs, rolled_cap, rolled_gain_percent) = match kind {
        Some(super::types::CardEffectKind::ChangeTokens {
            min,
            max,
            costs,
            cap_min,
            cap_max,
            gain_min_percent,
            gain_max_percent,
            ..
        }) => {
            let value = roll_range(rng, min, max);
            let costs = costs
                .iter()
                .map(|c| ConcreteEffectCost {
                    cost_type: c.cost_type.clone(),
                    rolled_percent: roll_range_u32(rng, c.min_percent, c.max_percent),
                })
                .collect();
            let r_cap = cap_min.zip(cap_max).map(|(lo, hi)| roll_range(rng, lo, hi));
            let r_gain = gain_min_percent
                .zip(gain_max_percent)
                .map(|(lo, hi)| roll_range_u32(rng, lo, hi));
            (value, costs, r_cap, r_gain)
        }
        _ => (0, vec![], None, None),
    };
    ConcreteEffect {
        effect_id,
        rolled_value,
        rolled_costs,
        rolled_cap,
        rolled_gain_percent,
    }
}

fn initialize_library(rng: &mut rand_pcg::Lcg64Xsh32) -> Library {
    let mut lib = Library::new();

    // ---- Player CardEffect deck entries (templates with ranges) ----

    // id 0: Player "deal damage" effect (range: 400-600, old: 5)
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: super::types::CardEffectKind::ChangeTokens {
                target: super::types::EffectTarget::OnOpponent,
                token_type: super::types::TokenType::Health,
                min: -600,
                max: -400,
                costs: vec![],
                duration: super::types::TokenLifecycle::PersistentCounter,
                cap_min: None,
                cap_max: None,
                gain_min_percent: None,
                gain_max_percent: None,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // id 1: Player "grant shield" effect (range: 200-400, old: 3)
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: super::types::CardEffectKind::ChangeTokens {
                target: super::types::EffectTarget::OnSelf,
                token_type: super::types::TokenType::Shield,
                min: 200,
                max: 400,
                costs: vec![],
                duration: super::types::TokenLifecycle::PersistentCounter,
                cap_min: None,
                cap_max: None,
                gain_min_percent: None,
                gain_max_percent: None,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // id 2: Player "grant stamina" effect (range: 150-250, old: 2)
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: super::types::CardEffectKind::ChangeTokens {
                target: super::types::EffectTarget::OnSelf,
                token_type: super::types::TokenType::Stamina,
                min: 150,
                max: 250,
                costs: vec![],
                duration: super::types::TokenLifecycle::PersistentCounter,
                cap_min: None,
                cap_max: None,
                gain_min_percent: None,
                gain_max_percent: None,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // id 3: Player "draw 1 attack, 1 defence, 2 resource" effect
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: super::types::CardEffectKind::DrawCards {
                attack: 1,
                defence: 1,
                resource: 2,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // ---- Enemy CardEffect deck entries (templates with ranges) ----

    // id 4: Enemy "deal damage" effect (range: 200-400, old: 3)
    lib.add_card(
        CardKind::EnemyCardEffect {
            kind: super::types::CardEffectKind::ChangeTokens {
                target: super::types::EffectTarget::OnOpponent,
                token_type: super::types::TokenType::Health,
                min: -400,
                max: -200,
                costs: vec![],
                duration: super::types::TokenLifecycle::PersistentCounter,
                cap_min: None,
                cap_max: None,
                gain_min_percent: None,
                gain_max_percent: None,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // id 5: Enemy "grant shield" effect (range: 150-250, old: 2)
    lib.add_card(
        CardKind::EnemyCardEffect {
            kind: super::types::CardEffectKind::ChangeTokens {
                target: super::types::EffectTarget::OnSelf,
                token_type: super::types::TokenType::Shield,
                min: 150,
                max: 250,
                costs: vec![],
                duration: super::types::TokenLifecycle::PersistentCounter,
                cap_min: None,
                cap_max: None,
                gain_min_percent: None,
                gain_max_percent: None,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // id 6: Enemy "grant stamina" effect (range: 80-120, old: 1)
    lib.add_card(
        CardKind::EnemyCardEffect {
            kind: super::types::CardEffectKind::ChangeTokens {
                target: super::types::EffectTarget::OnSelf,
                token_type: super::types::TokenType::Stamina,
                min: 80,
                max: 120,
                costs: vec![],
                duration: super::types::TokenLifecycle::PersistentCounter,
                cap_min: None,
                cap_max: None,
                gain_min_percent: None,
                gain_max_percent: None,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // id 7: Enemy "draw 1 attack, 1 defence, 2 resource" effect
    lib.add_card(
        CardKind::EnemyCardEffect {
            kind: super::types::CardEffectKind::DrawCards {
                attack: 1,
                defence: 1,
                resource: 2,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // ---- Player action cards (concrete rolled values from CardEffect ranges) ----

    // Attack card (id 8): deals damage to opponent
    lib.add_card(
        CardKind::Attack {
            effects: vec![roll_concrete_effect(rng, 0, &lib)],
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
    );

    // Defence card (id 9): grants shield to self
    lib.add_card(
        CardKind::Defence {
            effects: vec![roll_concrete_effect(rng, 1, &lib)],
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
    );

    // Resource card (id 10): grants stamina to self, draws cards
    lib.add_card(
        CardKind::Resource {
            effects: vec![
                roll_concrete_effect(rng, 2, &lib),
                roll_concrete_effect(rng, 3, &lib),
            ],
        },
        CardCounts {
            library: 0,
            deck: 35,
            hand: 5,
            discard: 0,
        },
    );

    // Combat encounter: Gnome (id 11) — enemy health scaled 20→2000
    lib.add_card(
        CardKind::Encounter {
            encounter_kind: super::types::EncounterKind::Combat {
                combatant_def: super::types::CombatantDef {
                    initial_tokens: HashMap::from([
                        (
                            super::types::Token::persistent(super::types::TokenType::Health),
                            2000,
                        ),
                        (
                            super::types::Token::persistent(super::types::TokenType::MaxHealth),
                            2000,
                        ),
                    ]),
                    attack_deck: vec![super::types::EnemyCardDef {
                        effects: vec![roll_concrete_effect(rng, 4, &lib)],
                        counts: super::types::DeckCounts {
                            deck: 0,
                            hand: 10,
                            discard: 0,
                        },
                    }],
                    defence_deck: vec![super::types::EnemyCardDef {
                        effects: vec![roll_concrete_effect(rng, 5, &lib)],
                        counts: super::types::DeckCounts {
                            deck: 0,
                            hand: 10,
                            discard: 0,
                        },
                    }],
                    resource_deck: vec![super::types::EnemyCardDef {
                        effects: vec![
                            roll_concrete_effect(rng, 6, &lib),
                            roll_concrete_effect(rng, 7, &lib),
                        ],
                        counts: super::types::DeckCounts {
                            deck: 0,
                            hand: 10,
                            discard: 0,
                        },
                    }],
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

    // ---- Mining player cards (scaled by ~100x) ----

    // Aggressive mining card (id 12): high ore damage, no protection
    lib.add_card(
        CardKind::Mining {
            mining_effect: super::types::MiningCardEffect {
                ore_damage: 500,
                durability_prevent: 0,
                stamina_cost: 0,
            },
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
    );

    // Balanced mining card (id 13): moderate ore damage and protection
    lib.add_card(
        CardKind::Mining {
            mining_effect: super::types::MiningCardEffect {
                ore_damage: 300,
                durability_prevent: 200,
                stamina_cost: 0,
            },
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
    );

    // Protective mining card (id 14): low ore damage, high protection
    lib.add_card(
        CardKind::Mining {
            mining_effect: super::types::MiningCardEffect {
                ore_damage: 100,
                durability_prevent: 300,
                stamina_cost: 0,
            },
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
    );

    // Mining encounter: Iron Ore (id 15) — scaled by ~100x
    lib.add_card(
        CardKind::Encounter {
            encounter_kind: super::types::EncounterKind::Mining {
                mining_def: super::types::MiningDef {
                    initial_tokens: HashMap::from([(
                        super::types::Token::persistent(super::types::TokenType::OreHealth),
                        1500,
                    )]),
                    ore_deck: vec![
                        super::types::OreCard {
                            durability_damage: 0,
                            counts: super::types::DeckCounts {
                                deck: 0,
                                hand: 6,
                                discard: 0,
                            },
                        },
                        super::types::OreCard {
                            durability_damage: 100,
                            counts: super::types::DeckCounts {
                                deck: 0,
                                hand: 8,
                                discard: 0,
                            },
                        },
                        super::types::OreCard {
                            durability_damage: 200,
                            counts: super::types::DeckCounts {
                                deck: 0,
                                hand: 4,
                                discard: 0,
                            },
                        },
                        super::types::OreCard {
                            durability_damage: 300,
                            counts: super::types::DeckCounts {
                                deck: 0,
                                hand: 2,
                                discard: 0,
                            },
                        },
                    ],
                    rewards: HashMap::from([(
                        super::types::Token::persistent(super::types::TokenType::Ore),
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

    // ---- Herbalism player cards (durability_cost scaled by ~100x) ----

    // Narrow herbalism card (id 16): targets 1 characteristic, low durability cost
    lib.add_card(
        CardKind::Herbalism {
            herbalism_effect: super::types::HerbalismCardEffect {
                target_characteristics: vec![super::types::PlantCharacteristic::Fragile],
                durability_cost: 100,
            },
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
    );

    // Medium herbalism card (id 17): targets 2 characteristics
    lib.add_card(
        CardKind::Herbalism {
            herbalism_effect: super::types::HerbalismCardEffect {
                target_characteristics: vec![
                    super::types::PlantCharacteristic::Thorny,
                    super::types::PlantCharacteristic::Aromatic,
                ],
                durability_cost: 100,
            },
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
    );

    // Broad herbalism card (id 18): targets 3 characteristics
    lib.add_card(
        CardKind::Herbalism {
            herbalism_effect: super::types::HerbalismCardEffect {
                target_characteristics: vec![
                    super::types::PlantCharacteristic::Bitter,
                    super::types::PlantCharacteristic::Luminous,
                    super::types::PlantCharacteristic::Fragile,
                ],
                durability_cost: 100,
            },
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
    );

    // Herbalism encounter: Meadow Herb (id 19) — rewards scaled by ~100x
    lib.add_card(
        CardKind::Encounter {
            encounter_kind: super::types::EncounterKind::Herbalism {
                herbalism_def: super::types::HerbalismDef {
                    plant_hand: vec![
                        super::types::PlantCard {
                            characteristics: vec![super::types::PlantCharacteristic::Fragile],
                            counts: super::types::DeckCounts {
                                deck: 0,
                                hand: 1,
                                discard: 0,
                            },
                        },
                        super::types::PlantCard {
                            characteristics: vec![
                                super::types::PlantCharacteristic::Thorny,
                                super::types::PlantCharacteristic::Aromatic,
                            ],
                            counts: super::types::DeckCounts {
                                deck: 0,
                                hand: 1,
                                discard: 0,
                            },
                        },
                        super::types::PlantCard {
                            characteristics: vec![
                                super::types::PlantCharacteristic::Bitter,
                                super::types::PlantCharacteristic::Luminous,
                            ],
                            counts: super::types::DeckCounts {
                                deck: 0,
                                hand: 1,
                                discard: 0,
                            },
                        },
                        super::types::PlantCard {
                            characteristics: vec![
                                super::types::PlantCharacteristic::Fragile,
                                super::types::PlantCharacteristic::Thorny,
                            ],
                            counts: super::types::DeckCounts {
                                deck: 0,
                                hand: 1,
                                discard: 0,
                            },
                        },
                        super::types::PlantCard {
                            characteristics: vec![super::types::PlantCharacteristic::Luminous],
                            counts: super::types::DeckCounts {
                                deck: 0,
                                hand: 1,
                                discard: 0,
                            },
                        },
                    ],
                    rewards: HashMap::from([(
                        super::types::Token::persistent(super::types::TokenType::Plant),
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

    // ---- Woodcutting player cards (durability_cost scaled by ~100x) ----

    // LightChop card (id 20): value 2
    lib.add_card(
        CardKind::Woodcutting {
            woodcutting_effect: super::types::WoodcuttingCardEffect {
                chop_types: vec![super::types::ChopType::LightChop],
                chop_values: vec![2],
                durability_cost: 100,
                stamina_cost: 0,
            },
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 2,
            discard: 0,
        },
    );

    // HeavyChop card (id 21): value 5
    lib.add_card(
        CardKind::Woodcutting {
            woodcutting_effect: super::types::WoodcuttingCardEffect {
                chop_types: vec![super::types::ChopType::HeavyChop],
                chop_values: vec![5],
                durability_cost: 100,
                stamina_cost: 0,
            },
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 1,
            discard: 0,
        },
    );

    // MediumChop card (id 22): value 3
    lib.add_card(
        CardKind::Woodcutting {
            woodcutting_effect: super::types::WoodcuttingCardEffect {
                chop_types: vec![super::types::ChopType::MediumChop],
                chop_values: vec![3],
                durability_cost: 100,
                stamina_cost: 0,
            },
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 1,
            discard: 0,
        },
    );

    // PrecisionChop card (id 23): value 7
    lib.add_card(
        CardKind::Woodcutting {
            woodcutting_effect: super::types::WoodcuttingCardEffect {
                chop_types: vec![super::types::ChopType::PrecisionChop],
                chop_values: vec![7],
                durability_cost: 100,
                stamina_cost: 0,
            },
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 1,
            discard: 0,
        },
    );

    // Woodcutting encounter: Oak Tree (id 24) — rewards scaled by ~100x
    lib.add_card(
        CardKind::Encounter {
            encounter_kind: super::types::EncounterKind::Woodcutting {
                woodcutting_def: super::types::WoodcuttingDef {
                    max_plays: 8,
                    base_rewards: HashMap::from([(
                        super::types::Token::persistent(super::types::TokenType::Lumber),
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

    // ---- Fishing player cards (values and durability_cost scaled by ~100x) ----
    // Card id 25: Low value fishing card
    lib.add_card(
        CardKind::Fishing {
            fishing_effect: super::types::FishingCardEffect {
                value: 200,
                durability_cost: 100,
            },
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
    );
    // Card id 26: Medium value fishing card
    lib.add_card(
        CardKind::Fishing {
            fishing_effect: super::types::FishingCardEffect {
                value: 400,
                durability_cost: 100,
            },
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
    );
    // Card id 27: High value fishing card
    lib.add_card(
        CardKind::Fishing {
            fishing_effect: super::types::FishingCardEffect {
                value: 700,
                durability_cost: 100,
            },
        },
        CardCounts {
            library: 0,
            deck: 10,
            hand: 5,
            discard: 0,
        },
    );

    // Fishing encounter: River Spot (id 28) — values scaled by ~100x
    lib.add_card(
        CardKind::Encounter {
            encounter_kind: super::types::EncounterKind::Fishing {
                fishing_def: super::types::FishingDef {
                    valid_range_min: 100,
                    valid_range_max: 300,
                    max_turns: 8,
                    win_turns_needed: 4,
                    fish_deck: vec![
                        super::types::FishCard {
                            value: 100,
                            counts: super::types::DeckCounts {
                                deck: 0,
                                hand: 6,
                                discard: 0,
                            },
                        },
                        super::types::FishCard {
                            value: 300,
                            counts: super::types::DeckCounts {
                                deck: 0,
                                hand: 6,
                                discard: 0,
                            },
                        },
                        super::types::FishCard {
                            value: 500,
                            counts: super::types::DeckCounts {
                                deck: 0,
                                hand: 4,
                                discard: 0,
                            },
                        },
                        super::types::FishCard {
                            value: 700,
                            counts: super::types::DeckCounts {
                                deck: 0,
                                hand: 2,
                                discard: 0,
                            },
                        },
                    ],
                    rewards: HashMap::from([(
                        super::types::Token::persistent(super::types::TokenType::Fish),
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

    // ---- Cost card effect templates and variants (Step 9.2) ----

    // id 29: Powerful "deal damage" effect with Stamina cost (range: 700-900, cost: 30-50%)
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: super::types::CardEffectKind::ChangeTokens {
                target: super::types::EffectTarget::OnOpponent,
                token_type: super::types::TokenType::Health,
                min: -900,
                max: -700,
                costs: vec![super::types::CardEffectCost {
                    cost_type: super::types::TokenType::Stamina,
                    min_percent: 30,
                    max_percent: 50,
                }],
                duration: super::types::TokenLifecycle::PersistentCounter,
                cap_min: None,
                cap_max: None,
                gain_min_percent: None,
                gain_max_percent: None,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // id 30: Powerful "grant shield" effect with Stamina cost (range: 350-550, cost: 30-50%)
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: super::types::CardEffectKind::ChangeTokens {
                target: super::types::EffectTarget::OnSelf,
                token_type: super::types::TokenType::Shield,
                min: 350,
                max: 550,
                costs: vec![super::types::CardEffectCost {
                    cost_type: super::types::TokenType::Stamina,
                    min_percent: 30,
                    max_percent: 50,
                }],
                duration: super::types::TokenLifecycle::PersistentCounter,
                cap_min: None,
                cap_max: None,
                gain_min_percent: None,
                gain_max_percent: None,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // Cost Attack card (id 31): more powerful but costs Stamina
    lib.add_card(
        CardKind::Attack {
            effects: vec![roll_concrete_effect(rng, 29, &lib)],
        },
        CardCounts {
            library: 0,
            deck: 5,
            hand: 2,
            discard: 0,
        },
    );

    // Cost Defence card (id 32): more powerful but costs Stamina
    lib.add_card(
        CardKind::Defence {
            effects: vec![roll_concrete_effect(rng, 30, &lib)],
        },
        CardCounts {
            library: 0,
            deck: 5,
            hand: 2,
            discard: 0,
        },
    );

    // Cost Mining card (id 33): high ore damage, costs stamina
    lib.add_card(
        CardKind::Mining {
            mining_effect: super::types::MiningCardEffect {
                ore_damage: 800,
                durability_prevent: 0,
                stamina_cost: 100,
            },
        },
        CardCounts {
            library: 0,
            deck: 5,
            hand: 2,
            discard: 0,
        },
    );

    // Cost Woodcutting card (id 34): HeavyChop+LightChop combo, costs stamina
    lib.add_card(
        CardKind::Woodcutting {
            woodcutting_effect: super::types::WoodcuttingCardEffect {
                chop_types: vec![
                    super::types::ChopType::HeavyChop,
                    super::types::ChopType::LightChop,
                ],
                chop_values: vec![5, 3],
                durability_cost: 100,
                stamina_cost: 100,
            },
        },
        CardCounts {
            library: 0,
            deck: 5,
            hand: 1,
            discard: 0,
        },
    );

    if let Err(errors) = lib.validate_card_effects() {
        panic!("Library card effect validation failed: {:?}", errors);
    }

    lib
}

/// Apply card effects to combat using concrete rolled values.
/// Only processes ChangeTokens effects; DrawCards effects are handled separately.
fn apply_card_effects(
    effects: &[ConcreteEffect],
    is_player: bool,
    player_tokens: &mut HashMap<super::types::Token, i64>,
    combat: &mut CombatEncounterState,
    library: &Library,
) {
    for effect in effects {
        let kind = match library.resolve_effect(effect.effect_id) {
            Some(resolved) => resolved,
            None => continue,
        };

        let (target, token_type) = match &kind {
            super::types::CardEffectKind::ChangeTokens {
                target, token_type, ..
            } => (target, token_type),
            super::types::CardEffectKind::DrawCards { .. } => continue,
        };

        let amount = effect.rolled_value;

        let target_tokens = match (target, is_player) {
            (super::types::EffectTarget::OnSelf, true)
            | (super::types::EffectTarget::OnOpponent, false) => &mut *player_tokens,
            (super::types::EffectTarget::OnOpponent, true)
            | (super::types::EffectTarget::OnSelf, false) => &mut combat.enemy_tokens,
        };

        if *token_type == super::types::TokenType::Health && amount < 0 {
            let damage = -amount;
            // Dodge absorbs first (timing-based, expires after Defending phase)
            let dodge = target_tokens
                .get(&super::types::Token::dodge())
                .copied()
                .unwrap_or(0);
            let dodge_absorbed = dodge.min(damage);
            target_tokens.insert(
                super::types::Token::dodge(),
                (dodge - dodge_absorbed).max(0),
            );
            let after_dodge = damage - dodge_absorbed;
            // Shield absorbs next (persists for encounter, blocks 1:1)
            let shield_key = super::types::Token::persistent(super::types::TokenType::Shield);
            let shield = target_tokens.get(&shield_key).copied().unwrap_or(0);
            let shield_absorbed = shield.min(after_dodge);
            target_tokens.insert(shield_key, (shield - shield_absorbed).max(0));
            let remaining_damage = after_dodge - shield_absorbed;
            if remaining_damage > 0 {
                let health = target_tokens
                    .entry(super::types::Token::persistent(
                        super::types::TokenType::Health,
                    ))
                    .or_insert(0);
                *health = (*health - remaining_damage).max(0);
            }
        } else {
            // For token-granting effects with a cap: granted = cap * gain_percent / 100,
            // clamped so balance does not exceed cap.
            let grant_amount = match (effect.rolled_cap, effect.rolled_gain_percent) {
                (Some(cap), Some(pct)) => {
                    let raw_gain = cap * pct as i64 / 100;
                    let key = super::types::Token::persistent(token_type.clone());
                    let current = target_tokens.get(&key).copied().unwrap_or(0);
                    raw_gain.min((cap - current).max(0))
                }
                _ => amount,
            };
            let entry = target_tokens
                .entry(super::types::Token::persistent(token_type.clone()))
                .or_insert(0);
            *entry = (*entry + grant_amount).max(0);
        }
    }
}

/// Check if combat has ended (either side at 0 health).
fn check_combat_end(
    player_tokens: &HashMap<super::types::Token, i64>,
    combat: &mut CombatEncounterState,
) {
    let player_health = player_tokens
        .get(&super::types::Token::persistent(
            super::types::TokenType::Health,
        ))
        .copied()
        .unwrap_or(0);
    let enemy_health = combat
        .enemy_tokens
        .get(&super::types::Token::persistent(
            super::types::TokenType::Health,
        ))
        .copied()
        .unwrap_or(0);

    if enemy_health <= 0 || player_health <= 0 {
        combat.outcome = if enemy_health <= 0 && player_health > 0 {
            EncounterOutcome::PlayerWon
        } else if player_health <= 0 && enemy_health > 0 {
            EncounterOutcome::PlayerLost
        } else {
            EncounterOutcome::PlayerWon // Draw defaults to player
        };
    }
}

/// Minimal in-memory game state driven by the library's mutator API.
#[derive(Debug, Clone)]
pub struct GameState {
    pub action_log: std::sync::Arc<ActionLog>,
    pub token_balances: HashMap<super::types::Token, i64>,
    pub library: Library,
    pub current_encounter: Option<EncounterState>,
    pub encounter_phase: super::types::EncounterPhase,
    pub last_encounter_result: Option<EncounterOutcome>,
    pub encounter_results: Vec<EncounterOutcome>,
}

/// Evaluate played woodcutting cards and return (pattern_name, reward_multiplier).
/// Poker-inspired patterns adapted for 8 cards using ChopType counts.
fn evaluate_best_pattern(played: &[super::types::PlayedWoodcuttingCard]) -> (String, f64) {
    use super::types::ChopType;
    use std::collections::HashMap;

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
        return ("Eight of a Kind".to_string(), 5.0);
    }
    // Seven of a Kind
    if max_type_count >= 7 {
        return ("Seven of a Kind".to_string(), 4.0);
    }
    // Perfect Straight: 8 sequential values
    if longest_straight >= 8 {
        return ("Perfect Straight".to_string(), 4.0);
    }
    // Six of a Kind
    if max_type_count >= 6 {
        return ("Six of a Kind".to_string(), 3.5);
    }
    // Long Straight: 6-7 sequential values
    if longest_straight >= 6 {
        return ("Long Straight".to_string(), 3.0);
    }
    // Full Set: all 5 chop types present
    if distinct_types >= 5 {
        return ("Full Set".to_string(), 3.0);
    }
    // Five of a Kind
    if max_type_count >= 5 {
        return ("Five of a Kind".to_string(), 2.5);
    }
    // Four of a Kind with Pair: 4+ of one type plus 2+ of another
    if max_type_count >= 4 && sorted_type_counts.len() >= 2 && sorted_type_counts[1] >= 2 {
        return ("Full House".to_string(), 2.5);
    }
    // Four of a Kind
    if max_type_count >= 4 {
        return ("Four of a Kind".to_string(), 2.0);
    }
    // Short Straight: 4-5 sequential values
    if longest_straight >= 4 {
        return ("Short Straight".to_string(), 2.0);
    }
    // Two Pair Types: 2 types with 3+ each
    if sorted_type_counts.len() >= 2 && sorted_type_counts[0] >= 3 && sorted_type_counts[1] >= 3 {
        return ("Two Pair Types".to_string(), 1.8);
    }
    // Value Quads: 4+ of same value
    if freq_list.first().copied().unwrap_or(0) >= 4 {
        return ("Value Quads".to_string(), 1.8);
    }
    // Triple of a Kind
    if max_type_count >= 3 {
        return ("Triple".to_string(), 1.5);
    }
    // Value Triples
    if freq_list.first().copied().unwrap_or(0) >= 3 {
        return ("Value Triple".to_string(), 1.3);
    }
    // Pair (2+ of a type)
    if max_type_count >= 2 {
        return ("Pair".to_string(), 1.2);
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
    pub fn new() -> Self {
        use rand::SeedableRng;
        let mut rng = rand_pcg::Lcg64Xsh32::from_entropy();
        Self::new_with_rng(&mut rng)
    }

    pub fn new_with_rng(rng: &mut rand_pcg::Lcg64Xsh32) -> Self {
        let mut balances = HashMap::new();
        for id in super::types::TokenType::all() {
            balances.insert(super::types::Token::persistent(id), 0i64);
        }
        // Default Foresight controls area deck hand size
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::Foresight),
            3,
        );
        // Durabilities scaled by ~100x (100→10000)
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::MiningDurability),
            10000,
        );
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::HerbalismDurability),
            10000,
        );
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::WoodcuttingDurability),
            10000,
        );
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::FishingDurability),
            10000,
        );
        // Starting stamina
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::Stamina),
            1000,
        );
        // Max handsize tokens (player decks)
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::AttackMaxHand),
            10,
        );
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::DefenceMaxHand),
            10,
        );
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::ResourceMaxHand),
            10,
        );
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::MiningMaxHand),
            10,
        );
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::HerbalismMaxHand),
            10,
        );
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::WoodcuttingMaxHand),
            10,
        );
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::FishingMaxHand),
            10,
        );
        let _action_log = match std::env::var("ACTION_LOG_FILE") {
            Ok(path) => {
                #[allow(clippy::manual_unwrap_or_default)]
                let mut log = match super::action_log::ActionLog::load_from_file(&path) {
                    Ok(l) => l,
                    Err(_) => ActionLog::new(),
                };
                if let Ok(writer) =
                    crate::action::persistence::FileWriter::new(std::path::PathBuf::from(&path))
                {
                    log.set_writer(Some(writer));
                }
                log
            }
            Err(_) => ActionLog::new(),
        };
        Self {
            action_log: std::sync::Arc::new(ActionLog::new()),
            token_balances: balances,
            library: initialize_library(rng),
            current_encounter: None,
            encounter_phase: super::types::EncounterPhase::NoEncounter,
            last_encounter_result: None,
            encounter_results: Vec::new(),
        }
    }

    /// Append an action to the action log with optional metadata; returns the appended entry.
    pub fn append_action(&self, action_type: &str, payload: ActionPayload) -> ActionEntry {
        self.action_log.append(action_type, payload)
    }

    /// Initialize combat from a Library Encounter card.
    pub fn start_combat(
        &mut self,
        encounter_card_id: usize,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let lib_card = self
            .library
            .get(encounter_card_id)
            .ok_or_else(|| format!("Card {} not found in Library", encounter_card_id))?
            .clone();
        let combatant_def = match &lib_card.kind {
            CardKind::Encounter {
                encounter_kind: EncounterKind::Combat { combatant_def },
            } => combatant_def.clone(),
            _ => {
                return Err(format!(
                    "Card {} is not a combat encounter",
                    encounter_card_id
                ))
            }
        };
        let mut enemy_attack_deck = combatant_def.attack_deck.clone();
        let mut enemy_defence_deck = combatant_def.defence_deck.clone();
        let mut enemy_resource_deck = combatant_def.resource_deck.clone();
        Self::enemy_shuffle_hand(rng, &mut enemy_attack_deck);
        Self::enemy_shuffle_hand(rng, &mut enemy_defence_deck);
        Self::enemy_shuffle_hand(rng, &mut enemy_resource_deck);
        let snapshot = CombatEncounterState {
            round: 1,
            phase: super::types::CombatPhase::Defending,
            enemy_tokens: combatant_def
                .initial_tokens
                .iter()
                .map(|(k, v)| (k.clone(), *v as i64))
                .collect(),
            encounter_card_id,
            outcome: EncounterOutcome::Undecided,
            enemy_attack_deck,
            enemy_defence_deck,
            enemy_resource_deck,
        };
        self.current_encounter = Some(EncounterState::Combat(snapshot));
        self.encounter_phase = super::types::EncounterPhase::InEncounter;
        Ok(())
    }

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
        Self::ore_shuffle_hand(rng, &mut ore_deck);
        let state = MiningEncounterState {
            round: 1,
            encounter_card_id,
            outcome: EncounterOutcome::Undecided,
            ore_tokens: mining_def.initial_tokens,
            ore_deck,
            rewards: mining_def.rewards,
        };
        self.current_encounter = Some(EncounterState::Mining(state));
        self.encounter_phase = super::types::EncounterPhase::InEncounter;
        Ok(())
    }

    /// Resolve a player card play against the current combat encounter.
    pub fn resolve_player_card(
        &mut self,
        card_id: usize,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let combat = match &mut self.current_encounter {
            Some(EncounterState::Combat(c)) => c,
            _ => return Err("No active combat".to_string()),
        };
        let lib_card = self
            .library
            .get(card_id)
            .ok_or_else(|| format!("Card {} not found in Library", card_id))?
            .clone();
        let effects = match &lib_card.kind {
            CardKind::Attack { effects }
            | CardKind::Defence { effects }
            | CardKind::Resource { effects } => effects.clone(),
            _ => return Err("Cannot play a non-action card".to_string()),
        };
        // Multi-effect evaluation: each effect is evaluated independently.
        // A previous effect can grant tokens that a later effect needs.
        // If an effect's cost cannot be paid, it is skipped (partial success).
        let (mut atk_draws, mut def_draws, mut res_draws) = (0u32, 0u32, 0u32);
        for effect in &effects {
            // Try to pay cost for this single effect
            if Self::check_and_deduct_costs(std::slice::from_ref(effect), &mut self.token_balances)
                .is_err()
            {
                continue;
            }
            if let Some(super::types::CardEffectKind::DrawCards {
                attack,
                defence,
                resource,
            }) = self.library.resolve_effect(effect.effect_id)
            {
                atk_draws += attack;
                def_draws += defence;
                res_draws += resource;
            }
            apply_card_effects(
                std::slice::from_ref(effect),
                true,
                &mut self.token_balances,
                combat,
                &self.library,
            );
        }
        check_combat_end(&self.token_balances, combat);
        if combat.outcome != EncounterOutcome::Undecided {
            self.last_encounter_result = Some(combat.outcome.clone());
            self.encounter_results.push(combat.outcome.clone());
            self.current_encounter = None;
            self.encounter_phase = super::types::EncounterPhase::Scouting;
        }
        self.draw_player_cards_by_type(atk_draws, def_draws, res_draws, rng);
        Ok(())
    }

    /// Check if player can pay all costs on a card's effects. Deducts costs if affordable.
    fn check_and_deduct_costs(
        effects: &[ConcreteEffect],
        token_balances: &mut HashMap<super::types::Token, i64>,
    ) -> Result<(), String> {
        Self::preview_costs(effects, token_balances)?;
        // Deduct costs (we know they're affordable from preview)
        for effect in effects {
            for cost in &effect.rolled_costs {
                let cost_amount =
                    (effect.rolled_value.unsigned_abs() * cost.rolled_percent as u64 / 100) as i64;
                let entry = super::types::token_entry_by_type(token_balances, &cost.cost_type);
                *entry -= cost_amount;
            }
        }
        Ok(())
    }

    /// Check if player can afford all costs without deducting. Used for pre-validation.
    pub fn preview_costs(
        effects: &[ConcreteEffect],
        token_balances: &HashMap<super::types::Token, i64>,
    ) -> Result<(), String> {
        let mut total_costs: HashMap<super::types::TokenType, i64> = HashMap::new();
        for effect in effects {
            for cost in &effect.rolled_costs {
                let cost_amount =
                    (effect.rolled_value.unsigned_abs() * cost.rolled_percent as u64 / 100) as i64;
                *total_costs.entry(cost.cost_type.clone()).or_insert(0) += cost_amount;
            }
        }
        for (cost_type, cost_amount) in &total_costs {
            let balance = super::types::token_balance_by_type(token_balances, cost_type);
            if balance < *cost_amount {
                return Err(format!(
                    "Insufficient {:?}: need {} but have {}",
                    cost_type, cost_amount, balance
                ));
            }
        }
        Ok(())
    }

    /// Check if player can pay stamina cost for a gathering card. Deducts if affordable.
    fn check_and_deduct_stamina_cost(
        stamina_cost: i64,
        token_balances: &mut HashMap<super::types::Token, i64>,
    ) -> Result<(), String> {
        Self::preview_stamina_cost(stamina_cost, token_balances)?;
        if stamina_cost > 0 {
            let entry = super::types::token_entry_by_type(
                token_balances,
                &super::types::TokenType::Stamina,
            );
            *entry -= stamina_cost;
        }
        Ok(())
    }

    /// Check if player can afford stamina cost without deducting. Used for pre-validation.
    pub fn preview_stamina_cost(
        stamina_cost: i64,
        token_balances: &HashMap<super::types::Token, i64>,
    ) -> Result<(), String> {
        if stamina_cost <= 0 {
            return Ok(());
        }
        let balance =
            super::types::token_balance_by_type(token_balances, &super::types::TokenType::Stamina);
        if balance < stamina_cost {
            return Err(format!(
                "Insufficient Stamina: need {} but have {}",
                stamina_cost, balance
            ));
        }
        Ok(())
    }

    /// Draw player cards from deck to hand per card type, recycling discard if needed.
    fn draw_player_cards_by_type(
        &mut self,
        attack: u32,
        defence: u32,
        resource: u32,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) {
        self.draw_player_cards_of_kind(
            attack,
            |k| matches!(k, CardKind::Attack { .. }),
            rng,
            Some(super::types::TokenType::AttackMaxHand),
        );
        self.draw_player_cards_of_kind(
            defence,
            |k| matches!(k, CardKind::Defence { .. }),
            rng,
            Some(super::types::TokenType::DefenceMaxHand),
        );
        self.draw_player_cards_of_kind(
            resource,
            |k| matches!(k, CardKind::Resource { .. }),
            rng,
            Some(super::types::TokenType::ResourceMaxHand),
        );
    }

    /// Draw `count` player cards of a specific kind from deck to hand.
    /// Recycles discard→deck for cards matching `kind_filter` when deck is empty.
    /// Respects max handsize token if provided.
    fn draw_player_cards_of_kind(
        &mut self,
        count: u32,
        kind_filter: fn(&CardKind) -> bool,
        rng: &mut rand_pcg::Lcg64Xsh32,
        max_hand_token: Option<super::types::TokenType>,
    ) {
        use rand::RngCore;
        for _ in 0..count {
            let drawable: Vec<usize> = self
                .library
                .cards
                .iter()
                .enumerate()
                .filter(|(_, c)| c.counts.deck > 0 && kind_filter(&c.kind))
                .map(|(i, _)| i)
                .collect();
            if drawable.is_empty() {
                // Recycle discard→deck for this card type
                for card in self.library.cards.iter_mut() {
                    if kind_filter(&card.kind) && card.counts.discard > 0 {
                        card.counts.deck += card.counts.discard;
                        card.counts.discard = 0;
                    }
                }
                let drawable: Vec<usize> = self
                    .library
                    .cards
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.counts.deck > 0 && kind_filter(&c.kind))
                    .map(|(i, _)| i)
                    .collect();
                if drawable.is_empty() {
                    return;
                }
                let pick = (rng.next_u64() as usize) % drawable.len();
                if self.handsize_reached(&kind_filter, &max_hand_token) {
                    continue;
                }
                let _ = self.library.draw(drawable[pick]);
            } else {
                let pick = (rng.next_u64() as usize) % drawable.len();
                if self.handsize_reached(&kind_filter, &max_hand_token) {
                    continue;
                }
                let _ = self.library.draw(drawable[pick]);
            }
        }
    }

    fn handsize_reached(
        &self,
        kind_filter: &fn(&CardKind) -> bool,
        max_hand_token: &Option<super::types::TokenType>,
    ) -> bool {
        if let Some(ref token) = max_hand_token {
            let max_hand = super::types::token_balance_by_type(&self.token_balances, token);
            let current_hand: u32 = self
                .library
                .cards
                .iter()
                .filter(|c| kind_filter(&c.kind))
                .map(|c| c.counts.hand)
                .sum();
            current_hand as i64 >= max_hand
        } else {
            false
        }
    }

    /// Resolve an enemy card play from hand in the current combat phase.
    /// Played cards move to discard. DrawCards effects trigger per-type enemy draws.
    pub fn resolve_enemy_play(&mut self, rng: &mut rand_pcg::Lcg64Xsh32) -> Result<(), String> {
        let combat = match &self.current_encounter {
            Some(EncounterState::Combat(c)) => c,
            _ => return Err("No active combat".to_string()),
        };
        let phase = combat.phase.clone();

        let combat = match &mut self.current_encounter {
            Some(EncounterState::Combat(c)) => c,
            _ => return Err("No active combat".to_string()),
        };
        let deck = match phase {
            super::types::CombatPhase::Attacking => &mut combat.enemy_attack_deck,
            super::types::CombatPhase::Defending => &mut combat.enemy_defence_deck,
            super::types::CombatPhase::Resourcing => &mut combat.enemy_resource_deck,
        };

        // Collect indices of cards with hand > 0
        let hand_indices: Vec<usize> = deck
            .iter()
            .enumerate()
            .filter(|(_, c)| c.counts.hand > 0)
            .map(|(i, _)| i)
            .collect();

        if !hand_indices.is_empty() {
            use rand::RngCore;
            let pick_idx = (rng.next_u64() as usize) % hand_indices.len();
            let card_idx = hand_indices[pick_idx];
            deck[card_idx].counts.hand -= 1;
            deck[card_idx].counts.discard += 1;
            let effects = deck[card_idx].effects.clone();

            let (mut atk_draws, mut def_draws, mut res_draws) = (0u32, 0u32, 0u32);
            for effect in &effects {
                if let Some(super::types::CardEffectKind::DrawCards {
                    attack,
                    defence,
                    resource,
                }) = self.library.resolve_effect(effect.effect_id)
                {
                    atk_draws += attack;
                    def_draws += defence;
                    res_draws += resource;
                }
            }

            apply_card_effects(
                &effects,
                false,
                &mut self.token_balances,
                combat,
                &self.library,
            );
            check_combat_end(&self.token_balances, combat);

            // Handle enemy draws per deck type
            if combat.outcome == EncounterOutcome::Undecided {
                Self::enemy_draw_n(rng, &mut combat.enemy_attack_deck, atk_draws);
                Self::enemy_draw_n(rng, &mut combat.enemy_defence_deck, def_draws);
                Self::enemy_draw_n(rng, &mut combat.enemy_resource_deck, res_draws);
            }

            if combat.outcome != EncounterOutcome::Undecided {
                self.last_encounter_result = Some(combat.outcome.clone());
                self.encounter_results.push(combat.outcome.clone());
                self.current_encounter = None;
                self.encounter_phase = super::types::EncounterPhase::Scouting;
            }
        }
        Ok(())
    }

    /// Draw `count` random cards from a single enemy deck to hand, recycling discard if needed.
    fn enemy_draw_n(
        rng: &mut rand_pcg::Lcg64Xsh32,
        deck: &mut [super::types::EnemyCardDef],
        count: u32,
    ) {
        for _ in 0..count {
            Self::enemy_draw_random(rng, deck);
        }
    }

    /// Shuffle enemy hand: move all cards to deck, then draw random cards back to hand.
    fn enemy_shuffle_hand(rng: &mut rand_pcg::Lcg64Xsh32, deck: &mut [super::types::EnemyCardDef]) {
        let target_hand: u32 = deck.iter().map(|c| c.counts.hand).sum();
        // Move all hand cards to deck
        for card in deck.iter_mut() {
            card.counts.deck += card.counts.hand;
            card.counts.hand = 0;
        }
        // Draw random cards until hand is full again
        for _ in 0..target_hand {
            Self::enemy_draw_random(rng, deck);
        }
    }

    /// Draw one random card from enemy deck to hand, recycling discard if needed.
    fn enemy_draw_random(rng: &mut rand_pcg::Lcg64Xsh32, deck: &mut [super::types::EnemyCardDef]) {
        use rand::RngCore;
        let total_deck: u32 = deck.iter().map(|c| c.counts.deck).sum();
        if total_deck == 0 {
            // Recycle discard to deck
            let total_discard: u32 = deck.iter().map(|c| c.counts.discard).sum();
            if total_discard == 0 {
                return;
            }
            for card in deck.iter_mut() {
                card.counts.deck += card.counts.discard;
                card.counts.discard = 0;
            }
        }
        // Pick a random card from deck (weighted by deck count)
        let total_deck: u32 = deck.iter().map(|c| c.counts.deck).sum();
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

    /// Advance combat phase to next (Defending → Attacking → Resourcing → Defending).
    pub fn advance_combat_phase(&mut self) -> Result<(), String> {
        let combat = match &mut self.current_encounter {
            Some(EncounterState::Combat(c)) => c,
            _ => return Err("No active combat".to_string()),
        };
        combat.phase = combat.phase.next();
        Ok(())
    }

    /// Resolve a player mining card play against the current mining encounter.
    /// Applies ore damage, stores durability prevent, auto-resolves ore play,
    /// draws cards for both sides, and checks encounter end.
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

        // Check and deduct stamina cost before playing
        Self::check_and_deduct_stamina_cost(mining_effect.stamina_cost, &mut self.token_balances)?;

        // Apply player mining card: damage ore
        let ore_defeated = {
            let mining = match &mut self.current_encounter {
                Some(EncounterState::Mining(m)) => m,
                _ => return Err("No active mining encounter".to_string()),
            };
            let ore_health_key =
                super::types::Token::persistent(super::types::TokenType::OreHealth);
            let ore_hp = mining.ore_tokens.entry(ore_health_key).or_insert(0);
            *ore_hp = (*ore_hp - mining_effect.ore_damage).max(0);
            *ore_hp <= 0
        };

        if ore_defeated {
            self.finish_mining_encounter(true);
            return Ok(());
        }

        // Auto-resolve ore play with the prevent value from the card just played
        self.resolve_ore_play(rng, mining_effect.durability_prevent);

        // Player draws a mining card
        self.draw_player_mining_card(rng);

        Ok(())
    }

    /// Ore plays a random card from hand, dealing durability damage minus prevent.
    /// Then draws a card from deck to hand.
    fn resolve_ore_play(&mut self, rng: &mut rand_pcg::Lcg64Xsh32, durability_prevent: i64) {
        use rand::RngCore;

        // Play ore card and extract damage info
        let (effective_damage, played) = {
            let mining = match &mut self.current_encounter {
                Some(EncounterState::Mining(m)) => m,
                _ => return,
            };
            let hand_indices: Vec<usize> = mining
                .ore_deck
                .iter()
                .enumerate()
                .filter(|(_, c)| c.counts.hand > 0)
                .map(|(i, _)| i)
                .collect();
            if hand_indices.is_empty() {
                return;
            }
            let pick_idx = (rng.next_u64() as usize) % hand_indices.len();
            let card_idx = hand_indices[pick_idx];
            mining.ore_deck[card_idx].counts.hand -= 1;
            mining.ore_deck[card_idx].counts.discard += 1;
            let raw_damage = mining.ore_deck[card_idx].durability_damage;
            let effective = (raw_damage - durability_prevent).max(0);
            mining.round += 1;
            (effective, true)
        };

        if !played {
            return;
        }

        // Apply durability damage to player
        let durability_key =
            super::types::Token::persistent(super::types::TokenType::MiningDurability);
        let durability = self
            .token_balances
            .entry(durability_key.clone())
            .or_insert(0);
        *durability = (*durability - effective_damage).max(0);

        // Ore draws a card
        if let Some(EncounterState::Mining(mining)) = &mut self.current_encounter {
            Self::ore_draw_random(rng, &mut mining.ore_deck);
        }

        // Check if player durability is depleted
        let durability = self
            .token_balances
            .get(&durability_key)
            .copied()
            .unwrap_or(0);
        if durability <= 0 {
            self.finish_mining_encounter(false);
        }
    }

    /// Finalize a mining encounter: grant rewards (win) or apply penalties (loss).
    fn finish_mining_encounter(&mut self, is_win: bool) {
        if is_win {
            let rewards = match &self.current_encounter {
                Some(EncounterState::Mining(m)) => m.rewards.clone(),
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
        self.encounter_phase = super::types::EncounterPhase::Scouting;
    }

    /// Abort a non-combat encounter: mark as lost, transition to Scouting.
    pub fn abort_encounter(&mut self) {
        self.last_encounter_result = Some(EncounterOutcome::PlayerLost);
        self.encounter_results.push(EncounterOutcome::PlayerLost);
        self.current_encounter = None;
        self.encounter_phase = super::types::EncounterPhase::Scouting;
    }

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
        self.encounter_phase = super::types::EncounterPhase::InEncounter;
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

        // Apply durability cost
        let durability_key =
            super::types::Token::persistent(super::types::TokenType::HerbalismDurability);
        let durability = self
            .token_balances
            .entry(durability_key.clone())
            .or_insert(0);
        *durability = (*durability - herbalism_effect.durability_cost).max(0);
        let durability_depleted = *durability <= 0;

        if durability_depleted {
            self.finish_herbalism_encounter(false);
            return Ok(());
        }

        // Remove plant cards sharing ≥1 characteristic with the player's card
        {
            let herbalism = match &mut self.current_encounter {
                Some(EncounterState::Herbalism(h)) => h,
                _ => return Err("No active herbalism encounter".to_string()),
            };
            for plant_card in &mut herbalism.plant_hand {
                if plant_card.counts.hand == 0 {
                    continue;
                }
                let shares_characteristic = plant_card
                    .characteristics
                    .iter()
                    .any(|c| herbalism_effect.target_characteristics.contains(c));
                if shares_characteristic {
                    plant_card.counts.hand = 0;
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
        }

        Ok(())
    }

    /// Finalize an herbalism encounter: grant rewards (win) or apply penalties (loss).
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
        self.encounter_phase = super::types::EncounterPhase::Scouting;
    }

    /// Draw one player herbalism card from deck to hand, recycling discard if needed.
    fn draw_player_herbalism_card(&mut self, rng: &mut rand_pcg::Lcg64Xsh32) {
        self.draw_player_cards_of_kind(
            1,
            |k| matches!(k, CardKind::Herbalism { .. }),
            rng,
            Some(super::types::TokenType::HerbalismMaxHand),
        );
    }

    /// Draw one player woodcutting card from deck to hand, recycling discard if needed.
    fn draw_player_woodcutting_card(&mut self, rng: &mut rand_pcg::Lcg64Xsh32) {
        self.draw_player_cards_of_kind(
            1,
            |k| matches!(k, CardKind::Woodcutting { .. }),
            rng,
            Some(super::types::TokenType::WoodcuttingMaxHand),
        );
    }

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
        let state = super::types::WoodcuttingEncounterState {
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
        self.encounter_phase = super::types::EncounterPhase::InEncounter;
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

        // Check and deduct stamina cost before playing
        Self::check_and_deduct_stamina_cost(
            woodcutting_effect.stamina_cost,
            &mut self.token_balances,
        )?;

        // Deduct durability cost
        let durability_key =
            super::types::Token::persistent(super::types::TokenType::WoodcuttingDurability);
        let durability = self.token_balances.entry(durability_key).or_insert(0);
        *durability = (*durability - woodcutting_effect.durability_cost).max(0);
        let durability_depleted = *durability <= 0;

        if durability_depleted {
            self.finish_woodcutting_encounter(false);
            return Ok(());
        }

        // Track the played card
        let played = super::types::PlayedWoodcuttingCard {
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
        }

        Ok(())
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
        self.encounter_phase = super::types::EncounterPhase::Scouting;
    }

    /// Draw one player mining card from deck to hand, recycling discard if needed.
    fn draw_player_mining_card(&mut self, rng: &mut rand_pcg::Lcg64Xsh32) {
        self.draw_player_cards_of_kind(
            1,
            |k| matches!(k, CardKind::Mining { .. }),
            rng,
            Some(super::types::TokenType::MiningMaxHand),
        );
    }

    /// Shuffle ore hand: move all to deck, redraw to original hand size.
    fn ore_shuffle_hand(rng: &mut rand_pcg::Lcg64Xsh32, deck: &mut [super::types::OreCard]) {
        let target_hand: u32 = deck.iter().map(|c| c.counts.hand).sum();
        for card in deck.iter_mut() {
            card.counts.deck += card.counts.hand;
            card.counts.hand = 0;
        }
        for _ in 0..target_hand {
            Self::ore_draw_random(rng, deck);
        }
    }

    /// Draw one random ore card from deck to hand, recycling discard if needed.
    fn ore_draw_random(rng: &mut rand_pcg::Lcg64Xsh32, deck: &mut [super::types::OreCard]) {
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

    /// Shuffle plant hand: move all to deck, redraw to original hand size.
    fn plant_shuffle_hand(rng: &mut rand_pcg::Lcg64Xsh32, deck: &mut [super::types::PlantCard]) {
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
    fn plant_draw_random(rng: &mut rand_pcg::Lcg64Xsh32, deck: &mut [super::types::PlantCard]) {
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
        let state = super::types::FishingEncounterState {
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
        self.encounter_phase = super::types::EncounterPhase::InEncounter;
        Ok(())
    }

    /// Resolve a player fishing card play: subtract values, range check, track wins, apply durability.
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

        // Deduct durability cost
        let durability_key =
            super::types::Token::persistent(super::types::TokenType::FishingDurability);
        let durability = self.token_balances.entry(durability_key).or_insert(0);
        *durability = (*durability - fishing_effect.durability_cost).max(0);
        let durability_depleted = *durability <= 0;

        if durability_depleted {
            self.finish_fishing_encounter(false);
            return Ok(());
        }

        // Auto-resolve fish play: pick random fish card from hand
        let fish_value = Self::fish_play_random(rng, &mut self.current_encounter);

        // Calculate result: (player_value - fish_value).max(0)
        let result = (fishing_effect.value - fish_value).max(0);

        // Check if result is within valid range
        let (valid_min, valid_max, win_turns_needed) = match &self.current_encounter {
            Some(EncounterState::Fishing(f)) => {
                (f.valid_range_min, f.valid_range_max, f.win_turns_needed)
            }
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
                fishing.turns_won += 1;
            }
            fishing.round += 1;
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
        }

        Ok(())
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
        self.encounter_phase = super::types::EncounterPhase::Scouting;
    }

    fn draw_player_fishing_card(&mut self, rng: &mut rand_pcg::Lcg64Xsh32) {
        self.draw_player_cards_of_kind(
            1,
            |k| matches!(k, CardKind::Fishing { .. }),
            rng,
            Some(super::types::TokenType::FishingMaxHand),
        );
    }

    fn fish_shuffle_hand(rng: &mut rand_pcg::Lcg64Xsh32, deck: &mut [super::types::FishCard]) {
        let target_hand: u32 = deck.iter().map(|c| c.counts.hand).sum();
        for card in deck.iter_mut() {
            card.counts.deck += card.counts.hand;
            card.counts.hand = 0;
        }
        for _ in 0..target_hand {
            Self::fish_draw_random(rng, deck);
        }
    }

    fn fish_draw_random(rng: &mut rand_pcg::Lcg64Xsh32, deck: &mut [super::types::FishCard]) {
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

    /// Reconstruct state from an existing action log.
    /// The RNG is initialized from the first `SetSeed` entry in the log.
    pub fn replay_from_log(log: &ActionLog) -> Self {
        use rand::SeedableRng;

        let mut gs = GameState::new();
        let mut rng = rand_pcg::Lcg64Xsh32::from_seed([0u8; 16]);

        for e in log.entries() {
            match &e.payload {
                ActionPayload::SetSeed { seed } => {
                    let mut seed_bytes = [0u8; 16];
                    seed_bytes[0..8].copy_from_slice(&seed.to_le_bytes());
                    seed_bytes[8..16].copy_from_slice(&seed.to_le_bytes());
                    rng = rand_pcg::Lcg64Xsh32::from_seed(seed_bytes);
                    let new_gs = GameState::new();
                    gs.library = new_gs.library;
                    gs.token_balances = new_gs.token_balances;
                    gs.current_encounter = None;
                    gs.encounter_phase = new_gs.encounter_phase;
                    gs.last_encounter_result = None;
                    gs.encounter_results.clear();
                }
                ActionPayload::DrawEncounter { encounter_id } => {
                    if let Ok(card_id) = encounter_id.parse::<usize>() {
                        let health_key =
                            super::types::Token::persistent(super::types::TokenType::Health);
                        if gs.token_balances.get(&health_key).copied().unwrap_or(0) == 0 {
                            gs.token_balances.insert(health_key, 20);
                        }
                        let _ = gs.library.play(card_id);
                        // Dispatch based on encounter kind
                        if let Some(lib_card) = gs.library.get(card_id) {
                            match &lib_card.kind {
                                CardKind::Encounter {
                                    encounter_kind: EncounterKind::Mining { .. },
                                } => {
                                    let _ = gs.start_mining_encounter(card_id, &mut rng);
                                }
                                CardKind::Encounter {
                                    encounter_kind: EncounterKind::Herbalism { .. },
                                } => {
                                    let _ = gs.start_herbalism_encounter(card_id, &mut rng);
                                }
                                CardKind::Encounter {
                                    encounter_kind: EncounterKind::Woodcutting { .. },
                                } => {
                                    let _ = gs.start_woodcutting_encounter(card_id, &mut rng);
                                }
                                CardKind::Encounter {
                                    encounter_kind: EncounterKind::Fishing { .. },
                                } => {
                                    let _ = gs.start_fishing_encounter(card_id, &mut rng);
                                }
                                _ => {
                                    let _ = gs.start_combat(card_id, &mut rng);
                                }
                            }
                        }
                    }
                }
                ActionPayload::PlayCard { card_id } => {
                    let _ = gs.library.play(*card_id);
                    match &gs.current_encounter {
                        Some(EncounterState::Combat(_)) => {
                            let _ = gs.resolve_player_card(*card_id, &mut rng);
                            if gs.current_encounter.is_some() {
                                let _ = gs.resolve_enemy_play(&mut rng);
                                if gs.current_encounter.is_some() {
                                    let _ = gs.advance_combat_phase();
                                }
                            }
                        }
                        Some(EncounterState::Mining(_)) => {
                            let _ = gs.resolve_player_mining_card(*card_id, &mut rng);
                        }
                        Some(EncounterState::Herbalism(_)) => {
                            let _ = gs.resolve_player_herbalism_card(*card_id, &mut rng);
                        }
                        Some(EncounterState::Woodcutting(_)) => {
                            let _ = gs.resolve_player_woodcutting_card(*card_id, &mut rng);
                        }
                        Some(EncounterState::Fishing(_)) => {
                            let _ = gs.resolve_player_fishing_card(*card_id, &mut rng);
                        }
                        None => {}
                    }
                }
                ActionPayload::ApplyScouting { .. } => {
                    if let Some(ref enc) = gs.current_encounter {
                        let enc_id = enc.encounter_card_id();
                        let _ = gs.library.return_to_deck(enc_id);
                    }
                    let foresight = gs
                        .token_balances
                        .get(&super::types::Token::persistent(
                            super::types::TokenType::Foresight,
                        ))
                        .copied()
                        .unwrap_or(3) as usize;
                    gs.library.encounter_draw_to_hand(foresight);
                    gs.encounter_phase = super::types::EncounterPhase::NoEncounter;
                }
                ActionPayload::AbortEncounter => {
                    gs.abort_encounter();
                }
            }
            match gs.action_log.entries.lock() {
                Ok(mut g) => g.push(e.clone()),
                Err(err) => err.into_inner().push(e.clone()),
            };
            let cur = gs.action_log.seq.load(Ordering::SeqCst);
            if cur < e.seq {
                gs.action_log.seq.store(e.seq, Ordering::SeqCst);
            }
        }
        gs
    }

    /// Graceful shutdown helper to flush and close any background writer.
    pub fn shutdown(&self) {
        if let Some(w) = &self.action_log.writer {
            w.close();
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
