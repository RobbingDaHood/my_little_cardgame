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
    /// Canonical card definition (minimal)
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub struct CardDef {
        pub id: u64,
        pub card_type: String,
        pub effects: Vec<CardEffect>,
    }

    /// A single effect a card applies when played.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub struct CardEffect {
        pub target: EffectTarget,
        pub token_id: String,
        pub amount: i64,
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
        Attack {
            effects: Vec<CardEffect>,
        },
        Defence {
            effects: Vec<CardEffect>,
        },
        Resource {
            effects: Vec<CardEffect>,
            draw_count: u32,
        },
        CombatEncounter {
            combatant_def: CombatantDef,
        },
    }

    /// Definition of an enemy combatant for a combat encounter card.
    /// Enemies are self-contained: their cards are inline, not Library references.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub struct CombatantDef {
        pub initial_tokens: HashMap<String, i64>,
        pub attack_deck: Vec<EnemyCardDef>,
        pub defence_deck: Vec<EnemyCardDef>,
        pub resource_deck: Vec<EnemyCardDef>,
    }

    /// A simple inline card definition for enemy decks.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub struct EnemyCardDef {
        pub effects: Vec<CardEffect>,
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
    pub struct TokenType {
        pub id: String,
        pub lifecycle: TokenLifecycle,
        pub cap: Option<u64>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub enum TokenLifecycle {
        Permanent,
        PersistentCounter,
        FixedDuration {
            duration: u64,
        },
        FixedTypeDuration {
            duration: u64,
            phases: Vec<EncounterPhase>,
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
        GrantToken { token_id: String, amount: i64 },
    }

    /// Action payloads for the append-only log
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde", tag = "type")]
    pub enum ActionPayload {
        GrantToken {
            token_id: String,
            amount: i64,
            reason: Option<String>,
            resulting_amount: i64,
        },
        ConsumeToken {
            token_id: String,
            amount: i64,
            reason: Option<String>,
            resulting_amount: i64,
        },
        ExpireToken {
            token_id: String,
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

    /// Represents a combatant (player or enemy) in combat.
    /// Pure-data representation of combat state.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub struct Combatant {
        pub active_tokens: HashMap<String, i64>,
    }

    /// A combat action is a card play by a combatant.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub struct CombatAction {
        pub is_player: bool,
        pub card_id: u64,
    }

    /// Combat phases for turn-based combat.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
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
    pub struct CombatSnapshot {
        pub round: u64,
        pub player_turn: bool,
        pub phase: CombatPhase,
        pub player_tokens: HashMap<String, i64>,
        pub enemy: Combatant,
        pub encounter_card_id: Option<usize>,
        pub is_finished: bool,
        pub winner: Option<String>,
    }

    /// Result of a completed combat.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub struct CombatResult {
        pub winner: String,
    }

    // ====== Encounter types for the encounter loop (Step 7) ======

    /// Represents the state of a single encounter session.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub struct EncounterState {
        pub phase: EncounterPhase,
    }

    /// Phases of an encounter (Step 7 state machine)
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
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

    use super::types::{CardDef, CombatAction, CombatSnapshot, EffectTarget};
    use rand::RngCore;
    use rand_pcg::Lcg64Xsh32;
    use std::collections::HashMap;

    /// Resolve a single combat action (card play) deterministically.
    ///
    /// Looks up the card definition, applies its effects as token operations,
    /// checks victory/defeat, and advances the turn.
    /// Returns an error if the card_id is unknown.
    pub fn resolve_combat_tick(
        current_state: &CombatSnapshot,
        action: &CombatAction,
        card_defs: &HashMap<u64, CardDef>,
        rng: &mut Lcg64Xsh32,
    ) -> Result<(CombatSnapshot, Vec<u64>), String> {
        let card = card_defs
            .get(&action.card_id)
            .ok_or_else(|| format!("Unknown card_id: {}", action.card_id))?;

        let mut rng_values = Vec::new();
        let mut state_after = current_state.clone();

        let rng_val = rng.next_u64();
        rng_values.push(rng_val);

        for effect in &card.effects {
            let (actor_tokens, target_tokens) = match (&effect.target, action.is_player) {
                (EffectTarget::OnSelf, true) | (EffectTarget::OnOpponent, false) => {
                    // Affect player tokens
                    (&mut state_after.player_tokens, None)
                }
                (EffectTarget::OnOpponent, true) | (EffectTarget::OnSelf, false) => {
                    // Affect enemy tokens
                    (
                        &mut state_after.enemy.active_tokens,
                        None::<&mut HashMap<String, i64>>,
                    )
                }
            };
            let _ = target_tokens; // suppress unused warning
            let entry = actor_tokens.entry(effect.token_id.clone()).or_insert(0);
            *entry = (*entry + effect.amount).max(0);

            if effect.token_id == "health" && *entry == 0 {
                state_after.is_finished = true;
                // Determine winner: if player health hit 0, enemy wins; otherwise player wins
                let affected_is_player = matches!(
                    (&effect.target, action.is_player),
                    (EffectTarget::OnSelf, true) | (EffectTarget::OnOpponent, false)
                );
                state_after.winner = Some(if affected_is_player {
                    "enemy".to_string()
                } else {
                    "player".to_string()
                });
            }
        }

        if !state_after.is_finished {
            state_after.player_turn = !state_after.player_turn;
        }

        Ok((state_after, rng_values))
    }

    /// Simulate a full combat encounter from a seed and initial state.
    ///
    /// Returns the final combat snapshot. Pure-data; no side effects.
    pub fn simulate_combat(
        initial_state: CombatSnapshot,
        seed: u64,
        actions: Vec<CombatAction>,
        card_defs: &HashMap<u64, CardDef>,
    ) -> CombatSnapshot {
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

        for action in &actions {
            match resolve_combat_tick(&current_state, action, card_defs, &mut rng) {
                Ok((next_state, _rng_vals)) => {
                    current_state = next_state;
                    if current_state.is_finished {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        current_state
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
    use super::types::{TokenLifecycle, TokenType};
    use std::collections::HashMap;

    #[derive(Debug, Default, Clone)]
    pub struct TokenRegistry {
        pub tokens: HashMap<String, TokenType>,
    }

    impl TokenRegistry {
        pub fn new() -> Self {
            Self {
                tokens: HashMap::new(),
            }
        }
        pub fn register(&mut self, token: TokenType) {
            self.tokens.insert(token.id.clone(), token);
        }

        /// Create a minimal canonical token registry seeded from vision.md
        pub fn with_canonical() -> Self {
            use TokenLifecycle::*;
            let mut r = Self::new();
            r.register(TokenType {
                id: "Insight".into(),
                lifecycle: PersistentCounter,
                cap: Some(9999),
            });
            r.register(TokenType {
                id: "Renown".into(),
                lifecycle: PersistentCounter,
                cap: Some(9999),
            });
            r.register(TokenType {
                id: "Refinement".into(),
                lifecycle: PersistentCounter,
                cap: Some(9999),
            });
            r.register(TokenType {
                id: "Stability".into(),
                lifecycle: PersistentCounter,
                cap: Some(9999),
            });
            r.register(TokenType {
                id: "Foresight".into(),
                lifecycle: PersistentCounter,
                cap: Some(9999),
            });
            r.register(TokenType {
                id: "Momentum".into(),
                lifecycle: PersistentCounter,
                cap: Some(9999),
            });
            r.register(TokenType {
                id: "Corruption".into(),
                lifecycle: PersistentCounter,
                cap: Some(9999),
            });
            r.register(TokenType {
                id: "Exhaustion".into(),
                lifecycle: PersistentCounter,
                cap: Some(9999),
            });
            r.register(TokenType {
                id: "Durability".into(),
                lifecycle: PersistentCounter,
                cap: Some(9999),
            });
            // Combat tokens — expire when the encounter reaches Scouting phase
            let combat_lifecycle = FixedTypeDuration {
                duration: 1,
                phases: vec![super::types::EncounterPhase::Scouting],
            };
            r.register(TokenType {
                id: "health".into(),
                lifecycle: combat_lifecycle.clone(),
                cap: Some(9999),
            });
            r.register(TokenType {
                id: "max_health".into(),
                lifecycle: combat_lifecycle.clone(),
                cap: Some(9999),
            });
            r.register(TokenType {
                id: "dodge".into(),
                lifecycle: combat_lifecycle.clone(),
                cap: Some(9999),
            });
            r.register(TokenType {
                id: "shield".into(),
                lifecycle: combat_lifecycle.clone(),
                cap: Some(9999),
            });
            r.register(TokenType {
                id: "stamina".into(),
                lifecycle: combat_lifecycle.clone(),
                cap: Some(9999),
            });
            r.register(TokenType {
                id: "mana".into(),
                lifecycle: combat_lifecycle,
                cap: Some(9999),
            });
            r
        }

        pub fn contains(&self, id: &str) -> bool {
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
use types::{ActionEntry, ActionPayload, CardCounts, CardEffect, CardKind, LibraryCard};

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
}

/// Build the initial Library with starter cards.
fn initialize_library() -> Library {
    let mut lib = Library::new();

    // Attack card (id 0): deals 5 damage to opponent (~25% of starting deck)
    lib.add_card(
        CardKind::Attack {
            effects: vec![CardEffect {
                target: types::EffectTarget::OnOpponent,
                token_id: "health".to_string(),
                amount: -5,
            }],
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
    );

    // Defence card (id 1): grants 3 shield to self (~25% of starting deck)
    lib.add_card(
        CardKind::Defence {
            effects: vec![CardEffect {
                target: types::EffectTarget::OnSelf,
                token_id: "shield".to_string(),
                amount: 3,
            }],
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
    );

    // Resource card (id 2): grants 2 stamina to self, draws 1 card (~50% of starting deck)
    lib.add_card(
        CardKind::Resource {
            effects: vec![CardEffect {
                target: types::EffectTarget::OnSelf,
                token_id: "stamina".to_string(),
                amount: 2,
            }],
            draw_count: 1,
        },
        CardCounts {
            library: 0,
            deck: 35,
            hand: 5,
            discard: 0,
        },
    );

    // Combat encounter: Gnome (id 3)
    lib.add_card(
        CardKind::CombatEncounter {
            combatant_def: types::CombatantDef {
                initial_tokens: HashMap::from([
                    ("health".to_string(), 20),
                    ("max_health".to_string(), 20),
                ]),
                attack_deck: vec![types::EnemyCardDef {
                    effects: vec![CardEffect {
                        target: types::EffectTarget::OnOpponent,
                        token_id: "health".to_string(),
                        amount: -3,
                    }],
                }],
                defence_deck: vec![types::EnemyCardDef {
                    effects: vec![CardEffect {
                        target: types::EffectTarget::OnSelf,
                        token_id: "shield".to_string(),
                        amount: 2,
                    }],
                }],
                resource_deck: vec![types::EnemyCardDef {
                    effects: vec![CardEffect {
                        target: types::EffectTarget::OnSelf,
                        token_id: "stamina".to_string(),
                        amount: 1,
                    }],
                }],
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    lib
}

/// Apply card effects to a combat snapshot, handling dodge absorption.
fn apply_card_effects(effects: &[CardEffect], is_player: bool, combat: &mut types::CombatSnapshot) {
    for effect in effects {
        let target_tokens = match (&effect.target, is_player) {
            (types::EffectTarget::OnSelf, true) | (types::EffectTarget::OnOpponent, false) => {
                &mut combat.player_tokens
            }
            (types::EffectTarget::OnOpponent, true) | (types::EffectTarget::OnSelf, false) => {
                &mut combat.enemy.active_tokens
            }
        };

        if effect.token_id == "health" && effect.amount < 0 {
            // Damage: consume dodge first, then reduce health
            let damage = -effect.amount;
            let dodge = target_tokens.get("dodge").copied().unwrap_or(0);
            let absorbed = dodge.min(damage);
            target_tokens.insert("dodge".to_string(), (dodge - absorbed).max(0));
            let remaining_damage = damage - absorbed;
            if remaining_damage > 0 {
                let health = target_tokens.entry("health".to_string()).or_insert(0);
                *health = (*health - remaining_damage).max(0);
            }
        } else {
            let entry = target_tokens.entry(effect.token_id.clone()).or_insert(0);
            *entry = (*entry + effect.amount).max(0);
        }
    }
}

/// Check if combat has ended (either side at 0 health).
fn check_combat_end(combat: &mut types::CombatSnapshot) {
    let player_health = combat.player_tokens.get("health").copied().unwrap_or(0);
    let enemy_health = combat
        .enemy
        .active_tokens
        .get("health")
        .copied()
        .unwrap_or(0);

    if enemy_health <= 0 || player_health <= 0 {
        combat.is_finished = true;
        combat.winner = Some(if enemy_health <= 0 && player_health > 0 {
            "Player".to_string()
        } else if player_health <= 0 && enemy_health > 0 {
            "Enemy".to_string()
        } else {
            "Draw".to_string()
        });
    }
}

/// Minimal in-memory game state driven by the library's mutator API.
#[derive(Debug, Clone)]
pub struct GameState {
    pub registry: TokenRegistry,
    pub action_log: std::sync::Arc<ActionLog>,
    pub token_balances: HashMap<String, i64>,
    pub library: Library,
    pub current_combat: Option<types::CombatSnapshot>,
    pub encounter_state: types::EncounterState,
    pub last_combat_result: Option<types::CombatResult>,
}

impl GameState {
    /// Create a new game state seeded with the canonical token registry
    pub fn new() -> Self {
        let registry = TokenRegistry::with_canonical();
        let mut balances = HashMap::new();
        for id in registry.tokens.keys() {
            balances.insert(id.clone(), 0i64);
        }
        // Default Foresight controls area deck hand size
        balances.insert("Foresight".to_string(), 3);
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
    pub fn apply_grant(
        &mut self,
        token_id: &str,
        amount: i64,
        reason: Option<String>,
    ) -> Result<ActionEntry, String> {
        let token_type = self
            .registry
            .tokens
            .get(token_id)
            .ok_or_else(|| format!("Unknown token '{}'", token_id))?;

        // Check cap if present
        if let Some(cap) = token_type.cap {
            let current = self.token_balances.get(token_id).copied().unwrap_or(0);
            if current + amount > cap as i64 {
                return Err(format!(
                    "Token '{}' would exceed cap of {} (current: {})",
                    token_id, cap, current
                ));
            }
        }

        let current = self.token_balances.get(token_id).copied().unwrap_or(0);
        let resulting_amount = current + amount;
        let payload = ActionPayload::GrantToken {
            token_id: token_id.to_string(),
            amount,
            reason,
            resulting_amount,
        };
        let entry = self.append_action("GrantToken", payload);
        let v = self.token_balances.entry(token_id.to_string()).or_insert(0);
        *v += amount;
        Ok(entry)
    }

    /// Apply a ConsumeToken action: deduct from balances and append to the action log.
    pub fn apply_consume(
        &mut self,
        token_id: &str,
        amount: i64,
        reason: Option<String>,
    ) -> Result<ActionEntry, String> {
        if !self.registry.contains(token_id) {
            return Err(format!("Unknown token '{}'", token_id));
        }

        let current = self.token_balances.get(token_id).copied().unwrap_or(0);
        if current < amount {
            return Err(format!(
                "Cannot consume {} of token '{}': insufficient balance (have {})",
                amount, token_id, current
            ));
        }

        let resulting_amount = current - amount;
        let payload = ActionPayload::ConsumeToken {
            token_id: token_id.to_string(),
            amount,
            reason,
            resulting_amount,
        };
        let entry = self.append_action("ConsumeToken", payload);
        let v = self.token_balances.entry(token_id.to_string()).or_insert(0);
        *v -= amount;
        Ok(entry)
    }

    /// Append an action to the action log with optional metadata; returns the appended entry.
    pub fn append_action(&self, action_type: &str, payload: ActionPayload) -> ActionEntry {
        self.action_log.append(action_type, payload)
    }

    /// Initialize combat from a Library CombatEncounter card.
    pub fn start_combat(&mut self, encounter_card_id: usize) -> Result<(), String> {
        let lib_card = self
            .library
            .get(encounter_card_id)
            .ok_or_else(|| format!("Card {} not found in Library", encounter_card_id))?
            .clone();
        let combatant_def = match &lib_card.kind {
            CardKind::CombatEncounter { combatant_def } => combatant_def.clone(),
            _ => {
                return Err(format!(
                    "Card {} is not a CombatEncounter",
                    encounter_card_id
                ))
            }
        };
        // Initialize player combat tokens from token_balances
        let player_tokens = self.token_balances.clone();
        let snapshot = types::CombatSnapshot {
            round: 1,
            player_turn: true,
            phase: types::CombatPhase::Defending,
            player_tokens,
            enemy: types::Combatant {
                active_tokens: combatant_def.initial_tokens.clone(),
            },
            encounter_card_id: Some(encounter_card_id),
            is_finished: false,
            winner: None,
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
        let (effects, draw_count) = match &lib_card.kind {
            CardKind::Attack { effects } | CardKind::Defence { effects } => (effects.clone(), 0),
            CardKind::Resource {
                effects,
                draw_count,
            } => (effects.clone(), *draw_count),
            _ => return Err("Cannot play a non-action card".to_string()),
        };
        apply_card_effects(&effects, true, combat);
        check_combat_end(combat);
        if combat.is_finished {
            let winner = combat.winner.clone().unwrap_or_default();
            self.last_combat_result = Some(types::CombatResult {
                winner: winner.clone(),
            });
            self.current_combat = None;
            self.encounter_state.phase = types::EncounterPhase::Scouting;
        }
        // Resource cards trigger draws from deck → hand
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
                c.counts.deck > 0 && !matches!(c.kind, CardKind::CombatEncounter { .. })
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

    /// Resolve an enemy card play (random card from appropriate deck).
    pub fn resolve_enemy_play(&mut self, rng: &mut rand_pcg::Lcg64Xsh32) -> Result<(), String> {
        let combat = self.current_combat.as_ref().ok_or("No active combat")?;
        let encounter_card_id = combat
            .encounter_card_id
            .ok_or("No encounter card in combat")?;
        let phase = combat.phase.clone();

        let lib_card = self
            .library
            .get(encounter_card_id)
            .ok_or("Encounter card not found")?
            .clone();
        let combatant_def = match &lib_card.kind {
            CardKind::CombatEncounter { combatant_def } => combatant_def,
            _ => return Err("Not a CombatEncounter".to_string()),
        };

        let deck = match phase {
            types::CombatPhase::Defending => &combatant_def.defence_deck,
            types::CombatPhase::Attacking => &combatant_def.attack_deck,
            types::CombatPhase::Resourcing => &combatant_def.resource_deck,
        };

        if deck.is_empty() {
            return Ok(());
        }

        use rand::RngCore;
        let pick = (rng.next_u64() as usize) % deck.len();
        let enemy_card = &deck[pick];
        let effects = enemy_card.effects.clone();

        let combat = self.current_combat.as_mut().ok_or("No active combat")?;
        apply_card_effects(&effects, false, combat);
        check_combat_end(combat);
        if combat.is_finished {
            let winner = combat.winner.clone().unwrap_or_default();
            self.last_combat_result = Some(types::CombatResult {
                winner: winner.clone(),
            });
            self.current_combat = None;
            self.encounter_state.phase = types::EncounterPhase::Scouting;
        }
        Ok(())
    }

    /// Advance combat phase to next (Defending → Attacking → Resourcing → Defending).
    pub fn advance_combat_phase(&mut self) -> Result<(), String> {
        let combat = self.current_combat.as_mut().ok_or("No active combat")?;
        combat.phase = combat.phase.next();
        Ok(())
    }

    /// Reconstruct state from a registry and an existing action log (seed not modelled here).
    pub fn replay_from_log(registry: TokenRegistry, log: &ActionLog) -> Self {
        let mut gs = {
            let mut balances = HashMap::new();
            for id in registry.tokens.keys() {
                balances.insert(id.clone(), 0i64);
            }
            // Default Foresight controls area deck hand size
            balances.insert("Foresight".to_string(), 3);
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
        for e in log.entries() {
            match &e.payload {
                ActionPayload::GrantToken {
                    token_id, amount, ..
                } => {
                    let v = gs.token_balances.entry(token_id.to_string()).or_insert(0);
                    *v += *amount;
                }
                ActionPayload::ConsumeToken {
                    token_id, amount, ..
                } => {
                    let v = gs.token_balances.entry(token_id.to_string()).or_insert(0);
                    *v -= *amount;
                }
                ActionPayload::ExpireToken {
                    token_id, amount, ..
                } => {
                    let v = gs.token_balances.entry(token_id.to_string()).or_insert(0);
                    *v = (*v - *amount).max(0);
                }
                ActionPayload::SetSeed { .. } => {
                    // SetSeed is recorded but not applied to token balances during replay here
                }
                _ => {
                    // Other action payloads (RngDraw, RngSnapshot, PlayCard, etc.) are recorded for audit but don't affect token balances
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

/// Get canonical token registry
#[openapi]
#[get("/tokens")]
pub async fn list_library_tokens() -> Json<Vec<String>> {
    let reg = TokenRegistry::with_canonical();
    Json(reg.tokens.keys().cloned().collect())
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
