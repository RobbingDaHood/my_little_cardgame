// Domain library skeleton
// NOTE: The "Library" here is a domain entity (the canonical collection of CardDef objects and token registry)
// and not necessarily a separate Rust crate. This module is a minimal starting point for the refactor.

//! Minimal domain skeleton for Decks, Tokens, Library and ActionLog
//!
//! This file is intentionally small: it provides types and tiny helpers that will be expanded
//! during the refactor described in the plan.

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

    #[derive(Debug, Clone)]
    pub struct ActionEntry {
        pub seq: u64,
        pub action_type: String,
        pub payload: String,
    }
}

pub mod registry {
    use super::types::TokenType;

    #[derive(Debug, Default)]
    pub struct TokenRegistry {
        pub tokens: Vec<TokenType>,
    }

    impl TokenRegistry {
        pub fn new() -> Self { Self { tokens: Vec::new() } }
        pub fn register(&mut self, token: TokenType) { self.tokens.push(token); }
    }
}

pub mod action_log {
    use super::types::ActionEntry;

    #[derive(Debug, Default)]
    pub struct ActionLog {
        pub entries: Vec<ActionEntry>,
    }

    impl ActionLog {
        pub fn new() -> Self { Self { entries: Vec::new() } }
        pub fn append(&mut self, entry: ActionEntry) { self.entries.push(entry); }
        pub fn entries(&self) -> &[ActionEntry] { &self.entries }
    }
}
