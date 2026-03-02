use super::action_log::ActionLog;
use super::types::{
    ActionEntry, ActionPayload, CardCounts, CardKind, ConcreteEffect, ConcreteEffectCost,
    EncounterKind, EncounterOutcome, EncounterState,
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
        Some(super::types::CardEffectKind::GainTokens {
            cap_min,
            cap_max,
            gain_min_percent,
            gain_max_percent,
            costs,
            ..
        }) => {
            let r_cap = roll_range(rng, cap_min, cap_max);
            let r_gain = roll_range_u32(rng, gain_min_percent, gain_max_percent);
            let value = r_cap * r_gain as i64 / 100;
            let costs = costs
                .iter()
                .map(|c| ConcreteEffectCost {
                    cost_type: c.cost_type.clone(),
                    rolled_percent: roll_range_u32(rng, c.min_percent, c.max_percent),
                })
                .collect();
            (value, costs, Some(r_cap), Some(r_gain))
        }
        Some(super::types::CardEffectKind::LoseTokens {
            min, max, costs, ..
        }) => {
            let value = roll_range(rng, min, max);
            let costs = costs
                .iter()
                .map(|c| ConcreteEffectCost {
                    cost_type: c.cost_type.clone(),
                    rolled_percent: roll_range_u32(rng, c.min_percent, c.max_percent),
                })
                .collect();
            (value, costs, None, None)
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
            kind: super::types::CardEffectKind::LoseTokens {
                target: super::types::EffectTarget::OnOpponent,
                token_type: super::types::TokenType::Health,
                min: 400,
                max: 600,
                costs: vec![],
                duration: super::types::TokenLifecycle::PersistentCounter,
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
            kind: super::types::CardEffectKind::GainTokens {
                target: super::types::EffectTarget::OnSelf,
                token_type: super::types::TokenType::Shield,
                cap_min: 200,
                cap_max: 400,
                gain_min_percent: 100,
                gain_max_percent: 100,
                costs: vec![],
                duration: super::types::TokenLifecycle::PersistentCounter,
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
            kind: super::types::CardEffectKind::GainTokens {
                target: super::types::EffectTarget::OnSelf,
                token_type: super::types::TokenType::Stamina,
                cap_min: 150,
                cap_max: 250,
                gain_min_percent: 100,
                gain_max_percent: 100,
                costs: vec![],
                duration: super::types::TokenLifecycle::PersistentCounter,
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
            kind: super::types::CardEffectKind::LoseTokens {
                target: super::types::EffectTarget::OnOpponent,
                token_type: super::types::TokenType::Health,
                min: 200,
                max: 400,
                costs: vec![],
                duration: super::types::TokenLifecycle::PersistentCounter,
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
            kind: super::types::CardEffectKind::GainTokens {
                target: super::types::EffectTarget::OnSelf,
                token_type: super::types::TokenType::Shield,
                cap_min: 150,
                cap_max: 250,
                gain_min_percent: 100,
                gain_max_percent: 100,
                costs: vec![],
                duration: super::types::TokenLifecycle::PersistentCounter,
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
            kind: super::types::CardEffectKind::GainTokens {
                target: super::types::EffectTarget::OnSelf,
                token_type: super::types::TokenType::Stamina,
                cap_min: 80,
                cap_max: 120,
                gain_min_percent: 100,
                gain_max_percent: 100,
                costs: vec![],
                duration: super::types::TokenLifecycle::PersistentCounter,
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
                costs: vec![],
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

    // Balanced mining card (id 13): moderate ore damage and protection
    lib.add_card(
        CardKind::Mining {
            mining_effect: super::types::MiningCardEffect {
                ore_damage: 300,
                durability_prevent: 200,
                stamina_cost: 0,
                costs: vec![],
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

    // Protective mining card (id 14): low ore damage, high protection
    lib.add_card(
        CardKind::Mining {
            mining_effect: super::types::MiningCardEffect {
                ore_damage: 100,
                durability_prevent: 300,
                stamina_cost: 0,
                costs: vec![],
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
                durability_cost: 100,
                costs: vec![],
                match_mode: super::types::HerbalismMatchMode::Or {
                    types: vec![super::types::PlantCharacteristic::Fragile],
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

    // Medium herbalism card (id 17): targets 2 characteristics
    lib.add_card(
        CardKind::Herbalism {
            herbalism_effect: super::types::HerbalismCardEffect {
                durability_cost: 100,
                costs: vec![],
                match_mode: super::types::HerbalismMatchMode::Or {
                    types: vec![
                        super::types::PlantCharacteristic::Thorny,
                        super::types::PlantCharacteristic::Aromatic,
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

    // Broad herbalism card (id 18): targets 3 characteristics
    lib.add_card(
        CardKind::Herbalism {
            herbalism_effect: super::types::HerbalismCardEffect {
                durability_cost: 100,
                costs: vec![],
                match_mode: super::types::HerbalismMatchMode::Or {
                    types: vec![
                        super::types::PlantCharacteristic::Bitter,
                        super::types::PlantCharacteristic::Luminous,
                        super::types::PlantCharacteristic::Fragile,
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
                costs: vec![],
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

    // HeavyChop card (id 21): value 5
    lib.add_card(
        CardKind::Woodcutting {
            woodcutting_effect: super::types::WoodcuttingCardEffect {
                chop_types: vec![super::types::ChopType::HeavyChop],
                chop_values: vec![5],
                durability_cost: 100,
                stamina_cost: 0,
                costs: vec![],
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

    // MediumChop card (id 22): value 3
    lib.add_card(
        CardKind::Woodcutting {
            woodcutting_effect: super::types::WoodcuttingCardEffect {
                chop_types: vec![super::types::ChopType::MediumChop],
                chop_values: vec![3],
                durability_cost: 100,
                stamina_cost: 0,
                costs: vec![],
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

    // PrecisionChop card (id 23): value 7
    lib.add_card(
        CardKind::Woodcutting {
            woodcutting_effect: super::types::WoodcuttingCardEffect {
                chop_types: vec![super::types::ChopType::PrecisionChop],
                chop_values: vec![7],
                durability_cost: 100,
                stamina_cost: 0,
                costs: vec![],
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
                values: vec![200],
                durability_cost: 100,
                costs: vec![],
                modify_range_min: 0,
                modify_range_max: 0,
                modify_fish_amount: 0,
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
    // Card id 26: Medium value fishing card
    lib.add_card(
        CardKind::Fishing {
            fishing_effect: super::types::FishingCardEffect {
                values: vec![400],
                durability_cost: 100,
                costs: vec![],
                modify_range_min: 0,
                modify_range_max: 0,
                modify_fish_amount: 0,
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
    // Card id 27: High value fishing card
    lib.add_card(
        CardKind::Fishing {
            fishing_effect: super::types::FishingCardEffect {
                values: vec![700],
                durability_cost: 100,
                costs: vec![],
                modify_range_min: 0,
                modify_range_max: 0,
                modify_fish_amount: 0,
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
            kind: super::types::CardEffectKind::LoseTokens {
                target: super::types::EffectTarget::OnOpponent,
                token_type: super::types::TokenType::Health,
                min: 700,
                max: 900,
                costs: vec![super::types::CardEffectCost {
                    cost_type: super::types::TokenType::Stamina,
                    min_percent: 30,
                    max_percent: 50,
                }],
                duration: super::types::TokenLifecycle::PersistentCounter,
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
            kind: super::types::CardEffectKind::GainTokens {
                target: super::types::EffectTarget::OnSelf,
                token_type: super::types::TokenType::Shield,
                cap_min: 350,
                cap_max: 550,
                gain_min_percent: 100,
                gain_max_percent: 100,
                costs: vec![super::types::CardEffectCost {
                    cost_type: super::types::TokenType::Stamina,
                    min_percent: 30,
                    max_percent: 50,
                }],
                duration: super::types::TokenLifecycle::PersistentCounter,
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
                costs: vec![],
                gains: vec![],
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
                costs: vec![],
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

    // ---- New mining expansion cards ----

    // High damage + high protection, stamina cost card
    lib.add_card(
        CardKind::Mining {
            mining_effect: super::types::MiningCardEffect {
                ore_damage: 600,
                durability_prevent: 300,
                stamina_cost: 0,
                costs: vec![super::types::GatheringCost {
                    cost_type: super::types::TokenType::Stamina,
                    amount: 200,
                }],
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

    // Very high damage, no protection, higher stamina cost
    lib.add_card(
        CardKind::Mining {
            mining_effect: super::types::MiningCardEffect {
                ore_damage: 1000,
                durability_prevent: 0,
                stamina_cost: 0,
                costs: vec![super::types::GatheringCost {
                    cost_type: super::types::TokenType::Stamina,
                    amount: 300,
                }],
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

    // Mining rest card: grants stamina, no damage/protection
    lib.add_card(
        CardKind::Mining {
            mining_effect: super::types::MiningCardEffect {
                ore_damage: 0,
                durability_prevent: 0,
                stamina_cost: 0,
                costs: vec![],
                gains: vec![super::types::GatheringCost {
                    cost_type: super::types::TokenType::Stamina,
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

    // ---- New fishing expansion cards ----

    // Card id 35: Widen range — reduces min value token (makes winning easier)
    lib.add_card(
        CardKind::Fishing {
            fishing_effect: super::types::FishingCardEffect {
                values: vec![],
                durability_cost: 100,
                costs: vec![],
                modify_range_min: -150,
                modify_range_max: 0,
                modify_fish_amount: 0,
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

    // Card id 36: Widen range — increases max value token (makes winning easier)
    lib.add_card(
        CardKind::Fishing {
            fishing_effect: super::types::FishingCardEffect {
                values: vec![],
                durability_cost: 100,
                costs: vec![],
                modify_range_min: 0,
                modify_range_max: 150,
                modify_fish_amount: 0,
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

    // Card id 37: Cost card — narrows range but has multiple values (3 values)
    lib.add_card(
        CardKind::Fishing {
            fishing_effect: super::types::FishingCardEffect {
                values: vec![100, 350, 600],
                durability_cost: 100,
                costs: vec![super::types::GatheringCost {
                    cost_type: super::types::TokenType::Stamina,
                    amount: 150,
                }],
                modify_range_min: 50,
                modify_range_max: -50,
                modify_fish_amount: 0,
                gains: vec![],
            },
        },
        CardCounts {
            library: 0,
            deck: 3,
            hand: 1,
            discard: 0,
        },
    );

    // Card id 38: Increase fish amount
    lib.add_card(
        CardKind::Fishing {
            fishing_effect: super::types::FishingCardEffect {
                values: vec![],
                durability_cost: 100,
                costs: vec![],
                modify_range_min: 0,
                modify_range_max: 0,
                modify_fish_amount: 1,
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

    // Card id 39: Multi-value but decreases fish amount (cost based on spread)
    lib.add_card(
        CardKind::Fishing {
            fishing_effect: super::types::FishingCardEffect {
                values: vec![150, 400, 650],
                durability_cost: 100,
                costs: vec![super::types::GatheringCost {
                    cost_type: super::types::TokenType::Stamina,
                    amount: 100,
                }],
                modify_range_min: 0,
                modify_range_max: 0,
                modify_fish_amount: -1,
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

    // Card id 40: Rest card — grants stamina, no values (pure rest action)
    lib.add_card(
        CardKind::Fishing {
            fishing_effect: super::types::FishingCardEffect {
                values: vec![],
                durability_cost: 50,
                costs: vec![],
                modify_range_min: 0,
                modify_range_max: 0,
                modify_fish_amount: 0,
                gains: vec![super::types::GatheringCost {
                    cost_type: super::types::TokenType::Stamina,
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

    // Card id 41: Stamina cost card with multiple values
    lib.add_card(
        CardKind::Fishing {
            fishing_effect: super::types::FishingCardEffect {
                values: vec![50, 250, 500, 750],
                durability_cost: 100,
                costs: vec![super::types::GatheringCost {
                    cost_type: super::types::TokenType::Stamina,
                    amount: 200,
                }],
                modify_range_min: 0,
                modify_range_max: 0,
                modify_fish_amount: 0,
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

    // ---- New woodcutting expansion cards ----

    // No-cost card: SplitChop value 4 (benefit: 2 = 1 type + 1 value)
    lib.add_card(
        CardKind::Woodcutting {
            woodcutting_effect: super::types::WoodcuttingCardEffect {
                chop_types: vec![super::types::ChopType::SplitChop],
                chop_values: vec![4],
                durability_cost: 100,
                stamina_cost: 0,
                costs: vec![],
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

    // No-cost card: LightChop+MediumChop, values 1,6 (benefit: 4 = 2 types + 2 values)
    lib.add_card(
        CardKind::Woodcutting {
            woodcutting_effect: super::types::WoodcuttingCardEffect {
                chop_types: vec![
                    super::types::ChopType::LightChop,
                    super::types::ChopType::MediumChop,
                ],
                chop_values: vec![1, 6],
                durability_cost: 100,
                stamina_cost: 0,
                costs: vec![],
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

    // Cost card: 3 types, 3 values (benefit: 6), moderate stamina cost
    lib.add_card(
        CardKind::Woodcutting {
            woodcutting_effect: super::types::WoodcuttingCardEffect {
                chop_types: vec![
                    super::types::ChopType::HeavyChop,
                    super::types::ChopType::MediumChop,
                    super::types::ChopType::PrecisionChop,
                ],
                chop_values: vec![3, 5, 7],
                durability_cost: 100,
                stamina_cost: 0,
                costs: vec![super::types::GatheringCost {
                    cost_type: super::types::TokenType::Stamina,
                    amount: 150,
                }],
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

    // Cost card: 4 types, 4 values (benefit: 8), higher stamina cost
    lib.add_card(
        CardKind::Woodcutting {
            woodcutting_effect: super::types::WoodcuttingCardEffect {
                chop_types: vec![
                    super::types::ChopType::LightChop,
                    super::types::ChopType::HeavyChop,
                    super::types::ChopType::MediumChop,
                    super::types::ChopType::SplitChop,
                ],
                chop_values: vec![2, 4, 6, 8],
                durability_cost: 100,
                stamina_cost: 0,
                costs: vec![super::types::GatheringCost {
                    cost_type: super::types::TokenType::Stamina,
                    amount: 250,
                }],
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
            woodcutting_effect: super::types::WoodcuttingCardEffect {
                chop_types: vec![],
                chop_values: vec![],
                durability_cost: 50,
                stamina_cost: 0,
                costs: vec![],
                gains: vec![super::types::GatheringCost {
                    cost_type: super::types::TokenType::Stamina,
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

    // ---- New herbalism expansion cards ----

    // Card id 42: MostCommon card — removes the most common characteristic (limit 1)
    lib.add_card(
        CardKind::Herbalism {
            herbalism_effect: super::types::HerbalismCardEffect {
                durability_cost: 100,
                costs: vec![super::types::GatheringCost {
                    cost_type: super::types::TokenType::Stamina,
                    amount: 150,
                }],
                match_mode: super::types::HerbalismMatchMode::MostCommon {
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

    // Card id 43: LeastCommon card — removes the least common characteristic (limit 1)
    lib.add_card(
        CardKind::Herbalism {
            herbalism_effect: super::types::HerbalismCardEffect {
                durability_cost: 100,
                costs: vec![super::types::GatheringCost {
                    cost_type: super::types::TokenType::Stamina,
                    amount: 150,
                }],
                match_mode: super::types::HerbalismMatchMode::LeastCommon {
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

    // Card id 44: AND-based multi-type card — removes only plants matching ALL listed types
    lib.add_card(
        CardKind::Herbalism {
            herbalism_effect: super::types::HerbalismCardEffect {
                durability_cost: 100,
                costs: vec![super::types::GatheringCost {
                    cost_type: super::types::TokenType::Stamina,
                    amount: 100,
                }],
                match_mode: super::types::HerbalismMatchMode::And {
                    types: vec![
                        super::types::PlantCharacteristic::Fragile,
                        super::types::PlantCharacteristic::Thorny,
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

    // Card id 45: Stamina rest card for herbalism
    lib.add_card(
        CardKind::Herbalism {
            herbalism_effect: super::types::HerbalismCardEffect {
                durability_cost: 50,
                costs: vec![],
                match_mode: super::types::HerbalismMatchMode::Or { types: vec![] },
                gains: vec![super::types::GatheringCost {
                    cost_type: super::types::TokenType::Stamina,
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

    if let Err(errors) = lib.validate_card_effects() {
        panic!("Library card effect validation failed: {:?}", errors);
    }

    lib
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

    /// Check if player can pay all costs on a card's effects. Deducts costs if affordable.
    pub(crate) fn check_and_deduct_costs(
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

    /// Check and deduct a list of gathering costs. All costs must be affordable.
    pub(crate) fn check_and_deduct_gathering_costs(
        costs: &[super::types::GatheringCost],
        token_balances: &mut HashMap<super::types::Token, i64>,
    ) -> Result<(), String> {
        Self::preview_gathering_costs(costs, token_balances)?;
        for cost in costs {
            if cost.amount > 0 {
                let entry = super::types::token_entry_by_type(token_balances, &cost.cost_type);
                *entry -= cost.amount;
            }
        }
        Ok(())
    }

    /// Check if player can afford gathering costs without deducting.
    pub fn preview_gathering_costs(
        costs: &[super::types::GatheringCost],
        token_balances: &HashMap<super::types::Token, i64>,
    ) -> Result<(), String> {
        for cost in costs {
            if cost.amount <= 0 {
                continue;
            }
            let balance = super::types::token_balance_by_type(token_balances, &cost.cost_type);
            if balance < cost.amount {
                return Err(format!(
                    "Insufficient {:?}: need {} but have {}",
                    cost.cost_type, cost.amount, balance
                ));
            }
        }
        Ok(())
    }

    /// Merge explicit gathering costs with legacy inline cost fields.
    /// Legacy costs are only included if their amount > 0 and the same TokenType
    /// is not already present in the explicit costs vec.
    pub fn merge_gathering_costs(
        costs: &[super::types::GatheringCost],
        legacy: &[(super::types::TokenType, i64)],
    ) -> Vec<super::types::GatheringCost> {
        let mut merged: Vec<super::types::GatheringCost> = costs.to_vec();
        for (token_type, amount) in legacy {
            if *amount > 0 && !merged.iter().any(|c| c.cost_type == *token_type) {
                merged.push(super::types::GatheringCost {
                    cost_type: token_type.clone(),
                    amount: *amount,
                });
            }
        }
        merged
    }

    /// Draw player cards from deck to hand per card type, recycling discard if needed.
    pub(crate) fn draw_player_cards_by_type(
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
    pub(crate) fn draw_player_cards_of_kind(
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

    /// Abort a non-combat encounter: mark as lost, transition to Scouting.
    pub fn abort_encounter(&mut self) {
        self.last_encounter_result = Some(EncounterOutcome::PlayerLost);
        self.encounter_results.push(EncounterOutcome::PlayerLost);
        self.current_encounter = None;
        self.encounter_phase = super::types::EncounterPhase::Scouting;
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
