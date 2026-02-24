// Domain library skeleton
// NOTE: The "Library" here is a domain entity (the canonical collection of CardDef objects and token registry)
// and not necessarily a separate Rust crate. This module is a minimal starting point for the refactor.

//! Minimal domain skeleton for Decks, Tokens, Library and ActionLog
//!
//! This file provides small, well-scoped domain primitives used by higher-level systems.

use rocket::serde::json::Json;
use rocket_okapi::openapi;
use std::collections::HashMap;
use std::sync::atomic::Ordering;

pub mod types {
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

    /// Canonical card definition (minimal)
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub struct CardDef {
        pub id: u64,
        pub card_type: String,
        pub effects: Vec<CardEffect>,
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
            amount: u32,
        },
    }

    /// A single effect a card applies when played.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub struct CardEffect {
        #[serde(flatten)]
        pub kind: CardEffectKind,
        pub lifecycle: TokenLifecycle,
        /// Internal reference to the CardEffect card in the library.
        #[serde(skip)]
        pub card_effect_id: Option<usize>,
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
    #[serde(crate = "rocket::serde", tag = "kind")]
    pub enum CardKind {
        Attack { effects: Vec<CardEffect> },
        Defence { effects: Vec<CardEffect> },
        Resource { effects: Vec<CardEffect> },
        Encounter { encounter_kind: EncounterKind },
        PlayerCardEffect { effect: CardEffect },
        EnemyCardEffect { effect: CardEffect },
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
        pub effects: Vec<CardEffect>,
        pub counts: EnemyCardCounts,
    }

    /// A single entry in the Library. Index in the Vec = card ID.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub struct LibraryCard {
        pub kind: CardKind,
        pub counts: CardCounts,
    }

    /// Token type metadata and lifecycle
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub struct TokenRegistryEntry {
        pub id: TokenType,
        pub cap: Option<u64>,
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
    pub fn token_entry_by_type<'a>(
        map: &'a mut HashMap<Token, i64>,
        tt: &TokenType,
    ) -> &'a mut i64 {
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

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub struct Deck {
        pub id: String,
        pub card_ids: Vec<u64>,
    }

    /// Small, explicit action requests used by the library mutator.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub enum ActionRequest {
        GrantToken { token_id: TokenType, amount: i64 },
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

        pub fn allowed_card_kind(&self) -> &'static str {
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
        pub player_turn: bool,
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

    /// Represents the state of a single encounter session.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub struct EncounterState {
        pub phase: EncounterPhase,
    }

    /// Phases of an encounter (Step 7 state machine)
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
    #[serde(crate = "rocket::serde")]
    pub enum EncounterPhase {
        /// Encounter has been drawn; player can start combat
        Ready,
        /// Combat is currently active
        InCombat,
        /// Combat has finished; scouting is available
        Scouting,
        /// No active encounter
        NoEncounter,
    }

    /// User actions during an encounter (Step 7)
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde", tag = "action_type")]
    pub enum EncounterAction {
        /// Pick an encounter from the area deck and initialize combat
        PickEncounter { card_id: String },
        /// Play a card during combat
        PlayCard { card_id: u64 },
        /// Make a scouting choice post-encounter using card_ids
        ApplyScouting { card_ids: Vec<String> },
        /// System-driven: finish/conclude the encounter (not a player action)
        FinishEncounter,
    }
}

pub mod combat {
    //! Deterministic, pure-data combat resolution (Step 6)
    //!
    //! This module provides pure functions for resolving combat deterministically
    //! using seeded RNG. Current combat scope is minimal: attack cards reduce
    //! opponent HP via token manipulation. Features like dodge, stamina, and
    //! advanced mechanics are deferred to later roadmap steps.

    use super::types::{CardDef, CombatAction, CombatState, EffectTarget};
    use rand::RngCore;
    use rand_pcg::Lcg64Xsh32;
    use std::collections::HashMap;

    /// Result of resolving a combat tick: (updated snapshot, updated player tokens, rng values used).
    pub type CombatTickResult =
        Result<(CombatState, HashMap<super::types::Token, i64>, Vec<u64>), String>;

    /// Resolve a single combat action (card play) deterministically.
    ///
    /// Looks up the card definition, applies its effects as token operations,
    /// checks victory/defeat, and advances the turn.
    /// Returns an error if the card_id is unknown.
    pub fn resolve_combat_tick(
        current_state: &CombatState,
        player_tokens: &HashMap<super::types::Token, i64>,
        action: &CombatAction,
        card_defs: &HashMap<u64, CardDef>,
        rng: &mut Lcg64Xsh32,
    ) -> CombatTickResult {
        let card = card_defs
            .get(&action.card_id)
            .ok_or_else(|| format!("Unknown card_id: {}", action.card_id))?;

        let mut rng_values = Vec::new();
        let mut state_after = current_state.clone();
        let mut pt_after = player_tokens.clone();

        let rng_val = rng.next_u64();
        rng_values.push(rng_val);

        for effect in &card.effects {
            let (target, token_type, amount) = match &effect.kind {
                super::types::CardEffectKind::ChangeTokens {
                    target,
                    token_type,
                    amount,
                } => (target, token_type, *amount),
                super::types::CardEffectKind::DrawCards { .. } => continue,
            };
            let actor_tokens = match (target, action.is_player) {
                (EffectTarget::OnSelf, true) | (EffectTarget::OnOpponent, false) => &mut pt_after,
                (EffectTarget::OnOpponent, true) | (EffectTarget::OnSelf, false) => {
                    &mut state_after.enemy_tokens
                }
            };
            let token_key = super::types::Token {
                token_type: token_type.clone(),
                lifecycle: effect.lifecycle.clone(),
            };
            let entry = actor_tokens.entry(token_key).or_insert(0);
            *entry = (*entry + amount).max(0);

            if *token_type == super::types::TokenType::Health && *entry == 0 {
                state_after.is_finished = true;
                let affected_is_player = matches!(
                    (target, action.is_player),
                    (EffectTarget::OnSelf, true) | (EffectTarget::OnOpponent, false)
                );
                state_after.outcome = if affected_is_player {
                    super::types::CombatOutcome::EnemyWon
                } else {
                    super::types::CombatOutcome::PlayerWon
                };
            }
        }

        if !state_after.is_finished {
            state_after.player_turn = !state_after.player_turn;
        }

        Ok((state_after, pt_after, rng_values))
    }

