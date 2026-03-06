use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::JsonSchema;
use std::collections::HashMap;

fn default_persistent_lifecycle() -> TokenLifecycle {
    TokenLifecycle::PersistentCounter
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub enum Discipline {
    Combat,
    Mining,
    Herbalism,
    Woodcutting,
    Fishing,
    Rest,
    Crafting,
}

/// Canonical token identifier enum.
/// Each variant is a well-known token with associated lifecycle semantics.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub enum TokenType {
    // Combat tokens (expire when encounter reaches Scouting phase)
    Health,
    MaxHealth,
    Shield,
    Stamina,
    Dodge,
    Mana,
    // Persistent/meta tokens
    Insight,
    Renown,
    Refinement,
    Stability,
    Foresight,
    Momentum,
    Corruption,
    Exhaustion,
    MiningDurability,
    HerbalismDurability,
    WoodcuttingDurability,
    FishingDurability,
    // Material tokens (produced by gathering)
    Ore,
    Plant,
    Lumber,
    Fish,
    // Max handsize tokens (player decks)
    AttackMaxHand,
    DefenceMaxHand,
    ResourceMaxHand,
    MiningMaxHand,
    HerbalismMaxHand,
    WoodcuttingMaxHand,
    FishingMaxHand,
    // Max handsize tokens (enemy decks, encounter-scoped)
    EnemyAttackMaxHand,
    EnemyDefenceMaxHand,
    EnemyResourceMaxHand,
    // Milestone/reward tokens
    MilestoneInsight,
    // Fishing encounter-scoped tokens
    FishingRangeMin,
    FishingRangeMax,
    FishAmount,
    // Mining encounter-scoped tokens
    MiningLightLevel,
    MiningYield,
    MiningPower,
    // Rest encounter tokens
    RestToken,
    RestMaxHand,
    // Crafting encounter tokens
    CraftingToken,
    CraftingMaxHand,
    // Death tracking
    PlayerDeaths,
}

/// All known token types.
impl TokenType {
    pub fn all() -> Vec<TokenType> {
        vec![
            TokenType::Health,
            TokenType::MaxHealth,
            TokenType::Shield,
            TokenType::Stamina,
            TokenType::Dodge,
            TokenType::Mana,
            TokenType::Insight,
            TokenType::Renown,
            TokenType::Refinement,
            TokenType::Stability,
            TokenType::Foresight,
            TokenType::Momentum,
            TokenType::Corruption,
            TokenType::Exhaustion,
            TokenType::MiningDurability,
            TokenType::HerbalismDurability,
            TokenType::WoodcuttingDurability,
            TokenType::FishingDurability,
            TokenType::Ore,
            TokenType::Plant,
            TokenType::Lumber,
            TokenType::Fish,
            TokenType::AttackMaxHand,
            TokenType::DefenceMaxHand,
            TokenType::ResourceMaxHand,
            TokenType::MiningMaxHand,
            TokenType::HerbalismMaxHand,
            TokenType::WoodcuttingMaxHand,
            TokenType::FishingMaxHand,
            TokenType::EnemyAttackMaxHand,
            TokenType::EnemyDefenceMaxHand,
            TokenType::EnemyResourceMaxHand,
            TokenType::MilestoneInsight,
            TokenType::FishingRangeMin,
            TokenType::FishingRangeMax,
            TokenType::FishAmount,
            TokenType::MiningLightLevel,
            TokenType::MiningYield,
            TokenType::RestToken,
            TokenType::RestMaxHand,
            TokenType::CraftingToken,
            TokenType::CraftingMaxHand,
            TokenType::PlayerDeaths,
        ]
    }

    pub fn is_gathering_material(&self) -> bool {
        matches!(
            self,
            TokenType::Ore | TokenType::Plant | TokenType::Lumber | TokenType::Fish
        )
    }

    pub fn is_durability_cost(&self) -> bool {
        matches!(
            self,
            TokenType::MiningDurability
                | TokenType::HerbalismDurability
                | TokenType::WoodcuttingDurability
                | TokenType::FishingDurability
        )
    }
}

/// Split gathering costs into pre-play costs (reject card if unaffordable)
/// and post-play costs (durability — deplete encounter after play).
pub fn split_token_amounts(costs: &[TokenAmount]) -> (Vec<TokenAmount>, Vec<TokenAmount>) {
    let pre_play = costs
        .iter()
        .filter(|c| !c.token_type.is_durability_cost())
        .cloned()
        .collect();
    let post_play = costs
        .iter()
        .filter(|c| c.token_type.is_durability_cost())
        .cloned()
        .collect();
    (pre_play, post_play)
}

