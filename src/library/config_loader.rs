//! Config loader: reads JSON config files embedded at compile time and populates the Library.
//!
//! JSON files under `configurations/` are embedded via `include_str!()` and parsed at
//! `GameState` initialization. Effect templates get positional IDs; player cards and
//! encounters reference effects by name, resolved during loading.

use std::collections::HashMap;

use super::config::*;
use super::game_state::roll_concrete_effect;
use super::types::{
    self, CardKind, CombatantDef, CraftingDef, EncounterKind, EnemyCardDef, EnemyCraftingCard,
    FishCard, FishingDef, HerbalismDef, MilestoneDef, MiningDef, OreCard, PlantCard, Token,
    TokenType, WoodcuttingDef,
};
use super::Library;

// ---- Compile-time JSON embedding ----

static TOKENS_JSON: &str = include_str!("../../configurations/general/tokens.json");
static SHARED_EFFECTS_JSON: &str = include_str!("../../configurations/general/shared_effects.json");
static COMBAT_JSON: &str = include_str!("../../configurations/combat/cards.json");
static MINING_JSON: &str = include_str!("../../configurations/mining/cards.json");
static HERBALISM_JSON: &str = include_str!("../../configurations/herbalism/cards.json");
static WOODCUTTING_JSON: &str = include_str!("../../configurations/woodcutting/cards.json");
static FISHING_JSON: &str = include_str!("../../configurations/fishing/cards.json");
static REST_JSON: &str = include_str!("../../configurations/rest/cards.json");
static CRAFTING_JSON: &str = include_str!("../../configurations/crafting/cards.json");
static RESEARCH_JSON: &str = include_str!("../../configurations/research/cards.json");
static MILESTONE_JSON: &str = include_str!("../../configurations/milestone/cards.json");

/// Name→card_id mapping used for resolving effect references.
pub type EffectNameMap = HashMap<String, usize>;

/// Load initial token balances from configuration.
pub fn load_token_balances() -> HashMap<Token, i64> {
    let config: TokensConfig =
        serde_json::from_str(TOKENS_JSON).expect("Failed to parse tokens.json");
    let mut balances = HashMap::new();
    // Initialize all token types to 0
    for id in TokenType::all() {
        balances.insert(Token::persistent(id), 0i64);
    }
    // Override with configured values
    for (token_type, value) in config.initial_balances {
        balances.insert(Token::persistent(token_type), value);
    }
    balances
}

/// Load the full library from all JSON config files.
pub fn load_library(rng: &mut rand_pcg::Lcg64Xsh32) -> Library {
    let mut lib = Library::new();
    let mut name_map = EffectNameMap::new();

    // Phase 1: Shared effects
    load_discipline_config(&mut lib, rng, SHARED_EFFECTS_JSON, "shared", &mut name_map);

    // Phase 2: Discipline configs (order matters for card ID stability)
    load_discipline_config(&mut lib, rng, COMBAT_JSON, "combat", &mut name_map);
    load_discipline_config(&mut lib, rng, MINING_JSON, "mining", &mut name_map);
    load_discipline_config(&mut lib, rng, HERBALISM_JSON, "herbalism", &mut name_map);
    load_discipline_config(
        &mut lib,
        rng,
        WOODCUTTING_JSON,
        "woodcutting",
        &mut name_map,
    );
    load_discipline_config(&mut lib, rng, FISHING_JSON, "fishing", &mut name_map);
    load_discipline_config(&mut lib, rng, REST_JSON, "rest", &mut name_map);
    load_discipline_config(&mut lib, rng, CRAFTING_JSON, "crafting", &mut name_map);
    load_discipline_config(&mut lib, rng, RESEARCH_JSON, "research", &mut name_map);
    load_discipline_config(&mut lib, rng, MILESTONE_JSON, "milestone", &mut name_map);

    if let Err(errors) = lib.validate_card_effects() {
        panic!("Library card effect validation failed: {:?}", errors);
    }

    lib
}

