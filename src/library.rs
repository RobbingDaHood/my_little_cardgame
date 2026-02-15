// Domain library skeleton
// NOTE: The "Library" here is a domain entity (the canonical collection of CardDef objects and token registry)
// and not necessarily a separate Rust crate. This module is a minimal starting point for the refactor.

//! Minimal domain skeleton for Decks, Tokens, Library and ActionLog
//!
//! This file provides small, well-scoped domain primitives used by higher-level systems.

use std::collections::HashMap;

pub mod types {
    /// Canonical card definition (minimal)
    #[derive(Debug, Clone)]
    pub struct CardDef {
        pub id: u64,
        pub name: String,
        pub card_type: String,
    }

    /// Token type metadata and lifecycle
    #[derive(Debug, Clone)]
    pub struct TokenType {
        pub id: String,
        pub lifecycle: TokenLifecycle,
        pub cap: Option<u64>,
    }

    #[derive(Debug, Clone)]
    pub enum TokenLifecycle {
        Permanent,
        PersistentCounter,
        FixedDuration(u64),
        FixedTypeDuration(u64),
        UntilNextAction,
        SingleUse,
        Conditional,
    }

    #[derive(Debug, Clone)]
    pub struct Deck {
        pub id: String,
        pub card_ids: Vec<u64>,
    }

    /// Small, explicit action requests used by the library mutator.
    #[derive(Debug, Clone)]
    pub enum ActionRequest {
        GrantToken { token_id: String, amount: i64 },
    }

    /// Stored action entry in the append-only action log.
    #[derive(Debug, Clone)]
    pub struct ActionEntry {
        pub seq: u64,
        pub action_type: String,
        pub payload: String, // simple pipe-separated payload for now
    }
}

pub mod registry {
    use super::types::{TokenLifecycle, TokenType};

    #[derive(Debug, Default, Clone)]
    pub struct TokenRegistry {
        pub tokens: Vec<TokenType>,
    }

    impl TokenRegistry {
        pub fn new() -> Self { Self { tokens: Vec::new() } }
        pub fn register(&mut self, token: TokenType) { self.tokens.push(token); }

        /// Create a minimal canonical token registry seeded from vision.md
        pub fn with_canonical() -> Self {
            use TokenLifecycle::*;
            let mut r = Self::new();
            r.register(TokenType { id: "Insight".into(), lifecycle: PersistentCounter, cap: Some(9999) });
            r.register(TokenType { id: "Renown".into(), lifecycle: PersistentCounter, cap: Some(9999) });
            r.register(TokenType { id: "Refinement".into(), lifecycle: PersistentCounter, cap: Some(9999) });
            r.register(TokenType { id: "Stability".into(), lifecycle: PersistentCounter, cap: Some(9999) });
            r.register(TokenType { id: "Foresight".into(), lifecycle: PersistentCounter, cap: Some(9999) });
            r.register(TokenType { id: "Momentum".into(), lifecycle: PersistentCounter, cap: Some(9999) });
            r.register(TokenType { id: "Corruption".into(), lifecycle: PersistentCounter, cap: Some(9999) });
            r.register(TokenType { id: "Exhaustion".into(), lifecycle: PersistentCounter, cap: Some(9999) });
            r.register(TokenType { id: "Durability".into(), lifecycle: PersistentCounter, cap: Some(9999) });
            r
        }

        pub fn contains(&self, id: &str) -> bool {
            self.tokens.iter().any(|t| t.id == id)
        }
    }
}

pub mod action_log {
    use super::types::ActionEntry;

    #[derive(Debug, Default, Clone)]
    pub struct ActionLog {
        pub entries: Vec<ActionEntry>,
    }

    impl ActionLog {
        pub fn new() -> Self { Self { entries: Vec::new() } }

        /// Append an action entry, assigning an incrementing sequence number.
        pub fn append(&mut self, action_type: &str, payload: &str) -> ActionEntry {
            let seq = self.entries.last().map(|e| e.seq).unwrap_or(0) + 1;
            let entry = ActionEntry { seq, action_type: action_type.to_string(), payload: payload.to_string() };
            self.entries.push(entry.clone());
            entry
        }

        pub fn entries(&self) -> &[ActionEntry] { &self.entries }
    }
}

use types::{ActionEntry};
use registry::TokenRegistry;
use action_log::ActionLog;

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
        for t in &registry.tokens {
            balances.insert(t.id.clone(), 0i64);
        }
        Self { registry, action_log: ActionLog::new(), token_balances: balances }
    }

    /// Apply a simple GrantToken action: update balances and append to the action log.
    pub fn apply_grant(&mut self, token_id: &str, amount: i64) -> Result<ActionEntry, String> {
        if !self.registry.contains(token_id) {
            return Err(format!("Unknown token '{}'", token_id));
        }
        // simple pipe-separated payload: type|token_id|amount
        let payload = format!("{}|{}|{}", "GrantToken", token_id, amount);
        let entry = self.action_log.append("GrantToken", &payload);
        let v = self.token_balances.entry(token_id.to_string()).or_insert(0);
        *v += amount;
        Ok(entry)
    }

    /// Reconstruct state from a registry and an existing action log (seed not modelled here).
    pub fn replay_from_log(registry: TokenRegistry, log: &ActionLog) -> Self {
        let mut gs = {
            let mut balances = HashMap::new();
            for t in &registry.tokens {
                balances.insert(t.id.clone(), 0i64);
            }
            Self { registry, action_log: ActionLog::new(), token_balances: balances }
        };
        for e in &log.entries {
            if e.action_type == "GrantToken" {
                let parts: Vec<&str> = e.payload.split('|').collect();
                if parts.len() == 3 && parts[0] == "GrantToken" {
                    let token_id = parts[1].to_string();
                    if let Ok(amount) = parts[2].parse::<i64>() {
                        let v = gs.token_balances.entry(token_id.clone()).or_insert(0);
                        *v += amount;
                        gs.action_log.entries.push(e.clone());
                    }
                }
            }
        }
        gs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grant_and_replay() {
        let mut gs = GameState::new();
        assert_eq!(gs.token_balances.get("Insight").copied().unwrap_or(0), 0);
        let entry = gs.apply_grant("Insight", 10).expect("apply_grant failed");
        assert_eq!(entry.seq, 1);
        assert_eq!(gs.token_balances.get("Insight").copied().unwrap_or(0), 10);

        // replay
        let replayed = GameState::replay_from_log(gs.registry.clone(), &gs.action_log);
        assert_eq!(replayed.token_balances.get("Insight").copied().unwrap_or(0), 10);
        assert_eq!(replayed.action_log.entries.len(), 1);
    }
}
