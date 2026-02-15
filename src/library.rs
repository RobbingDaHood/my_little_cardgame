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
    /// Canonical card definition (minimal)
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub struct CardDef {
        pub id: u64,
        pub name: String,
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
        GrantToken { token_id: String, amount: i64 },
    }

    /// Stored action entry in the append-only action log.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[serde(crate = "rocket::serde")]
    pub struct ActionEntry {
        pub seq: u64,
        pub action_type: String,
        pub payload: ActionPayload, // structured payload for replay
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
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Mutex;

    #[derive(Debug)]
    pub struct ActionLog {
        pub entries: Mutex<Vec<ActionEntry>>,
        pub seq: AtomicU64,
    }

    impl Clone for ActionLog {
        fn clone(&self) -> Self {
            let entries = self.entries.lock().unwrap().clone();
            let seq = self.seq.load(Ordering::SeqCst);
            ActionLog {
                entries: Mutex::new(entries),
                seq: AtomicU64::new(seq),
            }
        }
    }

    impl Default for ActionLog {
        fn default() -> Self {
            ActionLog {
                entries: Mutex::new(Vec::new()),
                seq: AtomicU64::new(0),
            }
        }
    }

    impl ActionLog {
        pub fn new() -> Self {
            Self::default()
        }

        /// Append an action entry, assigning an incrementing sequence number.
        pub fn append(&self, action_type: &str, payload: ActionPayload) -> ActionEntry {
            let seq = self.seq.fetch_add(1, Ordering::SeqCst) + 1;
            let entry = ActionEntry {
                seq,
                action_type: action_type.to_string(),
                payload: payload.clone(),
            };
            self.entries.lock().unwrap().push(entry.clone());
            entry
        }

        /// Return a cloned snapshot of entries for replay/inspection
        pub fn entries(&self) -> Vec<ActionEntry> {
            self.entries.lock().unwrap().clone()
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
    pub action_log: ActionLog,
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
        Self {
            registry,
            action_log: ActionLog::new(),
            token_balances: balances,
        }
    }

    /// Apply a simple GrantToken action: update balances and append to the action log.
    pub fn apply_grant(&mut self, token_id: &str, amount: i64) -> Result<ActionEntry, String> {
        if !self.registry.contains(token_id) {
            return Err(format!("Unknown token '{}'", token_id));
        }
        let payload = ActionPayload::GrantToken { token_id: token_id.to_string(), amount };
        let entry = self.action_log.append("GrantToken", payload);
        let v = self.token_balances.entry(token_id.to_string()).or_insert(0);
        *v += amount;
        Ok(entry)
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
                action_log: ActionLog::new(),
                token_balances: balances,
            }
        };
        for e in log.entries() {
            if let ActionPayload::GrantToken { token_id, amount } = &e.payload {
                let v = gs.token_balances.entry(token_id.to_string()).or_insert(0);
                *v += *amount;
                gs.action_log.entries.lock().unwrap().push(e.clone());
                let cur = gs.action_log.seq.load(Ordering::SeqCst);
                if cur < e.seq {
                    gs.action_log.seq.store(e.seq, Ordering::SeqCst);
                }
            }
        }
        gs
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn grant_and_replay() {
        let mut gs = GameState::new();
        assert_eq!(gs.token_balances.get("Insight").copied().unwrap_or(0), 0);
        let entry = gs.apply_grant("Insight", 10).expect("apply_grant failed");
        assert_eq!(entry.seq, 1);
        assert_eq!(gs.token_balances.get("Insight").copied().unwrap_or(0), 10);

        // replay
        let replayed = GameState::replay_from_log(gs.registry.clone(), &gs.action_log);
        assert_eq!(
            replayed.token_balances.get("Insight").copied().unwrap_or(0),
            10
        );
        assert_eq!(replayed.action_log.entries().len(), 1);
    }

    #[test]
    fn action_log_concurrent_append() {
        let log = Arc::new(action_log::ActionLog::new());
        let threads = 8usize;
        let per_thread = 100usize;
        let mut handles = Vec::new();
        for i in 0..threads {
            let log_clone = Arc::clone(&log);
            handles.push(thread::spawn(move || {
                for j in 0..per_thread {
                    let payload = ActionPayload::GrantToken { token_id: format!("t{}_{}", i, j), amount: j as i64 };
                    log_clone.append("GrantToken", payload);
                }
            }));
        }
        for h in handles {
            h.join().expect("thread panicked");
        }
        let entries = log.entries();
        assert_eq!(entries.len(), threads * per_thread);
        // Ensure sequence numbers are unique and cover 1..N
        let mut seqs: Vec<u64> = entries.iter().map(|e| e.seq).collect();
        seqs.sort();
        for (idx, seq) in seqs.iter().enumerate() {
            assert_eq!(*seq as usize, idx + 1);
        }
    }
}

/// Expose a thin HTTP/OKAPI-friendly endpoint that returns canonical token ids
#[openapi]
#[get("/library/tokens")]
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
                name: format!("card_{}", c.id),
                card_type: ct,
            }
        })
        .collect();
    Json(items)
}