impl Token {
    /// Create a persistent counter token (most tokens use this).
    pub fn persistent(token_type: TokenType) -> Self {
        Token {
            token_type,
            lifecycle: TokenLifecycle::PersistentCounter,
        }
    }

    /// Create a dodge token with its unique lifecycle.
    pub fn dodge() -> Self {
        Token {
            token_type: TokenType::Dodge,
            lifecycle: TokenLifecycle::FixedTypeDuration {
                duration: 1,
                phases: vec![CombatPhase::Defending],
            },
        }
    }
}

/// Describes what kind of effect a card applies.
/// CardEffects are templates with min/max ranges; concrete cards roll fixed values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde", tag = "effect_type")]
pub enum CardEffectKind {
    /// Grant tokens: starts from a cap range and calculates gain as cap * gain_percent / 100,
    /// clamped so the token balance does not exceed the rolled cap.
    /// Costs are a percentage of the gain. The gain token_type must not match any cost token_type.
    GainTokens {
        target: EffectTarget,
        token_type: TokenType,
        cap_min: i64,
        cap_max: i64,
        gain_min_percent: u32,
        gain_max_percent: u32,
        #[serde(default)]
        costs: Vec<CardEffectCost>,
        #[serde(default = "default_persistent_lifecycle")]
        duration: TokenLifecycle,
    },
    /// Lose tokens: min/max represent the positive amount to lose. The apply logic subtracts.
    /// Costs are calculated after the loss value is determined.
    LoseTokens {
        target: EffectTarget,
        token_type: TokenType,
        min: i64,
        max: i64,
        #[serde(default)]
        costs: Vec<CardEffectCost>,
        #[serde(default = "default_persistent_lifecycle")]
        duration: TokenLifecycle,
    },
    DrawCards {
        attack: u32,
        defence: u32,
        resource: u32,
    },
    /// Grant Insight tokens: rolls a value from min..max range.
    Insight { min: i64, max: i64 },
}

/// Cost definition on a CardEffect template: a percentage range of the effect value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct CardEffectCost {
    pub token_type: TokenType,
    pub min_percent: u32,
    pub max_percent: u32,
}

/// A concrete effect on a card: references a CardEffect and stores rolled values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct ConcreteEffect {
    pub effect_id: usize,
    pub rolled_value: i64,
    #[serde(default)]
    pub rolled_costs: Vec<ConcreteEffectCost>,
    /// Rolled cap for token-granting effects (from cap_min..cap_max on the template).
    #[serde(default)]
    pub rolled_cap: Option<i64>,
    /// Rolled gain percentage (from gain_min_percent..gain_max_percent on the template).
    #[serde(default)]
    pub rolled_gain_percent: Option<u32>,
}

/// A concrete rolled cost on a card: the specific percentage rolled from the cost range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct ConcreteEffectCost {
    pub token_type: TokenType,
    pub rolled_percent: u32,
}

/// A fixed amount of a token type, used in costs and gains of gathering discipline cards.
/// When used as a gain, `cap` limits the maximum accumulated value of this token type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct TokenAmount {
    pub token_type: TokenType,
    pub amount: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cap: Option<i64>,
}

/// Who a card effect targets.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
pub enum EffectTarget {
    OnSelf,
    OnOpponent,
}

// ====== Library types (card location model from vision.md) ======

/// Where card copies reside.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub enum CardLocation {
    Library,
    Deck,
    Hand,
    Discard,
}

/// Exclusive copy counts describing where player copies reside.
/// [library, deck, hand, discard] — each copy exists in exactly one location.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct CardCounts {
    pub library: u32,
    pub deck: u32,
    pub hand: u32,
    pub discard: u32,
}

impl CardCounts {
    pub fn total(&self) -> u32 {
        self.library + self.deck + self.hand + self.discard
    }
}

/// The kind of card and its type-specific payload.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde", tag = "card_kind")]
pub enum CardKind {
    Attack {
        effects: Vec<ConcreteEffect>,
    },
    Defence {
        effects: Vec<ConcreteEffect>,
    },
    Resource {
        effects: Vec<ConcreteEffect>,
    },
    Mining {
        mining_effect: MiningCardEffect,
    },
    Herbalism {
        herbalism_effect: HerbalismCardEffect,
    },
    Woodcutting {
        woodcutting_effect: WoodcuttingCardEffect,
    },
    Fishing {
        fishing_effect: FishingCardEffect,
    },
    Rest {
        effects: Vec<ConcreteEffect>,
        rest_token_cost: i64,
    },
    Crafting {
        crafting_effect: CraftingCardEffect,
    },
    Encounter {
        encounter_kind: EncounterKind,
    },
    PlayerCardEffect {
        kind: CardEffectKind,
    },
    EnemyCardEffect {
        kind: CardEffectKind,
    },
}

