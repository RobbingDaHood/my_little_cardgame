use super::types::{Discipline, EncounterOutcome, EncounterRecord, TokenType};
use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket_okapi::{openapi, JsonSchema};
use std::collections::HashMap;

/// Per-discipline encounter statistics.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct DisciplineMetrics {
    pub discipline: Discipline,
    pub wins: u64,
    pub losses: u64,
    pub total: u64,
    pub win_rate: f64,
    pub avg_rounds: f64,
    pub token_deltas: HashMap<String, TokenSnapshot>,
}

/// Min/max/average snapshot for a token across encounters.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct TokenSnapshot {
    pub avg_start: f64,
    pub avg_end: f64,
    pub min_start: i64,
    pub max_start: i64,
    pub min_end: i64,
    pub max_end: i64,
}

/// Session-level statistics aggregated from encounter records.
///
/// Use this endpoint to monitor game balance during testing. Metrics track
/// per-discipline win/loss rates, average encounter duration, token flow
/// patterns, and cumulative session statistics. Resets on NewGame.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct SessionMetrics {
    pub total_encounters: u64,
    pub total_deaths: u64,
    pub per_discipline: Vec<DisciplineMetrics>,
    pub resource_flow: HashMap<String, ResourceFlow>,
}

/// Net resource inflow/outflow for a material token across all encounters.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct ResourceFlow {
    pub total_gained: i64,
    pub total_spent: i64,
    pub net: i64,
}

fn tracked_tokens() -> Vec<TokenType> {
    vec![
        TokenType::Health,
        TokenType::Stamina,
        TokenType::Shield,
        TokenType::Dodge,
        TokenType::Mana,
    ]
}

fn material_tokens() -> Vec<TokenType> {
    vec![
        TokenType::Ore,
        TokenType::Plant,
        TokenType::Lumber,
        TokenType::Fish,
    ]
}

fn compute_discipline_metrics(
    discipline: &Discipline,
    records: &[&EncounterRecord],
) -> DisciplineMetrics {
    let wins = records
        .iter()
        .filter(|r| r.outcome == EncounterOutcome::PlayerWon)
        .count() as u64;
    let losses = records
        .iter()
        .filter(|r| r.outcome == EncounterOutcome::PlayerLost)
        .count() as u64;
    let total = records.len() as u64;
    let win_rate = if total > 0 {
        wins as f64 / total as f64
    } else {
        0.0
    };
    let avg_rounds = if total > 0 {
        records.iter().map(|r| r.rounds as f64).sum::<f64>() / total as f64
    } else {
        0.0
    };

    let mut token_deltas = HashMap::new();
    for token_type in tracked_tokens() {
        let key = format!("{token_type:?}");
        let starts: Vec<i64> = records
            .iter()
            .filter_map(|r| r.tokens_at_start.get(&token_type).copied())
            .collect();
        let ends: Vec<i64> = records
            .iter()
            .filter_map(|r| r.tokens_at_end.get(&token_type).copied())
            .collect();
        if starts.is_empty() && ends.is_empty() {
            continue;
        }
        token_deltas.insert(
            key,
            TokenSnapshot {
                avg_start: if starts.is_empty() {
                    0.0
                } else {
                    starts.iter().sum::<i64>() as f64 / starts.len() as f64
                },
                avg_end: if ends.is_empty() {
                    0.0
                } else {
                    ends.iter().sum::<i64>() as f64 / ends.len() as f64
                },
                min_start: starts.iter().copied().min().unwrap_or(0),
                max_start: starts.iter().copied().max().unwrap_or(0),
                min_end: ends.iter().copied().min().unwrap_or(0),
                max_end: ends.iter().copied().max().unwrap_or(0),
            },
        );
    }

    DisciplineMetrics {
        discipline: discipline.clone(),
        wins,
        losses,
        total,
        win_rate,
        avg_rounds,
        token_deltas,
    }
}

pub fn compute_session_metrics(
    records: &[EncounterRecord],
    token_balances: &HashMap<super::types::Token, i64>,
) -> SessionMetrics {
    let total_encounters = records.len() as u64;
    let deaths_key = super::types::Token::persistent(TokenType::PlayerDeaths);
    let total_deaths = token_balances.get(&deaths_key).copied().unwrap_or(0).max(0) as u64;

    let disciplines = [
        Discipline::Combat,
        Discipline::Mining,
        Discipline::Herbalism,
        Discipline::Woodcutting,
        Discipline::Fishing,
        Discipline::Rest,
        Discipline::Crafting,
        Discipline::Research,
    ];

    let per_discipline: Vec<DisciplineMetrics> = disciplines
        .iter()
        .map(|d| {
            let disc_records: Vec<&EncounterRecord> =
                records.iter().filter(|r| r.discipline == *d).collect();
            compute_discipline_metrics(d, &disc_records)
        })
        .filter(|dm| dm.total > 0)
        .collect();

    let mut resource_flow = HashMap::new();
    for token_type in material_tokens() {
        let key = format!("{token_type:?}");
        let mut total_gained: i64 = 0;
        let mut total_spent: i64 = 0;
        for record in records {
            let start = record
                .tokens_at_start
                .get(&token_type)
                .copied()
                .unwrap_or(0);
            let end = record.tokens_at_end.get(&token_type).copied().unwrap_or(0);
            let delta = end - start;
            if delta > 0 {
                total_gained += delta;
            } else {
                total_spent += -delta;
            }
        }
        resource_flow.insert(
            key,
            ResourceFlow {
                total_gained,
                total_spent,
                net: total_gained - total_spent,
            },
        );
    }

    SessionMetrics {
        total_encounters,
        total_deaths,
        per_discipline,
        resource_flow,
    }
}

/// Session-level statistics: per-encounter-type win rates, average turns,
/// token balance snapshots, total deaths, and resource inflow/outflow.
/// Metrics accumulate in-memory during gameplay and reset on NewGame.
#[openapi]
#[get("/metrics")]
pub async fn get_metrics(
    game_state: &rocket::State<
        std::sync::Arc<rocket::futures::lock::Mutex<super::game_state::GameState>>,
    >,
) -> Json<SessionMetrics> {
    let gs = game_state.lock().await;
    Json(compute_session_metrics(
        &gs.encounter_records,
        &gs.token_balances,
    ))
}
