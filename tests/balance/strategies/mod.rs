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

    pub fn encounter_state_type(&self) -> Option<String> {
        self.encounter
            .as_ref()?
            .get("encounter_state_type")
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

    pub fn herbalism_outcome(&self) -> Option<String> {
        let enc = self.encounter.as_ref()?;
        if enc.get("encounter_state_type").and_then(|v| v.as_str()) != Some("Herbalism") {
            return None;
        }
        enc.get("outcome")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    pub fn fishing_outcome(&self) -> Option<String> {
        let enc = self.encounter.as_ref()?;
        if enc.get("encounter_state_type").and_then(|v| v.as_str()) != Some("Fishing") {
            return None;
        }
        enc.get("outcome")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    /// Count surviving plants (counts.hand > 0) in the current herbalism encounter.
    pub fn herbalism_plant_count(&self) -> Option<usize> {
        let enc = self.encounter.as_ref()?;
        if enc.get("encounter_state_type").and_then(|v| v.as_str()) != Some("Herbalism") {
            return None;
        }
        let plants = enc.get("plant_hand")?.as_array()?;
        Some(
            plants
                .iter()
                .filter(|p| {
                    p.get("counts")
                        .and_then(|c| c.get("hand"))
                        .and_then(|h| h.as_u64())
                        .unwrap_or(0)
                        > 0
                })
                .count(),
        )
    }

    /// Get the characteristics of all surviving plants in the current herbalism encounter.
    pub fn herbalism_plant_characteristics(&self) -> Option<Vec<Vec<String>>> {
        let enc = self.encounter.as_ref()?;
        if enc.get("encounter_state_type").and_then(|v| v.as_str()) != Some("Herbalism") {
            return None;
        }
        let plants = enc.get("plant_hand")?.as_array()?;
        Some(
            plants
                .iter()
                .filter(|p| {
                    p.get("counts")
                        .and_then(|c| c.get("hand"))
                        .and_then(|h| h.as_u64())
                        .unwrap_or(0)
                        > 0
                })
                .map(|p| {
                    p.get("characteristics")
                        .and_then(|c| c.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default()
                })
                .collect(),
        )
    }

    pub fn herbalism_durability(&self) -> i64 {
        extract_token_value(&self.tokens, "HerbalismDurability")
    }

    pub fn plant_tokens(&self) -> i64 {
        extract_token_value(&self.tokens, "Plant")
    }

    pub fn woodcutting_outcome(&self) -> Option<String> {
        let enc = self.encounter.as_ref()?;
        if enc.get("encounter_state_type").and_then(|v| v.as_str()) != Some("Woodcutting") {
            return None;
        }
        enc.get("outcome")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    pub fn woodcutting_durability(&self) -> i64 {
        extract_token_value(&self.tokens, "WoodcuttingDurability")
    }

    pub fn player_lumber(&self) -> i64 {
        extract_token_value(&self.tokens, "Lumber")
    }

    pub fn fishing_durability(&self) -> i64 {
        extract_token_value(&self.tokens, "FishingDurability")
    }

    pub fn fish_tokens(&self) -> i64 {
        extract_token_value(&self.tokens, "Fish")
    }

    pub fn fishing_turns_won(&self) -> Option<i32> {
        self.encounter
            .as_ref()?
            .get("turns_won")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
    }

    pub fn fishing_round(&self) -> Option<u32> {
        self.encounter
            .as_ref()?
            .get("round")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
    }

    pub fn fishing_max_turns(&self) -> Option<u32> {
        self.encounter
            .as_ref()?
            .get("max_turns")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
    }

    pub fn fishing_win_turns_needed(&self) -> Option<u32> {
        self.encounter
            .as_ref()?
            .get("win_turns_needed")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
    }

    /// The fish value for the current round (drawn at round start).
    pub fn fishing_current_fish_value(&self) -> Option<i64> {
        self.encounter.as_ref()?.get("current_fish_value")?.as_i64()
    }

    /// Current valid range [min, max] from the encounter state.
    pub fn fishing_valid_range(&self) -> Option<(i64, i64)> {
        let enc = self.encounter.as_ref()?;
        let min = enc.get("valid_range_min").and_then(|v| v.as_i64())?;
        let max = enc.get("valid_range_max").and_then(|v| v.as_i64())?;
        Some((min, max))
    }

    /// Fish deck composition: returns (value, remaining_count) pairs.
    /// Remaining count is the sum of hand + deck counts (fish available).
    pub fn fishing_fish_deck(&self) -> Vec<(i64, u32)> {
        let enc = match self.encounter.as_ref() {
            Some(e) => e,
            None => return vec![],
        };
        let deck = match enc.get("fish_deck").and_then(|v| v.as_array()) {
            Some(d) => d,
            None => return vec![],
        };
        deck.iter()
            .filter_map(|fish| {
                let value = fish.get("value").and_then(|v| v.as_i64())?;
                let counts = fish.get("counts")?;
                let in_deck = counts.get("deck").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let in_hand = counts.get("hand").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let available = in_deck + in_hand;
                if available > 0 {
                    Some((value, available))
                } else {
                    None
                }
            })
            .collect()
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
