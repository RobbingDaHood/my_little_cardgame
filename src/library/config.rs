//! JSON configuration types for externalized card, effect, and encounter definitions.
//!
//! These types are deserialized from JSON files under `configurations/` at compile time
//! and used by the config loader to populate the Library.

use rocket::serde::Deserialize;
use std::collections::HashMap;

use super::types::{CardCounts, CardEffectKind, DeckCounts, Discipline, RestDef, TokenType};

/// Top-level configuration for initial token balances.
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct TokensConfig {
    pub initial_balances: HashMap<TokenType, i64>,
}

/// Top-level configuration for a discipline's cards.
/// Cards are an ordered list processed sequentially to preserve card ID assignment order.
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct DisciplineConfig {
    pub cards: Vec<CardEntry>,
}

/// A single card entry — tagged union preserving registration order.
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde", tag = "type")]
pub enum CardEntry {
    /// A card effect template (PlayerCardEffect or EnemyCardEffect).
    #[serde(rename = "effect")]
    Effect(CardEffectConfig),
    /// A player card that references effect templates by name.
    #[serde(rename = "player_card")]
    PlayerCard(PlayerCardConfig),
    /// An encounter card.
    #[serde(rename = "encounter")]
    Encounter(EncounterCardConfig),
}

/// Whether an effect template is for the player or the enemy.
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub enum EffectOwner {
    Player,
    Enemy,
}

/// A named card effect template (PlayerCardEffect or EnemyCardEffect).
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CardEffectConfig {
    pub name: String,
    pub owner: EffectOwner,
    pub kind: CardEffectKind,
    #[serde(default = "default_effect_counts")]
    pub counts: CardCounts,
    #[serde(default)]
    pub valid_disciplines: Vec<Discipline>,
}

fn default_effect_counts() -> CardCounts {
    CardCounts {
        library: 1,
        deck: 0,
        hand: 0,
        discard: 0,
    }
}

/// Which card kind a player card belongs to.
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub enum PlayerCardKind {
    Attack,
    Defence,
    Resource,
    Mining,
    Herbalism,
    Woodcutting,
    Fishing,
    Rest,
    Crafting,
    Research,
}

/// A player card definition that references effect templates by name.
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PlayerCardConfig {
    pub card_kind: PlayerCardKind,
    pub effect_refs: Vec<String>,
    pub counts: CardCounts,
    #[serde(default)]
    pub valid_disciplines: Vec<Discipline>,
}

/// An encounter card definition referencing its discipline-specific encounter def.
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct EncounterCardConfig {
    pub encounter_def: EncounterDefConfig,
    pub counts: CardCounts,
    #[serde(default)]
    pub valid_disciplines: Vec<Discipline>,
}

/// Encounter definitions — one variant per discipline.
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde", tag = "encounter_type")]
pub enum EncounterDefConfig {
    Combat {
        combatant: CombatantConfig,
    },
    Mining {
        mining_def: MiningDefConfig,
    },
    Herbalism {
        herbalism_def: HerbalismDefConfig,
    },
    Woodcutting {
        woodcutting_def: WoodcuttingDefConfig,
    },
    Fishing {
        fishing_def: FishingDefConfig,
    },
    Rest {
        rest_def: RestDef,
    },
    Crafting {
        crafting_def: CraftingDefConfig,
    },
    Research {
        research_def: ResearchDefConfig,
    },
    Milestone {
        milestone_def: MilestoneDefConfig,
    },
}

/// Combat encounter config — references effect names for enemy decks.
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CombatantConfig {
    pub initial_tokens: HashMap<TokenType, u64>,
    pub attack_deck: Vec<EnemyDeckEntryConfig>,
    pub defence_deck: Vec<EnemyDeckEntryConfig>,
    pub resource_deck: Vec<EnemyDeckEntryConfig>,
}

/// An entry in an enemy deck, referencing effects by name.
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct EnemyDeckEntryConfig {
    pub effect_refs: Vec<String>,
    pub counts: DeckCounts,
}

/// Mining encounter config.
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct MiningDefConfig {
    pub initial_light_level: i64,
    pub ore_deck: Vec<OreDeckEntryConfig>,
}

/// An ore card entry referencing effects by name.
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct OreDeckEntryConfig {
    pub effect_refs: Vec<String>,
    pub counts: DeckCounts,
}

/// Herbalism encounter config.
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct HerbalismDefConfig {
    pub plant_hand: Vec<PlantCardConfig>,
    pub rewards: HashMap<TokenType, i64>,
}

/// A plant card entry with characteristics and effect name references.
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PlantCardConfig {
    pub characteristics: Vec<super::types::PlantCharacteristic>,
    #[serde(default)]
    pub effect_refs: Vec<String>,
    pub counts: DeckCounts,
}