/// Inline effect for Mining discipline cards.
/// All effects expressed through token-based costs/gains vectors.
/// Resolution logic interprets gain token types: MiningPower triggers yield formula,
/// MiningLightLevel triggers light level gain with per-gain cap.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct MiningCardEffect {
    #[serde(default)]
    pub costs: Vec<TokenAmount>,
    #[serde(default)]
    pub gains: Vec<TokenAmount>,
}

/// Inline effect for Crafting discipline cards.
/// Gains reduce material costs of an active craft (gathering token types: Ore, Plant, Lumber, Fish).
/// Costs may include Stamina or Health.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct CraftingCardEffect {
    #[serde(default)]
    pub costs: Vec<TokenAmount>,
    #[serde(default)]
    pub reductions: Vec<TokenAmount>,
}

/// Plant characteristics used by Herbalism encounters.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub enum PlantCharacteristic {
    Fragile,
    Thorny,
    Aromatic,
    Bitter,
    Luminous,
}

/// How an herbalism card matches plant characteristics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub enum HerbalismMatchMode {
    /// Remove plants sharing ANY listed characteristic (existing behavior).
    Or { types: Vec<PlantCharacteristic> },
    /// Remove plants matching ALL listed characteristics.
    And { types: Vec<PlantCharacteristic> },
    /// Remove plants matching the characteristic(s) present on the MOST cards.
    MostCommon {
        limit: u32,
        types: Vec<PlantCharacteristic>,
    },
    /// Remove plants matching the characteristic(s) present on the LEAST cards.
    LeastCommon {
        limit: u32,
        types: Vec<PlantCharacteristic>,
    },
}

/// Inline effect for Herbalism discipline cards.
/// Targets characteristics to remove matching plant cards; broader cards are riskier.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct HerbalismCardEffect {
    #[serde(default)]
    pub costs: Vec<TokenAmount>,
    pub match_mode: HerbalismMatchMode,
    #[serde(default)]
    pub gains: Vec<TokenAmount>,
}

/// A card in the plant hand. Each card has characteristics that Herbalism cards can target.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct PlantCard {
    pub characteristics: Vec<PlantCharacteristic>,
    #[serde(default)]
    pub effects: Vec<ConcreteEffect>,
    pub counts: DeckCounts,
}

/// Definition of a plant node for an herbalism gathering encounter.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct HerbalismDef {
    pub plant_hand: Vec<PlantCard>,
    #[serde(with = "token_map_serde")]
    #[schemars(with = "token_map_serde::SchemaHelper")]
    pub rewards: HashMap<Token, i64>,
}

/// Chop types for Woodcutting discipline cards.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub enum ChopType {
    LightChop,
    HeavyChop,
    MediumChop,
    PrecisionChop,
    SplitChop,
}

/// Inline effect for Woodcutting discipline cards.
/// Cards have chop types, chop values (for pattern building), and a durability cost.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct WoodcuttingCardEffect {
    pub chop_types: Vec<ChopType>,
    pub chop_values: Vec<u32>,
    #[serde(default)]
    pub costs: Vec<TokenAmount>,
    #[serde(default)]
    pub gains: Vec<TokenAmount>,
}

/// Snapshot of a played woodcutting card for pattern evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct PlayedWoodcuttingCard {
    pub card_id: usize,
    pub chop_types: Vec<ChopType>,
    pub chop_values: Vec<u32>,
}

/// Definition of a woodcutting encounter (no enemy deck).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct WoodcuttingDef {
    pub max_plays: u32,
    #[serde(with = "token_map_serde")]
    #[schemars(with = "token_map_serde::SchemaHelper")]
    pub base_rewards: HashMap<Token, i64>,
}

/// Inline effect for Fishing discipline cards.
/// Cards can have multiple values; the best value for winning is chosen.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct FishingCardEffect {
    pub values: Vec<i64>,
    #[serde(default)]
    pub costs: Vec<TokenAmount>,
    #[serde(default)]
    pub gains: Vec<TokenAmount>,
}

/// A card in the fish (enemy) deck. Each card has a numeric value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct FishCard {
    pub value: i64,
    #[serde(default)]
    pub effects: Vec<ConcreteEffect>,
    pub counts: DeckCounts,
}

