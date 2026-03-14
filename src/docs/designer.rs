use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket_okapi::{openapi, JsonSchema};

/// Reference entry for a game concept (card kind, token lifecycle, etc.).
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct ReferenceEntry {
    pub name: String,
    pub description: String,
}

/// A section in the designer guide.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct DesignerSection {
    pub title: String,
    pub description: String,
    pub entries: Vec<ReferenceEntry>,
}

/// Complete designer guide for understanding and authoring game content.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct DesignerGuide {
    pub title: String,
    pub introduction: String,
    pub sections: Vec<DesignerSection>,
}

fn build_designer_guide() -> DesignerGuide {
    DesignerGuide {
        title: "My Little Card Game — Designer Guide".to_string(),
        introduction: "This guide describes how encounters, cards, tokens, and effects \
            are structured. Use it to understand the building blocks of the game and how \
            to author new content. The game follows an 'everything is a deck' philosophy: \
            all game entities are modelled as cards in decks with token-based state tracking."
            .to_string(),
        sections: vec![
            build_encounter_templates(),
            build_card_kinds(),
            build_token_lifecycles(),
            build_effect_system(),
            build_balance_levers(),
            build_configuration_notes(),
        ],
    }
}

fn build_encounter_templates() -> DesignerSection {
    DesignerSection {
        title: "Encounter Templates".to_string(),
        description: "Each encounter is defined by an EncounterKind variant containing \
            a discipline-specific definition (CombatantDef, MiningDef, etc.). Encounters \
            are stored as cards with CardKind::Encounter in the library."
            .to_string(),
        entries: vec![
            ReferenceEntry {
                name: "Combat (CombatantDef)".to_string(),
                description: "Defines enemy Health, enemy card decks (attack/defence/resource), \
                    and cards per phase. The 3-phase turn cycle (Defending → Attacking → Resourcing) \
                    is fixed — difficulty scales through enemy stats and card effects."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Mining (MiningDef)".to_string(),
                description: "Defines initial_light_level and ore_deck (OreCard entries with \
                    damage/yield values). Difficulty scales through light drain rate and ore quality."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Herbalism (HerbalismDef)".to_string(),
                description: "Defines plants with characteristics for the matching puzzle. \
                    Difficulty scales through number of plants, characteristic complexity, \
                    and match mode requirements (Or/And/MostCommon)."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Woodcutting (WoodcuttingDef)".to_string(),
                description: "Defines target chop values and pattern multipliers. Difficulty \
                    scales through higher target values and rarer pattern requirements."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Fishing (FishingDef)".to_string(),
                description: "Defines max_turns, win_turns_needed, and fish value ranges. \
                    Difficulty scales through narrower valid ranges and more wins required."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Rest (RestDef)".to_string(),
                description: "Defines recovery parameters. Rest encounters always succeed — \
                    difficulty comes from material costs of rest cards."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Crafting (CraftingDef)".to_string(),
                description: "Defines craftable targets and enemy cost-inflation deck. \
                    Difficulty scales through base crafting costs and inflation rate."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Research (ResearchDef)".to_string(),
                description: "Hidden-multiplier deduction encounter. Fields: target_size \
                    (hidden slots, default 3), position_match_yield (Y=100), \
                    type_match_yield (X=10), base_insight_cost (Z=5, round N costs N×Z). \
                    Player plays Research cards with ResearchSymbol types (Alpha-Zeta) \
                    against hidden symbol slots. 1:1 optimal matching maximizes yield. \
                    Premium cards have multiple symbols (cost Stamina/Health). \
                    Tier costs scale exponentially (10 → 20 → 40 per tier)."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Milestone (MilestoneDef)".to_string(),
                description: "Wraps an inner discipline encounter (Combat, Mining, etc.) with \
                    escalating difficulty. Fields: inner_encounter_kind, discipline, tier, \
                    insight_cost. Cost = 100 * 2^(tier-1). On win, generates 50%-improved \
                    CardEffects and auto-assigns a single next-tier encounter. Lives in dedicated milestone hand."
                    .to_string(),
            },
        ],
    }
}

