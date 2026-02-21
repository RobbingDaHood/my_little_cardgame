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
        FixedDuration(u64),
        FixedTypeDuration(u64),
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
        CombatEvent {
            round: u64,
            action_type: String, // "DealDamage", "GrantToken", etc.
            source: Option<String>,
            target: Option<String>,
            details: serde_json::Value, // flexible details for different event types
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
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub struct Combatant {
        pub id: String, // unique identifier (e.g., "player", "enemy_0")
        pub current_hp: i64,
        pub max_hp: i64,
        pub active_tokens: HashMap<String, i64>, // token_id -> count
    }

    /// Represents a single action taken during combat.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde", tag = "action_type")]
    pub enum CombatAction {
        PlayCard {
            combatant_id: String,
            card_id: u64,
            effects: Vec<String>, // effect descriptions
        },
        DealDamage {
            source: String,
            target: String,
            amount: i64,
        },
        GrantToken {
            combatant_id: String,
            token_id: String,
            amount: i64,
        },
        ConsumeToken {
            combatant_id: String,
            token_id: String,
            amount: i64,
        },
        DrawCard {
            combatant_id: String,
            card_id: u64,
        },
    }

    /// Represents the overall state of combat at any point.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub struct CombatState {
        pub round: u64,
        pub current_turn: String, // "player" or specific combatant id
        pub combatants: Vec<Combatant>,
        pub is_finished: bool,
        pub winner: Option<String>, // None if ongoing, Some(id) if finished
    }

    /// Represents a single event that occurred during combat resolution.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub struct CombatLogEntry {
        pub step: u64, // sequence number within combat
        pub action: CombatAction,
        pub state_before: CombatState,
        pub state_after: CombatState,
        pub rng_values: Vec<u64>, // RNG values consumed by this step
    }

    /// Complete deterministic log of a combat encounter.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub struct CombatLog {
        pub seed: u64,
        pub initial_state: CombatState,
        pub entries: Vec<CombatLogEntry>,
        pub final_state: CombatState,
    }

    // ====== Encounter types for the encounter loop (Step 7) ======

    /// Represents the state of a single encounter session.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub struct EncounterState {
        pub encounter_id: String,
        pub area_id: String,
        pub combat_state: CombatState,
        pub phase: EncounterPhase,
        pub scouting_parameters: ScoutingParameters,
    }

    /// Phases of an encounter (Step 7 state machine)
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
    #[serde(crate = "rocket::serde")]
    pub enum EncounterPhase {
        /// Encounter has been drawn; player can start combat
        Ready,
        /// Combat is currently active
        InCombat,
        /// Combat has finished; post-encounter scouting is available
        PostEncounter,
        /// Encounter is finished (resolved or abandoned)
        Finished,
    }

    /// Parameters controlling how replacement encounters are generated (Step 7)
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub struct ScoutingParameters {
        pub preview_count: u64, // How many cards can be peeked ahead
        pub affix_bias: String, // Bias towards certain affixes
        pub pool_modifier: f64, // Modifier on encounter generation pool
    }

    /// User actions during an encounter (Step 7)
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde", tag = "action_type")]
    pub enum EncounterAction {
        /// Pick an encounter from the area deck and initialize combat
        PickEncounter { area_id: String },
        /// Play a card during combat (moves turn forward)
        PlayCard { card_id: u64, effects: Vec<String> },
        /// Make a scouting choice post-encounter
        ApplyScouting {
            choice_id: String,
            parameters: ScoutingParameters,
        },
        /// Finish/abandon the encounter
        FinishEncounter,
    }
}

pub mod combat {
    //! Deterministic, pure-data combat resolution (Step 6)
    //!
    //! This module provides pure functions for resolving combat deterministically
    //! using seeded RNG. All state changes are captured in a CombatLog for replay.