/// Definition of a fishing encounter.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct FishingDef {
    pub valid_range_min: i64,
    pub valid_range_max: i64,
    pub max_turns: u32,
    pub win_turns_needed: u32,
    pub fish_deck: Vec<FishCard>,
    #[serde(with = "token_map_serde")]
    #[schemars(with = "token_map_serde::SchemaHelper")]
    pub rewards: HashMap<Token, i64>,
}

/// Sub-type of encounter cards.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde", tag = "encounter_type")]
pub enum EncounterKind {
    Combat { combatant_def: CombatantDef },
    Mining { mining_def: MiningDef },
    Herbalism { herbalism_def: HerbalismDef },
    Woodcutting { woodcutting_def: WoodcuttingDef },
    Fishing { fishing_def: FishingDef },
    Rest { rest_def: RestDef },
    Crafting { crafting_def: CraftingDef },
}

/// Definition of a mining node for a gathering encounter.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct MiningDef {
    pub initial_light_level: i64,
    pub ore_deck: Vec<OreCard>,
}

/// Definition of a rest encounter.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct RestDef {
    pub rest_token_min: i64,
    pub rest_token_max: i64,
}

/// An enemy crafting card that increases material costs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct EnemyCraftingCard {
    pub increases: Vec<TokenAmount>,
    #[serde(default)]
    pub effects: Vec<ConcreteEffect>,
    pub counts: DeckCounts,
}

impl HasDeckCounts for EnemyCraftingCard {
    fn deck_count(&self) -> u32 {
        self.counts.deck
    }
    fn hand_count(&self) -> u32 {
        self.counts.hand
    }
    fn discard_count(&self) -> u32 {
        self.counts.discard
    }
    fn deck_count_mut(&mut self) -> &mut u32 {
        &mut self.counts.deck
    }
    fn hand_count_mut(&mut self) -> &mut u32 {
        &mut self.counts.hand
    }
    fn discard_count_mut(&mut self) -> &mut u32 {
        &mut self.counts.discard
    }
}

/// Definition of a crafting encounter.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct CraftingDef {
    pub initial_crafting_tokens: i64,
    pub enemy_crafting_deck: Vec<EnemyCraftingCard>,
}

/// State of an active craft-a-card mini-game within a crafting encounter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct CraftingCraftState {
    pub target_card_id: usize,
    #[serde(with = "token_type_map_serde")]
    #[schemars(with = "token_type_map_serde::SchemaHelper")]
    pub original_costs: HashMap<TokenType, i64>,
    #[serde(with = "token_type_map_serde")]
    #[schemars(with = "token_type_map_serde::SchemaHelper")]
    pub current_costs: HashMap<TokenType, i64>,
}

/// A card in the ore deck. Each card applies token-based damages to the player.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct OreCard {
    pub damages: Vec<TokenAmount>,
    #[serde(default)]
    pub effects: Vec<ConcreteEffect>,
    pub counts: DeckCounts,
}

/// Copy counts for non-library cards (enemy, ore, etc.): deck, hand, discard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct DeckCounts {
    pub deck: u32,
    pub hand: u32,
    pub discard: u32,
}

/// Trait for card types that have deck/hand/discard tracking.
/// Both `DeckCounts` and `CardCounts` implement this, enabling generic
/// deck operations across encounter-internal and player-owned cards.
pub trait HasDeckCounts {
    fn deck_count(&self) -> u32;
    fn hand_count(&self) -> u32;
    fn discard_count(&self) -> u32;
    fn deck_count_mut(&mut self) -> &mut u32;
    fn hand_count_mut(&mut self) -> &mut u32;
    fn discard_count_mut(&mut self) -> &mut u32;
}

impl HasDeckCounts for DeckCounts {
    fn deck_count(&self) -> u32 {
        self.deck
    }
    fn hand_count(&self) -> u32 {
        self.hand
    }
    fn discard_count(&self) -> u32 {
        self.discard
    }
    fn deck_count_mut(&mut self) -> &mut u32 {
        &mut self.deck
    }
    fn hand_count_mut(&mut self) -> &mut u32 {
        &mut self.hand
    }
    fn discard_count_mut(&mut self) -> &mut u32 {
        &mut self.discard
    }
}

impl HasDeckCounts for CardCounts {
    fn deck_count(&self) -> u32 {
        self.deck
    }
    fn hand_count(&self) -> u32 {
        self.hand
    }
    fn discard_count(&self) -> u32 {
        self.discard
    }
    fn deck_count_mut(&mut self) -> &mut u32 {
        &mut self.deck
    }
    fn hand_count_mut(&mut self) -> &mut u32 {
        &mut self.hand
    }
    fn discard_count_mut(&mut self) -> &mut u32 {
        &mut self.discard
    }
}