fn build_card_kinds() -> DesignerSection {
    DesignerSection {
        title: "Card Kinds".to_string(),
        description: "Cards are typed by CardKind, which determines when they can be played \
            and what effects they produce. Each kind references effects through the \
            ConcreteEffect system."
            .to_string(),
        entries: vec![
            ReferenceEntry {
                name: "Attack".to_string(),
                description: "Played during Combat Attacking phase. Effects deal damage to enemy Health.".to_string(),
            },
            ReferenceEntry {
                name: "Defence".to_string(),
                description: "Played during Combat Defending phase. Effects grant Shield, Dodge, or other mitigation.".to_string(),
            },
            ReferenceEntry {
                name: "Resource".to_string(),
                description: "Played during Combat Resourcing phase. Effects draw cards, generate Stamina/Mana, or provide utility.".to_string(),
            },
            ReferenceEntry {
                name: "Mining".to_string(),
                description: "Played during Mining encounters. Effects provide mining power (ore extraction) or light restoration.".to_string(),
            },
            ReferenceEntry {
                name: "Herbalism".to_string(),
                description: "Played during Herbalism encounters. Effects target and remove plants based on characteristic matching.".to_string(),
            },
            ReferenceEntry {
                name: "Woodcutting".to_string(),
                description: "Played during Woodcutting encounters. Effects provide chop values of various types (Light, Medium, Heavy, Precision, Split).".to_string(),
            },
            ReferenceEntry {
                name: "Fishing".to_string(),
                description: "Played during Fishing encounters. Effects provide fish values or modify the valid catch range.".to_string(),
            },
            ReferenceEntry {
                name: "Rest".to_string(),
                description: "Played during Rest encounters. Effects restore Health/Stamina at the cost of materials.".to_string(),
            },
            ReferenceEntry {
                name: "Crafting".to_string(),
                description: "Played during Crafting encounters. Effects reduce crafting costs or provide special crafting abilities.".to_string(),
            },
            ReferenceEntry {
                name: "Encounter".to_string(),
                description: "Template cards defining encounter types. Stored in the library; drawn to hand for encounter selection.".to_string(),
            },
            ReferenceEntry {
                name: "PlayerCardEffect / EnemyCardEffect".to_string(),
                description: "Effect template cards defining reusable effects. Referenced by other cards via effect_id for composition.".to_string(),
            },
        ],
    }
}

fn build_token_lifecycles() -> DesignerSection {
    DesignerSection {
        title: "Token Lifecycles".to_string(),
        description: "Tokens have different lifecycle semantics determining when they persist, \
            reset, or expire. Understanding lifecycles is critical for balance design."
            .to_string(),
        entries: vec![
            ReferenceEntry {
                name: "PersistentCounter (default)".to_string(),
                description: "Persists across encounters and sessions. Examples: Health, Stamina, \
                    Ore, Plant, Lumber, Fish, Renown, PlayerDeaths."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Combat tokens (reset at Scouting)".to_string(),
                description: "Active during combat, cleared when entering Scouting phase. \
                    Examples: Shield, Dodge, Mana."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Encounter-scoped tokens".to_string(),
                description:
                    "Exist only during a specific encounter, dropped when encounter ends. \
                    Examples: MiningLightLevel, MiningYield, FishingRangeMin, FishingRangeMax, \
                    FishAmount, RestToken, CraftingToken."
                        .to_string(),
            },
            ReferenceEntry {
                name: "Durability tokens".to_string(),
                description: "Persistent but consumed by gathering encounters. When depleted, \
                    that discipline's encounters may degrade. Examples: MiningDurability, \
                    HerbalismDurability, WoodcuttingDurability, FishingDurability."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Insight tokens".to_string(),
                description: "Persistent meta-tokens earned from encounters, spent on Research. \
                    Per-discipline variants: CombatInsight, MiningInsight, etc. Used as round \
                    costs in Research experiments (round N costs N × base_insight_cost)."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Max hand size tokens".to_string(),
                description: "Control maximum cards in hand per deck type. Both player \
                    (AttackMaxHand, DefenceMaxHand, etc.) and enemy variants exist."
                    .to_string(),
            },
        ],
    }
}

fn build_effect_system() -> DesignerSection {
    DesignerSection {
        title: "Effect System (ConcreteEffect)".to_string(),
        description: "Card effects are defined as ConcreteEffect structs. Each effect has a \
            target token, a value (positive or negative), optional costs, and may reference \
            other effects via effect_id for composition. Effects are resolved through the \
            library's resolve_effect() method."
            .to_string(),
        entries: vec![
            ReferenceEntry {
                name: "Direct effects".to_string(),
                description: "Modify a token by a fixed value. Example: +100 Health, -50 enemy Health, +3 Shield.".to_string(),
            },
            ReferenceEntry {
                name: "Cost effects (rolled_costs)".to_string(),
                description: "Effects can have costs that must be affordable to play. Costs are \
                    defined as a percentage of the rolled value applied to a specific token. \
                    If costs can't be paid, the card is unpayable."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Draw effects".to_string(),
                description: "Resource cards can trigger card draws via DrawCards effect. Specifies \
                    attack/defence/resource draw counts per card play."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Library-referenced effects (effect_id)".to_string(),
                description: "Cards reference effect templates stored in the library by index. \
                    This enables effect reuse and composition. Shared templates are at indices 0-4 \
                    (damage, shield, stamina, draw, insight). Effect templates >= 1_000_000 are \
                    standalone templates added via add_effect_template()."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Insight effects".to_string(),
                description: "Gathering cards can include Insight effects in their effects vec, \
                    awarding discipline-specific Insight tokens when played."
                    .to_string(),
            },
        ],
    }
}