/// Load a single discipline config: parse JSON, register cards in order.
fn load_discipline_config(
    lib: &mut Library,
    rng: &mut rand_pcg::Lcg64Xsh32,
    json: &str,
    prefix: &str,
    name_map: &mut EffectNameMap,
) {
    let config: DisciplineConfig =
        serde_json::from_str(json).unwrap_or_else(|e| panic!("Failed to parse {prefix}: {e}"));

    for entry in &config.cards {
        match entry {
            CardEntry::Effect(effect) => {
                let kind = match &effect.owner {
                    EffectOwner::Player => CardKind::PlayerCardEffect {
                        kind: effect.kind.clone(),
                    },
                    EffectOwner::Enemy => CardKind::EnemyCardEffect {
                        kind: effect.kind.clone(),
                    },
                };
                let card_id = lib.add_card(
                    kind,
                    effect.counts.clone(),
                    rng,
                    effect.valid_disciplines.clone(),
                );
                let full_name = format!("{}:{}", prefix, effect.name);
                name_map.insert(full_name, card_id);
            }
            CardEntry::PlayerCard(card) => {
                let effects = resolve_effect_refs(&card.effect_refs, name_map, rng, lib);
                let kind = match card.card_kind {
                    PlayerCardKind::Attack => CardKind::Attack { effects },
                    PlayerCardKind::Defence => CardKind::Defence { effects },
                    PlayerCardKind::Resource => CardKind::Resource { effects },
                    PlayerCardKind::Mining => CardKind::Mining { effects },
                    PlayerCardKind::Herbalism => CardKind::Herbalism { effects },
                    PlayerCardKind::Woodcutting => CardKind::Woodcutting { effects },
                    PlayerCardKind::Fishing => CardKind::Fishing { effects },
                    PlayerCardKind::Rest => CardKind::Rest { effects },
                    PlayerCardKind::Crafting => CardKind::Crafting { effects },
                    PlayerCardKind::Research => CardKind::Research { effects },
                };
                lib.add_card(
                    kind,
                    card.counts.clone(),
                    rng,
                    card.valid_disciplines.clone(),
                );
            }
            CardEntry::Encounter(encounter) => {
                let encounter_kind =
                    build_encounter_kind(&encounter.encounter_def, name_map, rng, lib);
                lib.add_card(
                    CardKind::Encounter { encounter_kind },
                    encounter.counts.clone(),
                    rng,
                    encounter.valid_disciplines.clone(),
                );
            }
        }
    }
}

/// Resolve a list of effect name references to ConcreteEffects via rolling.
fn resolve_effect_refs(
    refs: &[String],
    name_map: &EffectNameMap,
    rng: &mut rand_pcg::Lcg64Xsh32,
    lib: &Library,
) -> Vec<types::ConcreteEffect> {
    refs.iter()
        .map(|name| {
            let effect_id = *name_map
                .get(name)
                .unwrap_or_else(|| panic!("Unknown effect reference: {name}"));
            roll_concrete_effect(rng, effect_id, lib)
        })
        .collect()
}

/// Build an EncounterKind from config, resolving all effect references.
fn build_encounter_kind(
    def: &EncounterDefConfig,
    name_map: &EffectNameMap,
    rng: &mut rand_pcg::Lcg64Xsh32,
    lib: &Library,
) -> EncounterKind {
    match def {
        EncounterDefConfig::Combat { combatant } => EncounterKind::Combat {
            combatant_def: build_combatant_def(combatant, name_map, rng, lib),
        },
        EncounterDefConfig::Mining { mining_def } => EncounterKind::Mining {
            mining_def: build_mining_def(mining_def, name_map, rng, lib),
        },
        EncounterDefConfig::Herbalism { herbalism_def } => EncounterKind::Herbalism {
            herbalism_def: build_herbalism_def(herbalism_def, name_map, rng, lib),
        },
        EncounterDefConfig::Woodcutting { woodcutting_def } => EncounterKind::Woodcutting {
            woodcutting_def: build_woodcutting_def(woodcutting_def),
        },
        EncounterDefConfig::Fishing { fishing_def } => EncounterKind::Fishing {
            fishing_def: build_fishing_def(fishing_def, name_map, rng, lib),
        },
        EncounterDefConfig::Rest { rest_def } => EncounterKind::Rest {
            rest_def: rest_def.clone(),
        },
        EncounterDefConfig::Crafting { crafting_def } => EncounterKind::Crafting {
            crafting_def: build_crafting_def(crafting_def, name_map, rng, lib),
        },
        EncounterDefConfig::Research { research_def } => EncounterKind::Research {
            research_def: research_def.clone(),
        },
        EncounterDefConfig::Milestone { milestone_def } => EncounterKind::Milestone {
            milestone_def: build_milestone_def(milestone_def, name_map, rng, lib),
        },
    }
}