impl HasDeckCounts for OreCard {
    fn deck_count(&self) -> u32 {
        self.counts.deck
    }
    fn hand_count(&self) -> u32 {
        self.counts.hand
    }
    fn discard_count(&self) -> u32 {
        self.counts.discard
    }
    fn deck_count_mut(&mut self) -> &mut u32 {
        &mut self.counts.deck
    }
    fn hand_count_mut(&mut self) -> &mut u32 {
        &mut self.counts.hand
    }
    fn discard_count_mut(&mut self) -> &mut u32 {
        &mut self.counts.discard
    }
}

impl HasDeckCounts for FishCard {
    fn deck_count(&self) -> u32 {
        self.counts.deck
    }
    fn hand_count(&self) -> u32 {
        self.counts.hand
    }
    fn discard_count(&self) -> u32 {
        self.counts.discard
    }
    fn deck_count_mut(&mut self) -> &mut u32 {
        &mut self.counts.deck
    }
    fn hand_count_mut(&mut self) -> &mut u32 {
        &mut self.counts.hand
    }
    fn discard_count_mut(&mut self) -> &mut u32 {
        &mut self.counts.discard
    }
}

impl HasDeckCounts for PlantCard {
    fn deck_count(&self) -> u32 {
        self.counts.deck
    }
    fn hand_count(&self) -> u32 {
        self.counts.hand
    }
    fn discard_count(&self) -> u32 {
        self.counts.discard
    }
    fn deck_count_mut(&mut self) -> &mut u32 {
        &mut self.counts.deck
    }
    fn hand_count_mut(&mut self) -> &mut u32 {
        &mut self.counts.hand
    }
    fn discard_count_mut(&mut self) -> &mut u32 {
        &mut self.counts.discard
    }
}

impl HasDeckCounts for EnemyCardDef {
    fn deck_count(&self) -> u32 {
        self.counts.deck
    }
    fn hand_count(&self) -> u32 {
        self.counts.hand
    }
    fn discard_count(&self) -> u32 {
        self.counts.discard
    }
    fn deck_count_mut(&mut self) -> &mut u32 {
        &mut self.counts.deck
    }
    fn hand_count_mut(&mut self) -> &mut u32 {
        &mut self.counts.hand
    }
    fn discard_count_mut(&mut self) -> &mut u32 {
        &mut self.counts.discard
    }
}

impl HasDeckCounts for LibraryCard {
    fn deck_count(&self) -> u32 {
        self.counts.deck
    }
    fn hand_count(&self) -> u32 {
        self.counts.hand
    }
    fn discard_count(&self) -> u32 {
        self.counts.discard
    }
    fn deck_count_mut(&mut self) -> &mut u32 {
        &mut self.counts.deck
    }
    fn hand_count_mut(&mut self) -> &mut u32 {
        &mut self.counts.hand
    }
    fn discard_count_mut(&mut self) -> &mut u32 {
        &mut self.counts.discard
    }
}

/// Definition of an enemy combatant for a combat encounter card.
/// Enemies are self-contained: their cards are inline, not Library references.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct CombatantDef {
    #[serde(with = "token_map_serde_u64")]
    #[schemars(with = "token_map_serde_u64::SchemaHelper")]
    pub initial_tokens: HashMap<Token, u64>,
    pub attack_deck: Vec<EnemyCardDef>,
    pub defence_deck: Vec<EnemyCardDef>,
    pub resource_deck: Vec<EnemyCardDef>,
}

/// A simple inline card definition for enemy decks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct EnemyCardDef {
    pub effects: Vec<ConcreteEffect>,
    pub counts: DeckCounts,
}

/// A single entry in the Library. Index in the Vec = card ID.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct LibraryCard {
    pub kind: CardKind,
    pub counts: CardCounts,
    /// Material cost to craft a copy of this card. Computed at creation time.
    /// Only meaningful for player cards (Attack, Defence, Resource, Mining, etc.).
    #[serde(default, with = "token_type_map_serde")]
    #[schemars(with = "token_type_map_serde::SchemaHelper")]
    pub crafting_cost: HashMap<TokenType, i64>,
    #[serde(default)]
    pub discipline_tags: Vec<Discipline>,
}

/// A token instance: token type + lifecycle. Used as key in token balance maps.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct Token {
    pub token_type: TokenType,
    pub lifecycle: TokenLifecycle,
}