fn build_balance_levers() -> DesignerSection {
    DesignerSection {
        title: "Balance Levers".to_string(),
        description: "Key parameters that affect game balance. Adjusting these changes the \
            difficulty and feel of encounters. Check /metrics during playtesting to measure impact."
            .to_string(),
        entries: vec![
            ReferenceEntry {
                name: "Starting token values".to_string(),
                description: "Health (1000), Stamina (1000), Durabilities (10000), Foresight (3), \
                    Max hand sizes (5 each). Defined in GameState::new_with_rng()."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Card effect values".to_string(),
                description: "Damage, shield, healing, draw counts per card. Higher values = easier encounters. \
                    Defined per card in the library initialization."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Card costs".to_string(),
                description: "Stamina, Mana, material costs per card. Higher costs = harder to sustain. \
                    Crafting costs use RNG distribution (2-4 materials, 75% cap per token)."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Enemy parameters".to_string(),
                description: "Enemy Health, attack/defence values, cards per phase. Scaling these \
                    directly affects encounter difficulty."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Encounter-specific parameters".to_string(),
                description: "Mining: initial light level, ore deck. Fishing: turns, win threshold. \
                    Herbalism: plant count/complexity. Woodcutting: target values. \
                    Research: target_size (hidden slots), position_match_yield (Y), \
                    type_match_yield (X), base_insight_cost (Z)."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Reward rates".to_string(),
                description: "Materials per encounter, MilestoneInsight per Combat win (100), \
                    pattern multipliers in Woodcutting. Affects progression speed."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Death penalty".to_string(),
                description: "Currently resets all gathering materials (Ore, Plant, Lumber, Fish) \
                    and restores Health/Stamina to 1000. Adjusting severity changes risk tolerance."
                    .to_string(),
            },
        ],
    }
}

fn build_configuration_notes() -> DesignerSection {
    DesignerSection {
        title: "Configuration & Tooling".to_string(),
        description: "Notes on how game content is authored and how to use the available \
            tools for testing and balance analysis."
            .to_string(),
        entries: vec![
            ReferenceEntry {
                name: "Library initialization".to_string(),
                description: "Cards and encounters are currently defined in Rust code \
                    (src/library/game_state.rs). Future: JSON configuration files in a \
                    configurations/ folder will allow editing without recompiling (see roadmap step 14)."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Deterministic seeding".to_string(),
                description: "All RNG is seeded — same seed + same actions = same outcome. \
                    Use this for reproducible testing: play a session, record the seed and \
                    action log, then replay to verify changes."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Action log replay".to_string(),
                description: "GET /actions/log returns the full action history. Combined with \
                    the seed, this enables exact replay of any session for debugging or testing."
                    .to_string(),
            },
            ReferenceEntry {
                name: "Metrics endpoint".to_string(),
                description: "GET /metrics returns per-discipline win rates, average rounds, \
                    token snapshots, and resource flow data. Use during playtesting to measure \
                    balance impact of changes."
                    .to_string(),
            },
            ReferenceEntry {
                name: "OpenAPI / Swagger".to_string(),
                description: "Interactive API documentation at /swagger/. All endpoints are \
                    documented with JSON schemas for request/response types."
                    .to_string(),
            },
        ],
    }
}

/// Designer guide for understanding and authoring game content.
///
/// Returns a structured reference covering encounter templates (how each discipline's
/// encounters are defined), card kinds (when each type is played and what it does),
/// token lifecycles (which tokens persist vs expire), the effect system (how card
/// effects compose), balance levers (what parameters to tune), and configuration
/// tooling (how to test changes). Use this to understand the building blocks when
/// creating new encounters, cards, or tuning game balance.
#[openapi]
#[get("/docs/designer")]
pub fn get_designer_guide() -> Json<DesignerGuide> {
    Json(build_designer_guide())
}