    use super::types::{CombatAction, CombatLog, CombatLogEntry, CombatState};
    use rand::RngCore;
    use rand_pcg::Lcg64Xsh32;

    /// Resolve a single tick (one action) of combat deterministically.
    ///
    /// Takes current combat state and RNG, applies one action, and returns
    /// the updated state plus the log entry recording the change.
    pub fn resolve_combat_tick(
        current_state: &CombatState,
        action: CombatAction,
        rng: &mut Lcg64Xsh32,
    ) -> (CombatState, CombatLogEntry, Vec<u64>) {
        let mut rng_values = Vec::new();
        let state_before = current_state.clone();

        // Apply action to produce new state
        let mut state_after = current_state.clone();

        match &action {
            CombatAction::DealDamage {
                source,
                target,
                amount,
            } => {
                // Find target combatant and reduce HP
                if let Some(target_combatant) =
                    state_after.combatants.iter_mut().find(|c| &c.id == target)
                {
                    target_combatant.current_hp = (target_combatant.current_hp - amount).max(0);

                    // Check if target is defeated
                    if target_combatant.current_hp == 0 {
                        state_after.is_finished = true;
                        state_after.winner = Some(source.clone());
                    }
                }
            }
            CombatAction::GrantToken {
                combatant_id,
                token_id,
                amount,
            } => {
                if let Some(combatant) = state_after
                    .combatants
                    .iter_mut()
                    .find(|c| &c.id == combatant_id)
                {
                    let entry = combatant.active_tokens.entry(token_id.clone()).or_insert(0);
                    *entry += amount;
                }
            }
            CombatAction::ConsumeToken {
                combatant_id,
                token_id,
                amount,
            } => {
                if let Some(combatant) = state_after
                    .combatants
                    .iter_mut()
                    .find(|c| &c.id == combatant_id)
                {
                    if let Some(entry) = combatant.active_tokens.get_mut(token_id) {
                        *entry = (*entry - amount).max(0);
                    }
                }
            }
            CombatAction::PlayCard {
                combatant_id: _,
                card_id: _,
                effects: _,
            } => {
                // Card play typically triggers effects; we'd apply them here
                // For now, just record the draw
                let rng_val = rng.next_u64();
                rng_values.push(rng_val);
            }
            CombatAction::DrawCard {
                combatant_id: _,
                card_id: _,
            } => {
                let rng_val = rng.next_u64();
                rng_values.push(rng_val);
            }
        }

        // Advance turn if combat not finished
        if !state_after.is_finished {
            state_after.current_turn = if state_after.current_turn == "player" {
                "enemy_0".to_string()
            } else {
                "player".to_string()
            };
        }

        let entry = CombatLogEntry {
            step: 0, // Will be set by caller based on sequence
            action,
            state_before,
            state_after: state_after.clone(),
            rng_values: rng_values.clone(),
        };

        (state_after, entry, rng_values)
    }

    /// Simulate a full combat encounter from a seed and initial state.
    ///
    /// Returns the deterministic log of the entire combat.
    /// This is pure-data; no side effects on game state.
    pub fn simulate_combat(
        initial_state: CombatState,
        seed: u64,
        actions: Vec<CombatAction>,
    ) -> CombatLog {
        use rand::SeedableRng;

        let seed_bytes: [u8; 16] = {
            let s = seed.to_le_bytes();
            let mut bytes = [0u8; 16];
            bytes[0..8].copy_from_slice(&s);
            bytes[8..16].copy_from_slice(&s);
            bytes
        };
        let mut rng = Lcg64Xsh32::from_seed(seed_bytes);

        let mut current_state = initial_state.clone();
        let mut entries = Vec::new();
        let mut all_rng_values = Vec::new();

        for action in actions {
            let (next_state, mut entry, rng_vals) =
                resolve_combat_tick(&current_state, action, &mut rng);
            entry.step = entries.len() as u64;
            all_rng_values.extend(rng_vals);
            entries.push(entry);
            current_state = next_state;

            // Stop if combat finished
            if current_state.is_finished {
                break;
            }
        }

        CombatLog {
            seed,
            initial_state: initial_state.clone(),
            entries,
            final_state: current_state,
        }
    }