/// Serde helper to serialize/deserialize `HashMap<Token, i64>` as a compact JSON map.
/// Keys are formatted as `"TokenType:Lifecycle"` (e.g., `"Health:PersistentCounter"`).
pub mod token_map_serde {
    use super::{HashMap, Token, TokenLifecycle, TokenType};
    use rocket::serde::{self, Deserialize, Serialize};
    use schemars::JsonSchema;
    use std::collections::BTreeMap;

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub struct Entry {
        pub token: Token,
        pub value: i64,
    }

    /// Schema-only type so `#[schemars(with = "SchemaHelper")]` works.
    pub type SchemaHelper = BTreeMap<String, i64>;

    fn token_to_key(token: &Token) -> String {
        let type_str = format!("{:?}", token.token_type);
        match &token.lifecycle {
            TokenLifecycle::PersistentCounter => type_str,
            other => format!("{}:{:?}", type_str, other),
        }
    }

    fn key_to_token(key: &str) -> Result<Token, String> {
        let (type_str, lifecycle_str) = if let Some((t, l)) = key.split_once(':') {
            (t, Some(l))
        } else {
            (key, None)
        };

        let token_type: TokenType =
            serde_json::from_str(&format!("\"{}\"", type_str)).map_err(|e| e.to_string())?;

        let lifecycle = match lifecycle_str {
            None | Some("PersistentCounter") => TokenLifecycle::PersistentCounter,
            Some(s) => serde_json::from_str(s).unwrap_or(TokenLifecycle::PersistentCounter),
        };

        Ok(Token {
            token_type,
            lifecycle,
        })
    }

    pub fn serialize<S>(map: &HashMap<Token, i64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let compact: BTreeMap<String, i64> =
            map.iter().map(|(k, v)| (token_to_key(k), *v)).collect();
        compact.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<Token, i64>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Support both compact map format and legacy array format
        let value: serde_json::Value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::Object(map) => {
                let mut result = HashMap::new();
                for (k, v) in map {
                    let token = key_to_token(&k).map_err(serde::de::Error::custom)?;
                    let val = v.as_i64().ok_or_else(|| {
                        serde::de::Error::custom(format!("expected integer value for {}", k))
                    })?;
                    result.insert(token, val);
                }
                Ok(result)
            }
            serde_json::Value::Array(_) => {
                let entries: Vec<Entry> =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(entries.into_iter().map(|e| (e.token, e.value)).collect())
            }
            _ => Err(serde::de::Error::custom(
                "expected object or array for token map",
            )),
        }
    }
}

pub mod token_map_serde_u64 {
    use super::{HashMap, Token, TokenLifecycle, TokenType};
    use rocket::serde::{self, Deserialize, Serialize};
    use std::collections::BTreeMap;

    pub type SchemaHelper = BTreeMap<String, u64>;

    fn token_to_key(token: &Token) -> String {
        let type_str = format!("{:?}", token.token_type);
        match &token.lifecycle {
            TokenLifecycle::PersistentCounter => type_str,
            other => format!("{}:{:?}", type_str, other),
        }
    }

    fn key_to_token(key: &str) -> Result<Token, String> {
        let (type_str, lifecycle_str) = if let Some((t, l)) = key.split_once(':') {
            (t, Some(l))
        } else {
            (key, None)
        };

        let token_type: TokenType =
            serde_json::from_str(&format!("\"{}\"", type_str)).map_err(|e| e.to_string())?;

        let lifecycle = match lifecycle_str {
            None | Some("PersistentCounter") => TokenLifecycle::PersistentCounter,
            Some(s) => serde_json::from_str(s).unwrap_or(TokenLifecycle::PersistentCounter),
        };

        Ok(Token {
            token_type,
            lifecycle,
        })
    }

    pub fn serialize<S>(map: &HashMap<Token, u64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let compact: BTreeMap<String, u64> =
            map.iter().map(|(k, v)| (token_to_key(k), *v)).collect();
        compact.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<Token, u64>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: serde_json::Value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::Object(map) => {
                let mut result = HashMap::new();
                for (k, v) in map {
                    let token = key_to_token(&k).map_err(serde::de::Error::custom)?;
                    let val = v.as_u64().ok_or_else(|| {
                        serde::de::Error::custom(format!(
                            "expected unsigned integer value for {}",
                            k
                        ))
                    })?;
                    result.insert(token, val);
                }
                Ok(result)
            }
            _ => Err(serde::de::Error::custom("expected object for token map")),
        }
    }
}

