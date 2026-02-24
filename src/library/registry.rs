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
