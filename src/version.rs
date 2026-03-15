use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use sha2::{Digest, Sha256};

const GAME_VERSION: &str = "0.0.1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct VersionInfo {
    /// Full version string: game version + configuration hash (e.g. "0.0.1-a1b2c3d4").
    pub version: String,
    /// Base game version without configuration hash (e.g. "0.0.1").
    pub game_version: String,
    /// SHA-256 hash prefix of the combined configuration (8 hex chars).
    pub config_hash: String,
}

fn compute_config_hash() -> String {
    let configs = crate::library::config_loader::all_config_json_strings();
    let mut hasher = Sha256::new();
    for cfg in &configs {
        hasher.update(cfg.as_bytes());
    }
    let result = hasher.finalize();
    format!("{:x}", result)[..8].to_string()
}

/// Current game version and configuration fingerprint.
///
/// Returns the game version (semver) combined with a short hash of all
/// configuration files. Two instances running identical configs will report
/// the same version string, making it easy to verify config parity across
/// environments or after a deploy.
#[openapi]
#[get("/version")]
pub fn get_version() -> Json<VersionInfo> {
    let config_hash = compute_config_hash();
    let version = format!("{GAME_VERSION}-{config_hash}");
    Json(VersionInfo {
        version,
        game_version: GAME_VERSION.to_string(),
        config_hash,
    })
}