fn build_combatant_def(
    config: &CombatantConfig,
    name_map: &EffectNameMap,
    rng: &mut rand_pcg::Lcg64Xsh32,
    lib: &Library,
) -> CombatantDef {
    let to_token_map = |m: &HashMap<TokenType, u64>| -> HashMap<Token, u64> {
        m.iter()
            .map(|(tt, v)| (Token::persistent(tt.clone()), *v))
            .collect()
    };

    CombatantDef {
        initial_tokens: to_token_map(&config.initial_tokens),
        attack_deck: build_enemy_deck(&config.attack_deck, name_map, rng, lib),
        defence_deck: build_enemy_deck(&config.defence_deck, name_map, rng, lib),
        resource_deck: build_enemy_deck(&config.resource_deck, name_map, rng, lib),
    }
}

fn build_enemy_deck(
    entries: &[EnemyDeckEntryConfig],
    name_map: &EffectNameMap,
    rng: &mut rand_pcg::Lcg64Xsh32,
    lib: &Library,
) -> Vec<EnemyCardDef> {
    entries
        .iter()
        .map(|entry| EnemyCardDef {
            effects: resolve_effect_refs(&entry.effect_refs, name_map, rng, lib),
            counts: entry.counts.clone(),
        })
        .collect()
}

fn build_mining_def(
    config: &MiningDefConfig,
    name_map: &EffectNameMap,
    rng: &mut rand_pcg::Lcg64Xsh32,
    lib: &Library,
) -> MiningDef {
    MiningDef {
        initial_light_level: config.initial_light_level,
        ore_deck: config
            .ore_deck
            .iter()
            .map(|entry| OreCard {
                effects: resolve_effect_refs(&entry.effect_refs, name_map, rng, lib),
                counts: entry.counts.clone(),
            })
            .collect(),
    }
}

fn build_herbalism_def(
    config: &HerbalismDefConfig,
    name_map: &EffectNameMap,
    rng: &mut rand_pcg::Lcg64Xsh32,
    lib: &Library,
) -> HerbalismDef {
    let rewards = config
        .rewards
        .iter()
        .map(|(tt, v)| (Token::persistent(tt.clone()), *v))
        .collect();
    HerbalismDef {
        plant_hand: config
            .plant_hand
            .iter()
            .map(|p| PlantCard {
                characteristics: p.characteristics.clone(),
                effects: resolve_effect_refs(&p.effect_refs, name_map, rng, lib),
                counts: p.counts.clone(),
            })
            .collect(),
        rewards,
    }
}

fn build_woodcutting_def(config: &WoodcuttingDefConfig) -> WoodcuttingDef {
    let base_rewards = config
        .base_rewards
        .iter()
        .map(|(tt, v)| (Token::persistent(tt.clone()), *v))
        .collect();
    WoodcuttingDef {
        max_plays: config.max_plays,
        base_rewards,
    }
}

fn build_fishing_def(
    config: &FishingDefConfig,
    name_map: &EffectNameMap,
    rng: &mut rand_pcg::Lcg64Xsh32,
    lib: &Library,
) -> FishingDef {
    let rewards = config
        .rewards
        .iter()
        .map(|(tt, v)| (Token::persistent(tt.clone()), *v))
        .collect();
    FishingDef {
        valid_range_min: config.valid_range_min,
        valid_range_max: config.valid_range_max,
        max_turns: config.max_turns,
        win_turns_needed: config.win_turns_needed,
        fish_deck: config
            .fish_deck
            .iter()
            .map(|f| FishCard {
                value: f.value,
                effects: resolve_effect_refs(&f.effect_refs, name_map, rng, lib),
                counts: f.counts.clone(),
            })
            .collect(),
        rewards,
    }
}

fn build_crafting_def(
    config: &CraftingDefConfig,
    name_map: &EffectNameMap,
    rng: &mut rand_pcg::Lcg64Xsh32,
    lib: &Library,
) -> CraftingDef {
    CraftingDef {
        initial_crafting_tokens: config.initial_crafting_tokens,
        enemy_crafting_deck: config
            .enemy_crafting_deck
            .iter()
            .map(|entry| EnemyCraftingCard {
                effects: resolve_effect_refs(&entry.effect_refs, name_map, rng, lib),
                counts: entry.counts.clone(),
            })
            .collect(),
    }
}

fn build_milestone_def(
    config: &MilestoneDefConfig,
    name_map: &EffectNameMap,
    rng: &mut rand_pcg::Lcg64Xsh32,
    lib: &Library,
) -> MilestoneDef {
    MilestoneDef {
        inner_encounter_kind: Box::new(build_encounter_kind(
            &config.inner_encounter,
            name_map,
            rng,
            lib,
        )),
        discipline: config.discipline.clone(),
        tier: config.tier,
        insight_cost: config.insight_cost,
    }
}
