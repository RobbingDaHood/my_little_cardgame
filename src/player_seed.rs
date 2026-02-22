use crate::player_data::PlayerData;
use rand::SeedableRng;
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
    seed_bytes[0..8].copy_from_slice(&s.to_le_bytes());
    seed_bytes[8..16].copy_from_slice(&s.to_le_bytes());
    *player_data.seed.lock().await = seed_bytes;
    let new_rng = Lcg64Xsh32::from_seed(seed_bytes);
    *player_data.random_generator_state.lock().await = new_rng;

    {
        let gs = game_state.lock().await;
        let payload = crate::library::types::ActionPayload::SetSeed { seed: s };
        let log_arc = std::sync::Arc::clone(&gs.action_log);
        drop(gs);
        log_arc.append_async("SetSeed", payload).await;
    }

    Json(format!("seed set to {}", s))
}