    /// Convert a CombatAction to an ActionPayload for logging.
    pub fn action_to_payload(round: u64, action: &CombatAction) -> super::types::ActionPayload {
        match action {
            CombatAction::DealDamage {
                source,
                target,
                amount,
            } => super::types::ActionPayload::CombatEvent {
                round,
                action_type: "DealDamage".to_string(),
                source: Some(source.clone()),
                target: Some(target.clone()),
                details: serde_json::json!({ "amount": amount }),
            },
            CombatAction::GrantToken {
                combatant_id,
                token_id,
                amount,
            } => super::types::ActionPayload::CombatEvent {
                round,
                action_type: "GrantToken".to_string(),
                source: Some(combatant_id.clone()),
                target: None,
                details: serde_json::json!({ "token_id": token_id, "amount": amount }),
            },
            CombatAction::ConsumeToken {
                combatant_id,
                token_id,
                amount,
            } => super::types::ActionPayload::CombatEvent {
                round,
                action_type: "ConsumeToken".to_string(),
                source: Some(combatant_id.clone()),
                target: None,
                details: serde_json::json!({ "token_id": token_id, "amount": amount }),
            },
            CombatAction::PlayCard {
                combatant_id,
                card_id,
                effects: _,
            } => super::types::ActionPayload::CombatEvent {
                round,
                action_type: "PlayCard".to_string(),
                source: Some(combatant_id.clone()),
                target: None,
                details: serde_json::json!({ "card_id": card_id }),
            },
            CombatAction::DrawCard {
                combatant_id,
                card_id,
            } => super::types::ActionPayload::CombatEvent {
                round,
                action_type: "DrawCard".to_string(),
                source: Some(combatant_id.clone()),
                target: None,
                details: serde_json::json!({ "card_id": card_id }),
            },
        }
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
use types::{ActionEntry, ActionPayload};

/// Minimal in-memory game state driven by the library's mutator API.
#[derive(Debug, Clone)]
pub struct GameState {
    pub registry: TokenRegistry,
    pub action_log: std::sync::Arc<ActionLog>,
    pub token_balances: HashMap<String, i64>,
}

impl GameState {
    /// Create a new game state seeded with the canonical token registry
    pub fn new() -> Self {
        let registry = TokenRegistry::with_canonical();
        let mut balances = HashMap::new();
        for id in registry.tokens.keys() {
            balances.insert(id.clone(), 0i64);
        }
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

    /// Reconstruct state from a registry and an existing action log (seed not modelled here).
    pub fn replay_from_log(registry: TokenRegistry, log: &ActionLog) -> Self {
        let mut gs = {
            let mut balances = HashMap::new();
            for id in registry.tokens.keys() {
                balances.insert(id.clone(), 0i64);
            }
            Self {
                registry,
                action_log: std::sync::Arc::new(ActionLog::new()),
                token_balances: balances,
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

/// Minimal "library" view exposing canonical card entries derived from current player_data.
#[openapi]
#[get("/library/cards")]
pub async fn list_library_cards(
    player_data: &rocket::State<crate::player_data::PlayerData>,
) -> Json<Vec<types::CardDef>> {
    let cards = player_data.cards.lock().await.clone();
    let items: Vec<types::CardDef> = cards
        .into_iter()
        .map(|c| {
            let ct = match c.card_type {
                crate::deck::card::CardType::Attack => "Attack".to_string(),
                crate::deck::card::CardType::Defence => "Defence".to_string(),
                crate::deck::card::CardType::Resource => "Resource".to_string(),
            };
            types::CardDef {
                id: c.id as u64,
                card_type: ct,
            }
        })
        .collect();
    Json(items)
}
