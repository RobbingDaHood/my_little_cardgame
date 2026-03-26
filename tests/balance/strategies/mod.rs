use serde_json::Value;

/// Snapshot of public game state gathered from API endpoints.
/// All data comes from GET requests — no internal state access.
pub struct GameSnapshot {
    /// Current encounter state from GET /encounter (None if no active encounter)
    pub encounter: Option<Value>,
    /// Player token balances from GET /player/tokens
    pub tokens: Value,
}

impl GameSnapshot {
    pub fn player_health(&self) -> i64 {
        extract_token_value(&self.tokens, "Health")
    }

    pub fn player_stamina(&self) -> i64 {
        extract_token_value(&self.tokens, "Stamina")
    }

    pub fn player_deaths(&self) -> i64 {
        extract_token_value(&self.tokens, "PlayerDeaths")
    }

    pub fn combat_phase(&self) -> Option<String> {
        self.encounter
            .as_ref()?
            .get("phase")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    pub fn combat_outcome(&self) -> Option<String> {
        self.encounter
            .as_ref()?
            .get("outcome")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    pub fn mining_outcome(&self) -> Option<String> {
        let enc = self.encounter.as_ref()?;
        if enc.get("encounter_state_type").and_then(|v| v.as_str()) != Some("Mining") {
            return None;
        }
        enc.get("outcome")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    pub fn mining_light_level(&self) -> i64 {
        self.encounter
            .as_ref()
            .and_then(|e| e.get("encounter_tokens"))
            .and_then(|t| t.get("MiningLightLevel"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
    }

    pub fn mining_yield(&self) -> i64 {
        self.encounter
            .as_ref()
            .and_then(|e| e.get("encounter_tokens"))
            .and_then(|t| t.get("MiningYield"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
    }

    pub fn mining_durability(&self) -> i64 {
        extract_token_value(&self.tokens, "MiningDurability")
    }

    pub fn player_ore(&self) -> i64 {
        extract_token_value(&self.tokens, "Ore")
    }
}

fn extract_token_value(tokens: &Value, token_type: &str) -> i64 {
    tokens
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .find(|t| {
            t.get("token")
                .and_then(|tok| tok.get("token_type"))
                .and_then(|tt| tt.as_str())
                == Some(token_type)
        })
        .and_then(|t| t.get("value"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
}

/// Strategy trait — all bots implement this.
/// Strategies see only public API data (possible actions + game snapshot).
pub trait Strategy: Send + Sync {
    fn name(&self) -> &str;

    /// Given the list of possible actions and current game state,
    /// choose which action to submit to POST /action.
    fn choose_action(&self, possible_actions: &[Value], game_state: &GameSnapshot) -> Value;
}
