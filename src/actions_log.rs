use rocket::serde::json::Json;
use rocket_okapi::openapi;

use crate::library::{types::ActionEntry, GameState};

#[openapi]
#[get("/actions/log?<from_seq>&<limit>&<action_type>&<since>")]
pub async fn list_actions_log(
    from_seq: Option<u64>,
    limit: Option<usize>,
    action_type: Option<String>,
    since: Option<u128>,
    game_state: &rocket::State<std::sync::Arc<rocket::futures::lock::Mutex<GameState>>>,
) -> Json<Vec<ActionEntry>> {
    let gs = game_state.lock().await;
    let entries = gs.action_log.entries();
    let mut filtered: Vec<ActionEntry> = entries
        .into_iter()
        .filter(|e| {
            if let Some(f) = from_seq {
                if e.seq < f {
                    return false;
                }
            }
            if let Some(ref at) = action_type {
                if e.action_type != *at {
                    return false;
                }
            }
            if let Some(s) = since {
                if let Ok(ts) = e.timestamp.parse::<u128>() {
                    if ts < s {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            true
        })
        .collect();
    let max = limit.unwrap_or(1000);
    filtered.truncate(max);
    Json(filtered)
}
