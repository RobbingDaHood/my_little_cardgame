use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::JsonSchema;
use std::collections::HashMap;

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
    Durability,
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
            TokenType::Durability,
        ]
    }
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde", tag = "effect_type")]
pub enum CardEffectKind {
    ChangeTokens {
        target: EffectTarget,
        token_type: TokenType,
        amount: i64,
    },
    DrawCards {
        attack: u32,
        defence: u32,
        resource: u32,
    },
}

/// Who a card effect targets.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
pub enum EffectTarget {
    OnSelf,
    OnOpponent,
}

// ====== Library types (card location model from vision.md) ======

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
    Attack { effect_ids: Vec<usize> },
    Defence { effect_ids: Vec<usize> },
    Resource { effect_ids: Vec<usize> },
    Encounter { encounter_kind: EncounterKind },
    PlayerCardEffect { kind: CardEffectKind },
    EnemyCardEffect { kind: CardEffectKind },
}

/// Sub-type of encounter cards.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde", tag = "encounter_type")]
pub enum EncounterKind {
    Combat { combatant_def: CombatantDef },
}

/// Definition of an enemy combatant for a combat encounter card.
/// Enemies are self-contained: their cards are inline, not Library references.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct CombatantDef {
    #[serde(with = "token_map_serde")]
    #[schemars(with = "token_map_serde::SchemaHelper")]
    pub initial_tokens: HashMap<Token, i64>,
    pub attack_deck: Vec<EnemyCardDef>,
    pub defence_deck: Vec<EnemyCardDef>,
    pub resource_deck: Vec<EnemyCardDef>,
}

/// Copy counts for enemy cards: deck, hand, discard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct EnemyCardCounts {
    pub deck: u32,
    pub hand: u32,
    pub discard: u32,
}

/// A simple inline card definition for enemy decks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct EnemyCardDef {
    pub effect_ids: Vec<usize>,
    pub counts: EnemyCardCounts,
}

/// A single entry in the Library. Index in the Vec = card ID.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct LibraryCard {
    pub kind: CardKind,
    pub counts: CardCounts,
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

/// Get the total value of all tokens of a given type in a token map.
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

/// Action payloads for the append-only log
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde", tag = "type")]
pub enum ActionPayload {
    GrantToken {
        token_id: TokenType,
        amount: i64,
        reason: Option<String>,
        resulting_amount: i64,
    },
    ConsumeToken {
        token_id: TokenType,
        amount: i64,
        reason: Option<String>,
        resulting_amount: i64,
    },
    ExpireToken {
        token_id: TokenType,
        amount: i64,
        reason: Option<String>,
    },
    SetSeed {
        seed: u64,
    },
    RngDraw {
        purpose: String,
        value: u64,
    },
    RngSnapshot {
        snapshot: String,
    },
    PlayCard {
        card_id: usize,
        deck_id: Option<String>,
        reason: Option<String>,
    },
    DrawEncounter {
        area_id: String,
        encounter_id: String,
        reason: Option<String>,
    },
    ReplaceEncounter {
        area_id: String,
        old_encounter_id: String,
        new_encounter_id: String,
        affixes_applied: Vec<String>,
        reason: Option<String>,
    },
    ConsumEntryCost {
        area_id: String,
        encounter_id: String,
        cost_amount: i64,
        reason: Option<String>,
    },
    ApplyScouting {
        area_id: String,
        parameters: String,
        reason: Option<String>,
    },
}

/// Stored action entry in the append-only action log.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct ActionEntry {
    pub seq: u64,
    pub action_type: String,
    pub payload: ActionPayload, // structured payload for replay
    pub timestamp: String,      // milliseconds since epoch as string
    pub actor: Option<String>,
    pub request_id: Option<String>,
    pub version: Option<u32>,
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
pub struct CombatState {
    pub round: u64,
    pub phase: CombatPhase,
    #[serde(with = "token_map_serde")]
    #[schemars(with = "token_map_serde::SchemaHelper")]
    pub enemy_tokens: HashMap<Token, i64>,
    pub encounter_card_id: Option<usize>,
    pub is_finished: bool,
    pub outcome: CombatOutcome,
    pub enemy_attack_deck: Vec<EnemyCardDef>,
    pub enemy_defence_deck: Vec<EnemyCardDef>,
    pub enemy_resource_deck: Vec<EnemyCardDef>,
}

/// Outcome of a combat encounter.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
pub enum CombatOutcome {
    Undecided,
    PlayerWon,
    EnemyWon,
}

// ====== Encounter types for the encounter loop (Step 7) ======

/// Phases of an encounter (Step 7 state machine)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(crate = "rocket::serde")]
pub enum EncounterPhase {
    /// Combat is currently active
    Combat,
    /// Combat has finished; scouting is available
    Scouting,
    /// No active encounter
    NoEncounter,
}