    /// Simulate a full combat encounter from a seed and initial state.
    ///
    /// Returns the final combat snapshot and player tokens. Pure-data; no side effects.
    pub fn simulate_combat(
        initial_state: CombatState,
        initial_player_tokens: HashMap<super::types::Token, i64>,
        seed: u64,
        actions: Vec<CombatAction>,
        card_defs: &HashMap<u64, CardDef>,
    ) -> (CombatState, HashMap<super::types::Token, i64>) {
        use rand::SeedableRng;

        let seed_bytes: [u8; 16] = {
            let s = seed.to_le_bytes();
            let mut bytes = [0u8; 16];
            bytes[0..8].copy_from_slice(&s);
            bytes[8..16].copy_from_slice(&s);
            bytes
        };
        let mut rng = Lcg64Xsh32::from_seed(seed_bytes);

        let mut current_state = initial_state;
        let mut player_tokens = initial_player_tokens;

        for action in &actions {
            match resolve_combat_tick(&current_state, &player_tokens, action, card_defs, &mut rng) {
                Ok((next_state, next_pt, _rng_vals)) => {
                    current_state = next_state;
                    player_tokens = next_pt;
                    if current_state.is_finished {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        (current_state, player_tokens)
    }
}

pub mod encounter {
    //! Encounter loop state machine (Step 7)
    //!
    //! Pure-data functions that manage encounter state transitions
    //! based on player actions. Works with EncounterAction and EncounterState.

    use super::types::{EncounterAction, EncounterPhase, EncounterState};

    /// Process an EncounterAction and transition state accordingly.
    ///
    /// Returns the new EncounterState after applying the action.
    /// Returns None if the action is invalid for the current phase.
    pub fn apply_action(state: &EncounterState, action: EncounterAction) -> Option<EncounterState> {
        match (&state.phase, action) {
            // Ready phase: can pick encounter or finish
            (EncounterPhase::Ready, EncounterAction::PickEncounter { card_id: _ }) => {
                let mut new_state = state.clone();
                new_state.phase = EncounterPhase::InCombat;
                Some(new_state)
            }
            (EncounterPhase::Ready, EncounterAction::FinishEncounter) => {
                let mut new_state = state.clone();
                new_state.phase = EncounterPhase::NoEncounter;
                Some(new_state)
            }

            // InCombat phase: play cards or end encounter
            (EncounterPhase::InCombat, EncounterAction::PlayCard { .. }) => {
                let new_state = state.clone();
                Some(new_state)
            }
            (EncounterPhase::InCombat, EncounterAction::FinishEncounter) => {
                let mut new_state = state.clone();
                new_state.phase = EncounterPhase::NoEncounter;
                Some(new_state)
            }

            // Scouting phase: apply scouting or finish
            (EncounterPhase::Scouting, EncounterAction::ApplyScouting { card_ids: _ }) => {
                let new_state = state.clone();
                // Scouting keeps encounter in Scouting phase until explicitly finished
                Some(new_state)
            }
            (EncounterPhase::Scouting, EncounterAction::FinishEncounter) => {
                let mut new_state = state.clone();
                new_state.phase = EncounterPhase::NoEncounter;
                Some(new_state)
            }

            // Invalid: all other state/action combinations
            _ => None,
        }
    }

    /// Check if encounter is finished
    pub fn is_finished(state: &EncounterState) -> bool {
        state.phase == EncounterPhase::NoEncounter
    }

    /// Check if combat is active
    pub fn is_in_combat(state: &EncounterState) -> bool {
        state.phase == EncounterPhase::InCombat
    }

    /// Check if post-encounter scouting is available
    pub fn can_scout(state: &EncounterState) -> bool {
        state.phase == EncounterPhase::Scouting
    }

    /// Derive preview count from Foresight tokens.
    ///
    /// Base preview is 1; each Foresight token adds 1 additional preview.
    pub fn derive_preview_count(foresight_tokens: u64) -> u64 {
        1 + foresight_tokens
    }
}

pub mod registry {
    use super::types::{TokenRegistryEntry, TokenType};
    use std::collections::HashMap;

    #[derive(Debug, Default, Clone)]
    pub struct TokenRegistry {
        pub tokens: HashMap<TokenType, TokenRegistryEntry>,
    }

    impl TokenRegistry {
        pub fn new() -> Self {
            Self {
                tokens: HashMap::new(),
            }
        }
        pub fn register(&mut self, token: TokenRegistryEntry) {
            self.tokens.insert(token.id.clone(), token);
        }

        /// Create a minimal canonical token registry seeded from vision.md
        pub fn with_canonical() -> Self {
            let mut r = Self::new();
            for id in [
                TokenType::Insight,
                TokenType::Renown,
                TokenType::Refinement,
                TokenType::Stability,
                TokenType::Foresight,
                TokenType::Momentum,
                TokenType::Corruption,
                TokenType::Exhaustion,
                TokenType::Durability,
                TokenType::Health,
                TokenType::MaxHealth,
                TokenType::Shield,
                TokenType::Stamina,
                TokenType::Mana,
                TokenType::Dodge,
            ] {
                r.register(TokenRegistryEntry {
                    id,
                    cap: Some(9999),
                });
            }
            r
        }

        pub fn contains(&self, id: &TokenType) -> bool {
            self.tokens.contains_key(id)
        }
    }
}

pub mod action_log {
    use super::types::{ActionEntry, ActionPayload};
    use crate::action::persistence::FileWriter;
    use std::fs::{File, OpenOptions};
    use std::io::{BufRead, BufReader, Write};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{mpsc, Arc, Mutex};
    use std::thread;

    #[derive(Debug)]
    pub struct ActionLog {
        pub entries: Arc<Mutex<Vec<ActionEntry>>>,
        pub seq: AtomicU64,
        pub sender: mpsc::Sender<ActionEntry>,
        pub writer: Option<FileWriter>,
    }

    impl Clone for ActionLog {
        fn clone(&self) -> Self {
            // snapshot existing entries and seq
            let entries_vec = match self.entries.lock() {
                Ok(g) => g.clone(),
                Err(e) => e.into_inner().clone(),
            };
            let seq_val = self.seq.load(Ordering::SeqCst);
            // create a fresh ActionLog (spawns its own worker)
            let new = ActionLog::new();
            // replace entries with the snapshot
            match new.entries.lock() {
                Ok(mut g) => *g = entries_vec,
                Err(err) => *err.into_inner() = entries_vec,
            }
            new.seq.store(seq_val, Ordering::SeqCst);
            Self {
                entries: new.entries,
                seq: new.seq,
                sender: new.sender,
                writer: self.writer.clone(),
            }
        }
    }

    impl Default for ActionLog {
        fn default() -> Self {
            ActionLog::new()
        }
    }

    impl ActionLog {
        pub fn new() -> Self {
            let entries = Arc::new(Mutex::new(Vec::new()));
            let (tx, rx) = mpsc::channel::<ActionEntry>();
            let _worker_entries = Arc::clone(&entries);
            thread::spawn(move || {
                // Dedicated worker receives entries for offloaded processing (persistence, analytics, etc.).
                // Currently it consumes the channel and drops entries after receipt to keep the worker alive
                // without duplicating in-memory storage (append() writes directly into entries).
                for _entry in rx {
                    // placeholder: persist or forward the entry to external systems
                }
            });
            ActionLog {
                entries,
                seq: AtomicU64::new(0),
                sender: tx,
                writer: None,
            }
        }

        pub fn set_writer(&mut self, writer: Option<FileWriter>) {
            self.writer = writer;
        }

        pub fn load_from_file(path: &str) -> Result<ActionLog, String> {
            let file = File::open(path).map_err(|e| e.to_string())?;
            let reader = BufReader::new(file);
            let mut entries = Vec::new();
            let mut max_seq = 0u64;
            for line in reader.lines() {
                let line = line.map_err(|e| e.to_string())?;
                if line.trim().is_empty() {
                    continue;
                }
                let entry: ActionEntry = serde_json::from_str(&line).map_err(|e| e.to_string())?;
                if entry.seq > max_seq {
                    max_seq = entry.seq;
                }
                entries.push(entry);
            }
            let log = ActionLog::new();
            {
                match log.entries.lock() {
                    Ok(mut g) => *g = entries,
                    Err(e) => *e.into_inner() = entries,
                };
            }
            log.seq.store(max_seq, Ordering::SeqCst);
            Ok(log)
        }

        pub fn write_all_to_file(&self, path: &str) -> Result<(), String> {
            let entries = self.entries();
            let mut f = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)
                .map_err(|e| e.to_string())?;
            for e in entries {
                let line = serde_json::to_string(&e).map_err(|e| e.to_string())?;
                writeln!(f, "{}", line).map_err(|e| e.to_string())?;
            }
            f.flush().map_err(|e| e.to_string())
        }

        /// Append an action entry, assigning an incrementing sequence number.
        /// This implementation writes into the in-memory entries immediately (synchronously)
        /// and also sends the entry to a background worker for offloaded tasks.
        pub fn append(&self, action_type: &str, payload: ActionPayload) -> ActionEntry {
            self.append_with_meta(action_type, payload, None, None, None)
        }

        pub fn append_with_meta(
            &self,
            action_type: &str,
            payload: ActionPayload,
            actor: Option<String>,
            request_id: Option<String>,
            version: Option<u32>,
        ) -> ActionEntry {
            let seq = self.seq.fetch_add(1, Ordering::SeqCst) + 1;
            let timestamp = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            {
                Ok(dur) => format!("{}", dur.as_millis()),
                Err(_) => "0".to_string(),
            };
            let entry = ActionEntry {
                seq,
                action_type: action_type.to_string(),
                payload: payload.clone(),
                timestamp,
                actor,
                request_id,
                version,
            };
            // write into in-memory entries immediately to preserve synchronous semantics
            match self.entries.lock() {
                Ok(mut g) => g.push(entry.clone()),
                Err(e) => e.into_inner().push(entry.clone()),
            }
            // best-effort send to worker for offloaded processing; ignore errors if the worker has shut down
            let _ = self.sender.send(entry.clone());
            entry
        }

        /// Return a cloned snapshot of entries for replay/inspection
        pub fn entries(&self) -> Vec<ActionEntry> {
            match self.entries.lock() {
                Ok(g) => g.clone(),
                Err(e) => e.into_inner().clone(),
            }
        }

        /// Async wrapper for compatibility with async callsites.
        pub async fn append_async(
            self: Arc<Self>,
            action_type: &str,
            payload: ActionPayload,
        ) -> ActionEntry {
            // append is non-blocking (sends to worker) so this can be synchronous
            self.append(action_type, payload)
        }
    }
}

use action_log::ActionLog;
use registry::TokenRegistry;
use types::{
    ActionEntry, ActionPayload, CardCounts, CardEffect, CardKind, EncounterKind, LibraryCard,
};

/// The Library: canonical collection of all player-owned cards.
/// Index in the Vec = card ID. Per vision "card location model and counts".
#[derive(Debug, Clone)]
pub struct Library {
    pub cards: Vec<LibraryCard>,
}

impl Default for Library {
    fn default() -> Self {
        Self::new()
    }
}

impl Library {
    pub fn new() -> Self {
        Library { cards: Vec::new() }
    }

    /// Add a card to the library. Returns the card ID (index).
    pub fn add_card(&mut self, kind: CardKind, counts: CardCounts) -> usize {
        let id = self.cards.len();
        self.cards.push(LibraryCard { kind, counts });
        id
    }

    /// Get a card by ID (index).
    pub fn get(&self, card_id: usize) -> Option<&LibraryCard> {
        self.cards.get(card_id)
    }

    /// Draw a card: move one copy from deck → hand.
    pub fn draw(&mut self, card_id: usize) -> Result<(), String> {
        let card = self
            .cards
            .get_mut(card_id)
            .ok_or_else(|| format!("Card {card_id} not found"))?;
        if card.counts.deck == 0 {
            return Err(format!("Card {card_id} has no copies in deck"));
        }
        card.counts.deck -= 1;
        card.counts.hand += 1;
        Ok(())
    }

    /// Play/discard a card: move one copy from hand → discard.
    pub fn play(&mut self, card_id: usize) -> Result<(), String> {
        let card = self
            .cards
            .get_mut(card_id)
            .ok_or_else(|| format!("Card {card_id} not found"))?;
        if card.counts.hand == 0 {
            return Err(format!("Card {card_id} has no copies in hand"));
        }
        card.counts.hand -= 1;
        card.counts.discard += 1;
        Ok(())
    }

    /// Return a card from discard → library.
    pub fn return_to_library(&mut self, card_id: usize) -> Result<(), String> {
        let card = self
            .cards
            .get_mut(card_id)
            .ok_or_else(|| format!("Card {card_id} not found"))?;
        if card.counts.discard == 0 {
            return Err(format!("Card {card_id} has no copies in discard"));
        }
        card.counts.discard -= 1;
        card.counts.library += 1;
        Ok(())
    }

    /// Move copies from library → deck (adding cards to your deck).
    pub fn add_to_deck(&mut self, card_id: usize, count: u32) -> Result<(), String> {
        let card = self
            .cards
            .get_mut(card_id)
            .ok_or_else(|| format!("Card {card_id} not found"))?;
        if card.counts.library < count {
            return Err(format!(
                "Card {card_id} has only {} copies in library, need {count}",
                card.counts.library
            ));
        }
        card.counts.library -= count;
        card.counts.deck += count;
        Ok(())
    }

    /// All cards currently on hand.
    pub fn hand_cards(&self) -> Vec<(usize, &LibraryCard)> {
        self.cards
            .iter()
            .enumerate()
            .filter(|(_, c)| c.counts.hand > 0)
            .collect()
    }

    /// All cards matching a predicate on CardKind.
    pub fn cards_matching<F>(&self, predicate: F) -> Vec<(usize, &LibraryCard)>
    where
        F: Fn(&CardKind) -> bool,
    {
        self.cards
            .iter()
            .enumerate()
            .filter(|(_, c)| predicate(&c.kind))
            .collect()
    }

    /// Return a card from discard → deck (recycle).
    pub fn return_to_deck(&mut self, card_id: usize) -> Result<(), String> {
        let card = self
            .cards
            .get_mut(card_id)
            .ok_or_else(|| format!("Card {card_id} not found"))?;
        if card.counts.discard == 0 {
            return Err(format!("Card {card_id} has no copies in discard"));
        }
        card.counts.discard -= 1;
        card.counts.deck += 1;
        Ok(())
    }

    /// Encounter cards currently in the hand (visible/pickable).
    pub fn encounter_hand(&self) -> Vec<usize> {
        self.cards
            .iter()
            .enumerate()
            .filter(|(_, c)| matches!(c.kind, CardKind::Encounter { .. }) && c.counts.hand > 0)
            .flat_map(|(id, c)| std::iter::repeat_n(id, c.counts.hand as usize))
            .collect()
    }

    /// Check if an encounter card is in the hand.
    pub fn encounter_contains(&self, card_id: usize) -> bool {
        self.cards
            .get(card_id)
            .is_some_and(|c| matches!(c.kind, CardKind::Encounter { .. }) && c.counts.hand > 0)
    }

    /// Draw encounter cards from deck to hand until hand reaches target_count.
    pub fn encounter_draw_to_hand(&mut self, target_count: usize) {
        let current_hand: usize = self
            .cards
            .iter()
            .filter(|c| matches!(c.kind, CardKind::Encounter { .. }) && c.counts.hand > 0)
            .map(|c| c.counts.hand as usize)
            .sum();
        let mut remaining = target_count.saturating_sub(current_hand);
        for card in &mut self.cards {
            if remaining == 0 {
                break;
            }
            if matches!(card.kind, CardKind::Encounter { .. }) && card.counts.deck > 0 {
                let to_move = (card.counts.deck as usize).min(remaining) as u32;
                card.counts.deck -= to_move;
                card.counts.hand += to_move;
                remaining -= to_move as usize;
            }
        }
    }

    /// Validate that all card effects reference valid CardEffect deck entries.
    pub fn validate_card_effects(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        for (id, card) in self.cards.iter().enumerate() {
            match &card.kind {
                CardKind::Attack { effects }
                | CardKind::Defence { effects }
                | CardKind::Resource { effects } => {
                    for effect in effects {
                        if let Some(ref_id) = effect.card_effect_id {
                            match self.cards.get(ref_id) {
                                Some(ref_card)
                                    if matches!(
                                        ref_card.kind,
                                        CardKind::PlayerCardEffect { .. }
                                    ) => {}
                                _ => errors.push(format!(
                                    "Card {} has effect referencing invalid PlayerCardEffect {}",
                                    id, ref_id
                                )),
                            }
                        } else {
                            errors.push(format!("Card {} has effect without card_effect_id", id));
                        }
                    }
                }
                CardKind::Encounter {
                    encounter_kind: EncounterKind::Combat { combatant_def },
                } => {
                    for deck in [
                        &combatant_def.attack_deck,
                        &combatant_def.defence_deck,
                        &combatant_def.resource_deck,
                    ] {
                        for enemy_card in deck {
                            for effect in &enemy_card.effects {
                                if let Some(ref_id) = effect.card_effect_id {
                                    match self.cards.get(ref_id) {
                                        Some(ref_card)
                                            if matches!(
                                                ref_card.kind,
                                                CardKind::EnemyCardEffect { .. }
                                            ) => {}
                                        _ => errors.push(format!(
                                            "Enemy card in card {} has effect referencing invalid EnemyCardEffect {}",
                                            id, ref_id
                                        )),
                                    }
                                } else {
                                    errors.push(format!(
                                        "Enemy card in card {} has effect without card_effect_id",
                                        id
                                    ));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
fn initialize_library() -> Library {
    let mut lib = Library::new();

    // ---- Player CardEffect deck entries ----

    // id 0: Player "deal 5 damage" effect
    let player_damage_effect = CardEffect {
        kind: types::CardEffectKind::ChangeTokens {
            target: types::EffectTarget::OnOpponent,
            token_type: types::TokenType::Health,
            amount: -5,
        },
        lifecycle: types::TokenLifecycle::PersistentCounter,
        card_effect_id: Some(0),
    };
    lib.add_card(
        CardKind::PlayerCardEffect {
            effect: player_damage_effect.clone(),
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // id 1: Player "grant 3 shield" effect
    let player_shield_effect = CardEffect {
        kind: types::CardEffectKind::ChangeTokens {
            target: types::EffectTarget::OnSelf,
            token_type: types::TokenType::Shield,
            amount: 3,
        },
        lifecycle: types::TokenLifecycle::PersistentCounter,
        card_effect_id: Some(1),
    };
    lib.add_card(
        CardKind::PlayerCardEffect {
            effect: player_shield_effect.clone(),
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // id 2: Player "grant 2 stamina" effect
    let player_stamina_effect = CardEffect {
        kind: types::CardEffectKind::ChangeTokens {
            target: types::EffectTarget::OnSelf,
            token_type: types::TokenType::Stamina,
            amount: 2,
        },
        lifecycle: types::TokenLifecycle::PersistentCounter,
        card_effect_id: Some(2),
    };
    lib.add_card(
        CardKind::PlayerCardEffect {
            effect: player_stamina_effect.clone(),
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // id 3: Player "draw 2 cards" effect
    let player_draw_effect = CardEffect {
        kind: types::CardEffectKind::DrawCards { amount: 2 },
        lifecycle: types::TokenLifecycle::PersistentCounter,
        card_effect_id: Some(3),
    };
    lib.add_card(
        CardKind::PlayerCardEffect {
            effect: player_draw_effect.clone(),
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // ---- Enemy CardEffect deck entries ----

    // id 4: Enemy "deal 3 damage" effect
    let enemy_damage_effect = CardEffect {
        kind: types::CardEffectKind::ChangeTokens {
            target: types::EffectTarget::OnOpponent,
            token_type: types::TokenType::Health,
            amount: -3,
        },
        lifecycle: types::TokenLifecycle::PersistentCounter,
        card_effect_id: Some(4),
    };
    lib.add_card(
        CardKind::EnemyCardEffect {
            effect: enemy_damage_effect.clone(),
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // id 5: Enemy "grant 2 shield" effect
    let enemy_shield_effect = CardEffect {
        kind: types::CardEffectKind::ChangeTokens {
            target: types::EffectTarget::OnSelf,
            token_type: types::TokenType::Shield,
            amount: 2,
        },
        lifecycle: types::TokenLifecycle::PersistentCounter,
        card_effect_id: Some(5),
    };
    lib.add_card(
        CardKind::EnemyCardEffect {
            effect: enemy_shield_effect.clone(),
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // id 6: Enemy "grant 1 stamina" effect
    let enemy_stamina_effect = CardEffect {
        kind: types::CardEffectKind::ChangeTokens {
            target: types::EffectTarget::OnSelf,
            token_type: types::TokenType::Stamina,
            amount: 1,
        },
        lifecycle: types::TokenLifecycle::PersistentCounter,
        card_effect_id: Some(6),
    };
    lib.add_card(
        CardKind::EnemyCardEffect {
            effect: enemy_stamina_effect.clone(),
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // id 7: Enemy "draw 2 cards" effect
    let enemy_draw_effect = CardEffect {
        kind: types::CardEffectKind::DrawCards { amount: 2 },
        lifecycle: types::TokenLifecycle::PersistentCounter,
        card_effect_id: Some(7),
    };
    lib.add_card(
        CardKind::EnemyCardEffect {
            effect: enemy_draw_effect.clone(),
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // ---- Player action cards (reference CardEffect cards) ----

    // Attack card (id 8): deals 5 damage to opponent
    lib.add_card(
        CardKind::Attack {
            effects: vec![CardEffect {
                card_effect_id: Some(0),
                ..player_damage_effect
            }],
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
    );

    // Defence card (id 9): grants 3 shield to self
    lib.add_card(
        CardKind::Defence {
            effects: vec![CardEffect {
                card_effect_id: Some(1),
                ..player_shield_effect
            }],
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
    );

    // Resource card (id 10): grants 2 stamina to self, draws 1 card
    lib.add_card(
        CardKind::Resource {
            effects: vec![
                CardEffect {
                    card_effect_id: Some(2),
                    ..player_stamina_effect
                },
                CardEffect {
                    card_effect_id: Some(3),
                    ..player_draw_effect
                },
            ],
        },
        CardCounts {
            library: 0,
            deck: 35,
            hand: 5,
            discard: 0,
        },
    );

    // Combat encounter: Gnome (id 11)
    lib.add_card(
        CardKind::Encounter {
            encounter_kind: types::EncounterKind::Combat {
                combatant_def: types::CombatantDef {
                    initial_tokens: HashMap::from([
                        (types::Token::persistent(types::TokenType::Health), 20),
                        (types::Token::persistent(types::TokenType::MaxHealth), 20),
                    ]),
                    attack_deck: vec![types::EnemyCardDef {
                        effects: vec![CardEffect {
                            card_effect_id: Some(4),
                            ..enemy_damage_effect
                        }],
                        counts: types::EnemyCardCounts {
                            deck: 0,
                            hand: 10,
                            discard: 0,
                        },
                    }],
                    defence_deck: vec![types::EnemyCardDef {
                        effects: vec![CardEffect {
                            card_effect_id: Some(5),
                            ..enemy_shield_effect
                        }],
                        counts: types::EnemyCardCounts {
                            deck: 0,
                            hand: 10,
                            discard: 0,
                        },
                    }],
                    resource_deck: vec![types::EnemyCardDef {
                        effects: vec![
                            CardEffect {
                                card_effect_id: Some(6),
                                ..enemy_stamina_effect
                            },
                            CardEffect {
                                card_effect_id: Some(7),
                                ..enemy_draw_effect
                            },
                        ],
                        counts: types::EnemyCardCounts {
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

    if let Err(errors) = lib.validate_card_effects() {
        panic!("Library card effect validation failed: {:?}", errors);
    }

    lib
}

/// Apply card effects to combat, modifying player tokens and combat snapshot.
/// Only processes ChangeTokens effects; DrawCards effects are handled separately.
fn apply_card_effects(
    effects: &[CardEffect],
    is_player: bool,
    player_tokens: &mut HashMap<types::Token, i64>,
    combat: &mut types::CombatState,
) {
    for effect in effects {
        let (target, token_type, amount) = match &effect.kind {
            types::CardEffectKind::ChangeTokens {
                target,
                token_type,
                amount,
            } => (target, token_type, *amount),
            types::CardEffectKind::DrawCards { .. } => continue,
        };

        let target_tokens = match (target, is_player) {
            (types::EffectTarget::OnSelf, true) | (types::EffectTarget::OnOpponent, false) => {
                &mut *player_tokens
            }
            (types::EffectTarget::OnOpponent, true) | (types::EffectTarget::OnSelf, false) => {
                &mut combat.enemy_tokens
            }
        };

        if *token_type == types::TokenType::Health && amount < 0 {
            // Damage: consume dodge first, then reduce health
            let damage = -amount;
            let dodge = target_tokens
                .get(&types::Token::dodge())
                .copied()
                .unwrap_or(0);
            let absorbed = dodge.min(damage);
            target_tokens.insert(types::Token::dodge(), (dodge - absorbed).max(0));
            let remaining_damage = damage - absorbed;
            if remaining_damage > 0 {
                let health = target_tokens
                    .entry(types::Token::persistent(types::TokenType::Health))
                    .or_insert(0);
                *health = (*health - remaining_damage).max(0);
            }
        } else {
            let entry = target_tokens
                .entry(types::Token {
                    token_type: token_type.clone(),
                    lifecycle: effect.lifecycle.clone(),
                })
                .or_insert(0);
            *entry = (*entry + amount).max(0);
        }
    }
}

/// Check if combat has ended (either side at 0 health).
fn check_combat_end(player_tokens: &HashMap<types::Token, i64>, combat: &mut types::CombatState) {
    let player_health = player_tokens
        .get(&types::Token::persistent(types::TokenType::Health))
        .copied()
        .unwrap_or(0);
    let enemy_health = combat
        .enemy_tokens
        .get(&types::Token::persistent(types::TokenType::Health))
        .copied()
        .unwrap_or(0);

    if enemy_health <= 0 || player_health <= 0 {
        combat.is_finished = true;
        combat.outcome = if enemy_health <= 0 && player_health > 0 {
            types::CombatOutcome::PlayerWon
        } else if player_health <= 0 && enemy_health > 0 {
            types::CombatOutcome::EnemyWon
        } else {
            types::CombatOutcome::PlayerWon // Draw defaults to player
        };
    }
}

/// Minimal in-memory game state driven by the library's mutator API.
#[derive(Debug, Clone)]
pub struct GameState {
    pub registry: TokenRegistry,
    pub action_log: std::sync::Arc<ActionLog>,
    pub token_balances: HashMap<types::Token, i64>,
    pub library: Library,
    pub current_combat: Option<types::CombatState>,
    pub encounter_state: types::EncounterState,
    pub last_combat_result: Option<types::CombatOutcome>,
}

impl GameState {
    /// Create a new game state seeded with the canonical token registry
    pub fn new() -> Self {
        let registry = TokenRegistry::with_canonical();
        let mut balances = HashMap::new();
        for id in registry.tokens.keys() {
            balances.insert(types::Token::persistent(id.clone()), 0i64);
        }
        // Default Foresight controls area deck hand size
        balances.insert(types::Token::persistent(types::TokenType::Foresight), 3);
        let _action_log = match std::env::var("ACTION_LOG_FILE") {
            Ok(path) => {
                #[allow(clippy::manual_unwrap_or_default)]
                let mut log = match action_log::ActionLog::load_from_file(&path) {
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
            registry,
            action_log: std::sync::Arc::new(ActionLog::new()),
            token_balances: balances,
            library: initialize_library(),
            current_combat: None,
            encounter_state: types::EncounterState {
                phase: types::EncounterPhase::Ready,
            },
            last_combat_result: None,
        }
    }

    /// Apply a simple GrantToken action: update balances and append to the action log.
    /// Apply a grant operation: add to token balances with cap checking.
    pub fn apply_grant(
        &mut self,
        token_id: &types::TokenType,
        amount: i64,
        _reason: Option<String>,
    ) -> Result<(), String> {
        let token_type = self
            .registry
            .tokens
            .get(token_id)
            .ok_or_else(|| format!("Unknown token '{:?}'", token_id))?;

        // Check cap if present
        if let Some(cap) = token_type.cap {
            let current = types::token_balance_by_type(&self.token_balances, token_id);
            if current + amount > cap as i64 {
                return Err(format!(
                    "Token '{:?}' would exceed cap of {} (current: {})",
                    token_id, cap, current
                ));
            }
        }

        let v = self
            .token_balances
            .entry(types::Token::persistent(token_id.clone()))
            .or_insert(0);
        *v += amount;
        Ok(())
    }

    /// Apply a consume operation: deduct from token balances.
    pub fn apply_consume(
        &mut self,
        token_id: &types::TokenType,
        amount: i64,
        _reason: Option<String>,
    ) -> Result<(), String> {
        if !self.registry.contains(token_id) {
            return Err(format!("Unknown token '{:?}'", token_id));
        }

        let current = types::token_balance_by_type(&self.token_balances, token_id);
        if current < amount {
            return Err(format!(
                "Cannot consume {} of token '{:?}': insufficient balance (have {})",
                amount, token_id, current
            ));
        }

        let v = self
            .token_balances
            .entry(types::Token::persistent(token_id.clone()))
            .or_insert(0);
        *v -= amount;
        Ok(())
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
        let snapshot = types::CombatState {
            round: 1,
            player_turn: true,
            phase: types::CombatPhase::Defending,
            enemy_tokens: combatant_def.initial_tokens.clone(),
            encounter_card_id: Some(encounter_card_id),
            is_finished: false,
            outcome: types::CombatOutcome::Undecided,
            enemy_attack_deck,
            enemy_defence_deck,
            enemy_resource_deck,
        };
        self.current_combat = Some(snapshot);
        self.encounter_state.phase = types::EncounterPhase::InCombat;
        Ok(())
    }

    /// Resolve a player card play against the current combat snapshot.
    pub fn resolve_player_card(&mut self, card_id: usize) -> Result<(), String> {
        let combat = self.current_combat.as_mut().ok_or("No active combat")?;
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
        let draw_count: u32 = effects
            .iter()
            .filter_map(|e| match &e.kind {
                types::CardEffectKind::DrawCards { amount } => Some(*amount),
                _ => None,
            })
            .sum();
        apply_card_effects(&effects, true, &mut self.token_balances, combat);
        check_combat_end(&self.token_balances, combat);
        if combat.is_finished {
            self.last_combat_result = Some(combat.outcome.clone());
            self.current_combat = None;
            self.encounter_state.phase = types::EncounterPhase::Scouting;
        }
        if draw_count > 0 {
            self.draw_random_cards(draw_count);
        }
        Ok(())
    }

    /// Draw random cards from deck to hand (for resource card draw mechanic).
    fn draw_random_cards(&mut self, count: u32) {
        let drawable: Vec<usize> = self
            .library
            .cards
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                c.counts.deck > 0
                    && !matches!(
                        c.kind,
                        CardKind::Encounter { .. }
                            | CardKind::PlayerCardEffect { .. }
                            | CardKind::EnemyCardEffect { .. }
                    )
            })
            .map(|(i, _)| i)
            .collect();
        if drawable.is_empty() {
            return;
        }
        for i in 0..count {
            let idx = drawable[i as usize % drawable.len()];
            let _ = self.library.draw(idx);
        }
    }

    /// Resolve an enemy card play from hand in the current combat phase.
    /// Played cards move to discard. Resource cards with DrawCards trigger enemy draws.
    pub fn resolve_enemy_play(&mut self, rng: &mut rand_pcg::Lcg64Xsh32) -> Result<(), String> {
        let combat = self.current_combat.as_ref().ok_or("No active combat")?;
        let phase = combat.phase.clone();

        let combat = self.current_combat.as_mut().ok_or("No active combat")?;
        let deck = match phase {
            types::CombatPhase::Attacking => &mut combat.enemy_attack_deck,
            types::CombatPhase::Defending => &mut combat.enemy_defence_deck,
            types::CombatPhase::Resourcing => &mut combat.enemy_resource_deck,
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

            let draw_count: u32 = effects
                .iter()
                .filter_map(|e| match &e.kind {
                    types::CardEffectKind::DrawCards { amount } => Some(*amount),
                    _ => None,
                })
                .sum();

            apply_card_effects(&effects, false, &mut self.token_balances, combat);
            check_combat_end(&self.token_balances, combat);

            // Handle enemy draws from resource cards
            if draw_count > 0 && !combat.is_finished {
                let resource_deck = &mut combat.enemy_resource_deck;
                for _ in 0..draw_count {
                    Self::enemy_draw_random(rng, resource_deck);
                }
            }

            if combat.is_finished {
                self.last_combat_result = Some(combat.outcome.clone());
                self.current_combat = None;
                self.encounter_state.phase = types::EncounterPhase::Scouting;
            }
        }
        Ok(())
    }

    /// Shuffle enemy hand: move all cards to deck, then draw random cards back to hand.
    fn enemy_shuffle_hand(rng: &mut rand_pcg::Lcg64Xsh32, deck: &mut [types::EnemyCardDef]) {
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
    fn enemy_draw_random(rng: &mut rand_pcg::Lcg64Xsh32, deck: &mut [types::EnemyCardDef]) {
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
        let combat = self.current_combat.as_mut().ok_or("No active combat")?;
        combat.phase = combat.phase.next();
        Ok(())
    }

    /// Reconstruct state from a registry and an existing action log.
    /// The RNG is initialized from the first `SetSeed` entry in the log.
    pub fn replay_from_log(registry: TokenRegistry, log: &ActionLog) -> Self {
        use rand::SeedableRng;

        let mut gs = {
            let mut balances = HashMap::new();
            for id in registry.tokens.keys() {
                balances.insert(types::Token::persistent(id.clone()), 0i64);
            }
            // Default Foresight controls area deck hand size
            balances.insert(types::Token::persistent(types::TokenType::Foresight), 3);
            Self {
                registry,
                action_log: std::sync::Arc::new(ActionLog::new()),
                token_balances: balances,
                library: initialize_library(),
                current_combat: None,
                encounter_state: types::EncounterState {
                    phase: types::EncounterPhase::Ready,
                },
                last_combat_result: None,
            }
        };
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
                    gs.current_combat = None;
                    gs.encounter_state = new_gs.encounter_state;
                    gs.last_combat_result = None;
                }
                ActionPayload::DrawEncounter { encounter_id, .. } => {
                    if let Ok(card_id) = encounter_id.parse::<usize>() {
                        let health_key = types::Token::persistent(types::TokenType::Health);
                        if gs.token_balances.get(&health_key).copied().unwrap_or(0) == 0 {
                            gs.token_balances.insert(health_key, 20);
                        }
                        let _ = gs.library.play(card_id);
                        let _ = gs.start_combat(card_id, &mut rng);
                    }
                }
                ActionPayload::PlayCard { card_id, .. } => {
                    let _ = gs.library.play(*card_id);
                    let _ = gs.resolve_player_card(*card_id);
                    if gs.current_combat.is_some() {
                        let _ = gs.resolve_enemy_play(&mut rng);
                        if gs.current_combat.is_some() {
                            let _ = gs.advance_combat_phase();
                        }
                    }
                }
                ActionPayload::ApplyScouting { .. } => {
                    if let Some(ref combat) = gs.current_combat {
                        if let Some(enc_id) = combat.encounter_card_id {
                            let _ = gs.library.return_to_deck(enc_id);
                        }
                    }
                    let foresight = gs
                        .token_balances
                        .get(&types::Token::persistent(types::TokenType::Foresight))
                        .copied()
                        .unwrap_or(3) as usize;
                    gs.library.encounter_draw_to_hand(foresight);
                    gs.encounter_state.phase = types::EncounterPhase::Ready;
                }
                ActionPayload::GrantToken {
                    token_id, amount, ..
                } => {
                    let v = gs
                        .token_balances
                        .entry(types::Token::persistent(token_id.clone()))
                        .or_insert(0);
                    *v += *amount;
                }
                ActionPayload::ConsumeToken {
                    token_id, amount, ..
                } => {
                    let v = gs
                        .token_balances
                        .entry(types::Token::persistent(token_id.clone()))
                        .or_insert(0);
                    *v -= *amount;
                }
                ActionPayload::ExpireToken {
                    token_id, amount, ..
                } => {
                    let v = gs
                        .token_balances
                        .entry(types::Token::persistent(token_id.clone()))
                        .or_insert(0);
                    *v = (*v - *amount).max(0);
                }
                _ => {
                    // RngDraw, RngSnapshot, etc. are audit entries
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

// tests moved to tests/library_unit.rs

/// Get canonical token registry with full token details
#[openapi]
#[get("/tokens")]
pub async fn list_library_tokens() -> Json<Vec<types::TokenRegistryEntry>> {
    let reg = TokenRegistry::with_canonical();
    Json(reg.tokens.into_values().collect())
}

/// Library cards endpoint: returns all cards from the canonical Library.
#[openapi]
#[get("/library/cards")]
pub async fn list_library_cards(
    game_state: &rocket::State<std::sync::Arc<rocket::futures::lock::Mutex<GameState>>>,
) -> Json<Vec<types::LibraryCard>> {
    let gs = game_state.lock().await;
    Json(gs.library.cards.clone())
}

/// Test endpoint: add a card to the Library with specified kind and counts.
#[openapi]
#[post("/tests/library/cards", data = "<card>")]
pub async fn add_test_library_card(
    card: Json<types::LibraryCard>,
    game_state: &rocket::State<std::sync::Arc<rocket::futures::lock::Mutex<GameState>>>,
) -> rocket::response::status::Created<String> {
    let mut gs = game_state.lock().await;
    let id = gs.library.add_card(card.0.kind, card.0.counts);
    rocket::response::status::Created::new(format!("/library/cards/{}", id))
}

/// A single card effect entry with its library ID.
#[derive(
    Debug, Clone, rocket::serde::Serialize, rocket::serde::Deserialize, rocket_okapi::JsonSchema,
)]
#[serde(crate = "rocket::serde")]
pub struct CardEffectEntry {
    pub id: usize,
    pub card: types::LibraryCard,
}

/// Response for the card effects endpoint.
#[derive(
    Debug, Clone, rocket::serde::Serialize, rocket::serde::Deserialize, rocket_okapi::JsonSchema,
)]
#[serde(crate = "rocket::serde")]
pub struct CardEffectsResponse {
    pub player_effects: Vec<CardEffectEntry>,
    pub enemy_effects: Vec<CardEffectEntry>,
}

/// List all CardEffect deck entries (player and enemy).
#[openapi]
#[get("/library/card-effects")]
pub async fn list_card_effects(
    game_state: &rocket::State<std::sync::Arc<rocket::futures::lock::Mutex<GameState>>>,
) -> Json<CardEffectsResponse> {
    let gs = game_state.lock().await;
    let player_effects: Vec<CardEffectEntry> = gs
        .library
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| matches!(c.kind, CardKind::PlayerCardEffect { .. }))
        .map(|(i, c)| CardEffectEntry {
            id: i,
            card: c.clone(),
        })
        .collect();
    let enemy_effects: Vec<CardEffectEntry> = gs
        .library
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| matches!(c.kind, CardKind::EnemyCardEffect { .. }))
        .map(|(i, c)| CardEffectEntry {
            id: i,
            card: c.clone(),
        })
        .collect();
    Json(CardEffectsResponse {
        player_effects,
        enemy_effects,
    })
}
