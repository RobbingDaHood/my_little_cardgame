use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};

/// Scouting parameters that influence encounter replacement
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct ScoutingParams {
    pub preview_count: usize,
    pub affix_bias: Option<Vec<String>>,
    pub pool_modifier: Option<String>,
}

impl ScoutingParams {
    pub fn new(preview_count: usize) -> Self {
        ScoutingParams {
            preview_count,
            affix_bias: None,
            pool_modifier: None,
        }
    }

    pub fn with_affix_bias(mut self, bias: Vec<String>) -> Self {
        self.affix_bias = Some(bias);
        self
    }

    pub fn with_pool_modifier(mut self, modifier: String) -> Self {
        self.pool_modifier = Some(modifier);
        self
    }

    /// Apply scouting parameters to bias affix selection
    pub fn apply_to_seed(&self, base_seed: u64) -> u64 {
        let mut result = base_seed;

        if let Some(ref bias) = self.affix_bias {
            for affix in bias {
                result = result.wrapping_mul(31).wrapping_add(affix.len() as u64);
            }
        }

        if let Some(ref modifier) = self.pool_modifier {
            result = result.wrapping_mul(17).wrapping_add(modifier.len() as u64);
        }

        result.wrapping_add(self.preview_count as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scouting_params_creation() {
        let params = ScoutingParams::new(3);
        assert_eq!(params.preview_count, 3);
        assert!(params.affix_bias.is_none());
        assert!(params.pool_modifier.is_none());
    }

    #[test]
    fn test_deterministic_seed_modification() {
        let params1 =
            ScoutingParams::new(3).with_affix_bias(vec!["fire".to_string(), "poison".to_string()]);
        let params2 =
            ScoutingParams::new(3).with_affix_bias(vec!["fire".to_string(), "poison".to_string()]);

        let base_seed = 12345u64;
        assert_eq!(
            params1.apply_to_seed(base_seed),
            params2.apply_to_seed(base_seed)
        );
    }
}
