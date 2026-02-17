use rocket::serde::json::Json;
use rocket_okapi::openapi;

use crate::library::{types::ActionEntry, GameState};

#[derive(rocket::serde::Serialize, rocket::serde::Deserialize, rocket_okapi::JsonSchema, Debug)]
#[serde(crate = "rocket::serde")]
pub struct ActionLogResponse {
    pub entries: Vec<ActionEntry>,
    pub next_seq: Option<u64>,
    pub limit: usize,
}

#[openapi]
#[get("/actions/log?<from_seq>&<limit>&<action_type>&<since>")]
pub async fn list_actions_log(
    from_seq: Option<u64>,
    limit: Option<usize>,
    action_type: Option<String>,
    since: Option<u128>,
    game_state: &rocket::State<std::sync::Arc<rocket::futures::lock::Mutex<GameState>>>,
) -> Json<ActionLogResponse> {
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
    let has_more = filtered.len() > max;
    filtered.truncate(max);
    let next_seq = if has_more {
        filtered.last().map(|e| e.seq + 1)
    } else {
        None
    };
    Json(ActionLogResponse {
        entries: filtered,
        next_seq,
        limit: max,
    })
}
