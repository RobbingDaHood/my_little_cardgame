use crate::player_data::PlayerData;
use rand::{SeedableRng, RngCore};
use rand_pcg::Lcg64Xsh32;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use rocket_okapi::{openapi, JsonSchema};

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct SeedRequest {
    pub seed: u64,
}

#[openapi]
#[post("/player/seed", format = "json", data = "<seed_req>")]
pub async fn set_seed(
    seed_req: Json<SeedRequest>,
    player_data: &State<PlayerData>,
    game_state: &State<std::sync::Arc<rocket::futures::lock::Mutex<crate::library::GameState>>>,
) -> Json<String> {
    let s = seed_req.seed;
    let mut seed_bytes: [u8; 16] = [0u8; 16];
    // fill with two copies of the u64
    seed_bytes[0..8].copy_from_slice(&s.to_le_bytes());
    seed_bytes[8..16].copy_from_slice(&s.to_le_bytes());
    // set seed and RNG
    *player_data.seed.lock().await = seed_bytes;
    let new_rng = Lcg64Xsh32::from_seed(seed_bytes);
    *player_data.random_generator_state.lock().await = new_rng;

    // record seed set in the actions log for deterministic replay
    {
        let gs = game_state.lock().await;
        let payload = crate::library::types::ActionPayload::SetSeed { seed: s };
        gs.action_log.append("SetSeed", payload);
    }

    Json(format!("seed set to {}", s))
}

/// Derive a deterministic u64 subseed by consuming the session RNG and record the draw in the ActionLog.
pub async fn derive_subseed(
    player_data: &State<PlayerData>,
    game_state: &State<std::sync::Arc<rocket::futures::lock::Mutex<crate::library::GameState>>>,
    purpose: &str,
) -> u64 {
    let mut rng = player_data.random_generator_state.lock().await;
    let value = rng.next_u64();
    {
        let gs = game_state.lock().await;
        let payload = crate::library::types::ActionPayload::RngDraw {
            purpose: purpose.to_string(),
            value,
        };
        gs.action_log.append("RngDraw", payload);
    }
    value
}

/// Snapshot the RNG state as a JSON string (requires rand_pcg with serde support).
pub async fn snapshot_rng(player_data: &State<PlayerData>) -> Result<String, String> {
    let rng = player_data.random_generator_state.lock().await;
    serde_json::to_string(&*rng).map_err(|e| e.to_string())
}

/// Restore RNG state from a JSON snapshot produced by `snapshot_rng`.
pub async fn restore_rng_from_snapshot(
    player_data: &State<PlayerData>,
    snapshot: &str,
) -> Result<(), String> {
    let restored: Lcg64Xsh32 = serde_json::from_str(snapshot).map_err(|e| e.to_string())?;
    *player_data.random_generator_state.lock().await = restored;
    Ok(())
}