/// Serde helper to serialize/deserialize `HashMap<TokenType, i64>` as a compact JSON map.
pub mod token_type_map_serde {
    use super::{HashMap, TokenType};
    use rocket::serde::{self, Deserialize, Serialize};
    use std::collections::BTreeMap;

    pub type SchemaHelper = BTreeMap<String, i64>;

    pub fn serialize<S>(map: &HashMap<TokenType, i64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let compact: BTreeMap<String, i64> =
            map.iter().map(|(k, v)| (format!("{:?}", k), *v)).collect();
        compact.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<TokenType, i64>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map: BTreeMap<String, i64> = BTreeMap::deserialize(deserializer)?;
        let mut result = HashMap::new();
        for (k, v) in map {
            let token_type: TokenType =
                serde_json::from_str(&format!("\"{}\"", k)).map_err(serde::de::Error::custom)?;
            result.insert(token_type, v);
        }
        Ok(result)
    }
}

pub fn token_balance_by_type(map: &HashMap<Token, i64>, tt: &TokenType) -> i64 {
    map.iter()
        .filter(|(k, _)| k.token_type == *tt)
        .map(|(_, v)| *v)
        .sum()
}

/// Get a mutable reference to the first token entry matching a type,
/// or insert a new entry with the type's default lifecycle.
pub fn token_entry_by_type<'a>(map: &'a mut HashMap<Token, i64>, tt: &TokenType) -> &'a mut i64 {
    let key = Token::persistent(tt.clone());
    map.entry(key).or_insert(0)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub enum TokenLifecycle {
    Permanent,
    PersistentCounter,
    FixedDuration {
        duration: u64,
    },
    FixedTypeDuration {
        duration: u64,
        phases: Vec<CombatPhase>,
    },
    UntilNextAction,
    SingleUse,
    Conditional,
}

/// Action payloads for the append-only log — only player-initiated actions.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde", tag = "type")]
pub enum ActionPayload {
    SetSeed { seed: u64 },
    DrawEncounter { encounter_id: String },
    PlayCard { card_id: usize },
    ApplyScouting { card_ids: Vec<usize> },
    AbortEncounter,
    ConcludeEncounter,
    CraftSwap { from_id: usize, to_id: usize },
    CraftCard { target_card_id: usize },
    CraftDurability { discipline: String },
}

/// Stored action entry in the append-only action log.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct ActionEntry {
    pub seq: u64,
    pub payload: ActionPayload,
}

// ====== Combat types for deterministic, logged combat resolution (Step 6) ======

/// A combat action is a card play by a combatant.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct CombatAction {
    pub is_player: bool,
    pub card_id: u64,
}

/// Combat phases for turn-based combat.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(crate = "rocket::serde")]
pub enum CombatPhase {
    Defending,
    Attacking,
    Resourcing,
}

impl CombatPhase {
    pub fn next(&self) -> Self {
        match self {
            CombatPhase::Defending => CombatPhase::Attacking,
            CombatPhase::Attacking => CombatPhase::Resourcing,
            CombatPhase::Resourcing => CombatPhase::Defending,
        }
    }

    pub fn allowed_card_kind(&self) -> fn(&CardKind) -> bool {
        match self {
            CombatPhase::Defending => |k| matches!(k, CardKind::Defence { .. }),
            CombatPhase::Attacking => |k| matches!(k, CardKind::Attack { .. }),
            CombatPhase::Resourcing => |k| matches!(k, CardKind::Resource { .. }),
        }
    }

    pub fn allowed_card_kind_name(&self) -> &'static str {
        match self {
            CombatPhase::Defending => "Defence",
            CombatPhase::Attacking => "Attack",
            CombatPhase::Resourcing => "Resource",
        }
    }
}

/// Snapshot of combat state for deterministic simulation. Pure data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct CombatEncounterState {
    pub round: u64,
    pub phase: CombatPhase,
    #[serde(with = "token_map_serde")]
    #[schemars(with = "token_map_serde::SchemaHelper")]
    pub enemy_tokens: HashMap<Token, i64>,
    pub encounter_card_id: usize,
    pub outcome: EncounterOutcome,
    pub enemy_attack_deck: Vec<EnemyCardDef>,
    pub enemy_defence_deck: Vec<EnemyCardDef>,
    pub enemy_resource_deck: Vec<EnemyCardDef>,
}

/// Runtime state for a mining gathering encounter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct MiningEncounterState {
    pub round: u64,
    pub encounter_card_id: usize,
    pub outcome: EncounterOutcome,
    pub ore_deck: Vec<OreCard>,
    #[serde(with = "token_map_serde")]
    #[schemars(with = "token_map_serde::SchemaHelper")]
    pub encounter_tokens: HashMap<Token, i64>,
}