/// Woodcutting encounter config.
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct WoodcuttingDefConfig {
    pub max_plays: u32,
    pub base_rewards: HashMap<TokenType, i64>,
}

/// Fishing encounter config.
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct FishingDefConfig {
    pub valid_range_min: i64,
    pub valid_range_max: i64,
    pub max_turns: u32,
    pub win_turns_needed: u32,
    pub fish_deck: Vec<FishDeckEntryConfig>,
    pub rewards: HashMap<TokenType, i64>,
}

/// A fish card entry with value and effect name references.
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct FishDeckEntryConfig {
    pub value: i64,
    #[serde(default)]
    pub effect_refs: Vec<String>,
    pub counts: DeckCounts,
}

/// Crafting encounter config.
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CraftingDefConfig {
    pub initial_crafting_tokens: i64,
    pub enemy_crafting_deck: Vec<EnemyCraftingEntryConfig>,
}

/// An enemy crafting card entry referencing effects by name.
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct EnemyCraftingEntryConfig {
    pub effect_refs: Vec<String>,
    pub counts: DeckCounts,
}

/// Research encounter config — extends the base research params with interference deck.
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ResearchDefConfig {
    pub target_size: u32,
    pub position_match_yield: i64,
    pub type_match_yield: i64,
    pub base_insight_cost: i64,
    #[serde(default)]
    pub interference_deck: Vec<InterferenceDeckEntryConfig>,
}

/// An interference card entry referencing effects by name.
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct InterferenceDeckEntryConfig {
    pub effect_refs: Vec<String>,
    pub counts: DeckCounts,
}

/// Milestone encounter config — wraps an inner discipline encounter.
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct MilestoneDefConfig {
    pub inner_encounter: Box<EncounterDefConfig>,
    pub discipline: Discipline,
    pub tier: u32,
    pub insight_cost: i64,
}

// ---------------------------------------------------------------------------
// Game rules configuration (externalized constants)
// ---------------------------------------------------------------------------

/// Top-level game rules loaded from `configurations/general/game_rules.json`.
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct GameRulesConfig {
    pub general: GeneralRules,
    pub combat: CombatRules,
    pub research: ResearchRules,
    pub crafting: CraftingRules,
    pub milestone: MilestoneRules,
    pub scouting: ScoutingRules,
    pub woodcutting_patterns: Vec<WoodcuttingPatternRule>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct GeneralRules {
    pub death_reset_health: i64,
    pub death_reset_stamina: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CombatRules {
    pub milestone_insight_on_win: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ResearchRules {
    pub max_hand_size: usize,
    pub base_insight_cost: i64,
    pub insight_cost_multiplier: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CraftingRules {
    pub initial_draw_count: u32,
    pub durability_material_cost: i64,
    pub durability_grant: i64,
    pub min_craft_token_cost: i64,
    pub base_cost_divisor: i64,
    pub max_material_percent: i64,
    pub material_token_min: i64,
    pub material_token_max: i64,
    pub cost_reduction_floor_percent: i64,
    pub cost_formula_divisor: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct MilestoneRules {
    pub base_insight_cost: i64,
    pub insight_cost_multiplier: i64,
    pub scale_factor_base: f64,
    pub default_combat_enemy_hp: i64,
    pub default_woodcutting_max_plays_base: u32,
    pub default_woodcutting_max_plays_min: u32,
    pub default_woodcutting_lumber_reward: i64,
    pub default_woodcutting_insight_reward: i64,
    pub default_fishing_valid_range_min: i64,
    pub default_fishing_valid_range_max: i64,
    pub default_fishing_max_turns: u32,
    pub default_fishing_win_turns_needed: u32,
    pub effect_scaling_factor: f64,
    pub draw_cards_attack_increment: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ScoutingRules {
    pub choice_count: usize,
    pub difficulty_delta_min: f64,
    pub difficulty_delta_max: f64,
    pub difficulty_delta_min_separation: f64,
    pub mutation_fraction: f64,
    pub mutation_scale_probability: f64,
    pub mutation_redistribute_probability: f64,
    #[serde(default = "default_death_reduction_min")]
    pub death_difficulty_reduction_min: f64,
    #[serde(default = "default_death_reduction_max")]
    pub death_difficulty_reduction_max: f64,
}

fn default_death_reduction_min() -> f64 {
    -0.25
}

fn default_death_reduction_max() -> f64 {
    -0.05
}

#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct WoodcuttingPatternRule {
    pub name: String,
    pub min_type_count: usize,
    pub min_straight: usize,
    pub min_distinct_types: usize,
    pub second_type_min: usize,
    pub value_freq_min: usize,
    pub multiplier: f64,
}