/// Runtime state for an herbalism gathering encounter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct HerbalismEncounterState {
    pub round: u64,
    pub encounter_card_id: usize,
    pub outcome: EncounterOutcome,
    pub plant_hand: Vec<PlantCard>,
    #[serde(with = "token_map_serde")]
    #[schemars(with = "token_map_serde::SchemaHelper")]
    pub rewards: HashMap<Token, i64>,
}

/// Runtime state for a woodcutting gathering encounter (pattern-matching).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct WoodcuttingEncounterState {
    pub round: u64,
    pub encounter_card_id: usize,
    pub outcome: EncounterOutcome,
    pub played_cards: Vec<PlayedWoodcuttingCard>,
    pub max_plays: u32,
    pub pattern_name: Option<String>,
    pub pattern_multiplier: Option<f64>,
    #[serde(with = "token_map_serde")]
    #[schemars(with = "token_map_serde::SchemaHelper")]
    pub base_rewards: HashMap<Token, i64>,
}

/// Runtime state for a fishing gathering encounter (card-subtraction).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct FishingEncounterState {
    pub round: u64,
    pub encounter_card_id: usize,
    pub outcome: EncounterOutcome,
    pub turns_won: u32,
    pub max_turns: u32,
    pub win_turns_needed: u32,
    pub valid_range_min: i64,
    pub valid_range_max: i64,
    pub fish_deck: Vec<FishCard>,
    #[serde(with = "token_map_serde")]
    #[schemars(with = "token_map_serde::SchemaHelper")]
    pub rewards: HashMap<Token, i64>,
    #[serde(with = "token_map_serde")]
    #[schemars(with = "token_map_serde::SchemaHelper")]
    pub encounter_tokens: HashMap<Token, i64>,
}

/// Runtime state for a rest encounter (play rest cards using rest tokens).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct RestEncounterState {
    pub encounter_card_id: usize,
    pub outcome: EncounterOutcome,
    pub rest_tokens: i64,
}

/// Runtime state for a crafting encounter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct CraftingEncounterState {
    pub round: u64,
    pub encounter_card_id: usize,
    pub outcome: EncounterOutcome,
    pub crafting_tokens: i64,
    pub enemy_crafting_deck: Vec<EnemyCraftingCard>,
    pub active_craft: Option<CraftingCraftState>,
}

/// Active encounter state, dispatched by encounter type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde", tag = "encounter_state_type")]
pub enum EncounterState {
    Combat(CombatEncounterState),
    Mining(MiningEncounterState),
    Herbalism(HerbalismEncounterState),
    Woodcutting(WoodcuttingEncounterState),
    Fishing(FishingEncounterState),
    Rest(RestEncounterState),
    Crafting(CraftingEncounterState),
}

impl EncounterState {
    pub fn encounter_card_id(&self) -> usize {
        match self {
            EncounterState::Combat(c) => c.encounter_card_id,
            EncounterState::Mining(m) => m.encounter_card_id,
            EncounterState::Herbalism(h) => h.encounter_card_id,
            EncounterState::Woodcutting(w) => w.encounter_card_id,
            EncounterState::Fishing(f) => f.encounter_card_id,
            EncounterState::Rest(r) => r.encounter_card_id,
            EncounterState::Crafting(c) => c.encounter_card_id,
        }
    }

    pub fn is_finished(&self) -> bool {
        *self.outcome() != EncounterOutcome::Undecided
    }

    pub fn outcome(&self) -> &EncounterOutcome {
        match self {
            EncounterState::Combat(c) => &c.outcome,
            EncounterState::Mining(m) => &m.outcome,
            EncounterState::Herbalism(h) => &h.outcome,
            EncounterState::Woodcutting(w) => &w.outcome,
            EncounterState::Fishing(f) => &f.outcome,
            EncounterState::Rest(r) => &r.outcome,
            EncounterState::Crafting(c) => &c.outcome,
        }
    }
}

/// Outcome of an encounter (combat, gathering, etc.).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
pub enum EncounterOutcome {
    Undecided,
    PlayerWon,
    PlayerLost,
}

// ====== Encounter types for the encounter loop (Step 7) ======

/// Phases of an encounter (Step 7 state machine)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(crate = "rocket::serde")]
pub enum EncounterPhase {
    /// An encounter is currently active (combat, mining, etc.)
    InEncounter,
    /// Encounter has finished; scouting is available
    Scouting,
    /// No active encounter
    NoEncounter,
}
