use serde::Serialize;

use crate::runner::{SimulationConfig, StrategyResults};

/// Yield-per-durability target for a mining strategy.
#[derive(Debug, Clone)]
pub struct YieldTarget {
    pub strategy: String,
    pub target_min: f64,
    pub target_max: f64,
}

/// Mining balance targets: yield per durability ratio.
///
/// Tuned values (B2.4):
///   Tactician ≈ 0.34, Greedy ≈ 0.30, Random ≈ 0.28, Conservative ≈ 0.11
///
/// Bands are ±30–40% to absorb minor game-logic changes while catching regressions.
pub fn mining_yield_targets() -> Vec<YieldTarget> {
    vec![
        YieldTarget {
            strategy: "random".to_string(),
            target_min: 0.18,
            target_max: 0.40,
        },
        YieldTarget {
            strategy: "greedy".to_string(),
            target_min: 0.20,
            target_max: 0.42,
        },
        YieldTarget {
            strategy: "conservative".to_string(),
            target_min: 0.01,
            target_max: 0.25,
        },
        YieldTarget {
            strategy: "tactician".to_string(),
            target_min: 0.24,
            target_max: 0.46,
        },
    ]
}

/// Mining-specific metrics for a strategy.
#[derive(Debug, Serialize)]
pub struct MiningReport {
    pub total_encounters: u32,
    pub wins: u32,
    pub losses: u32,
    pub win_rate: f64,
    pub avg_rounds_per_encounter: f64,
    pub avg_yield_per_durability: f64,
    pub yield_target_min: f64,
    pub yield_target_max: f64,
    pub yield_pass: bool,
    pub total_yield: i64,
    pub total_durability: i64,
}

/// Per-strategy report including mining metrics.
#[derive(Debug, Serialize)]
pub struct MiningStrategyReport {
    pub name: String,
    pub total_games: u32,
    pub mining: MiningReport,
    pub total_deaths: i64,
    pub avg_encounters_before_death: f64,
    pub avg_health_final: f64,
}

/// Full mining simulation report.
#[derive(Debug, Serialize)]
pub struct MiningSimulationReport {
    pub config: MiningReportConfig,
    pub strategies: Vec<MiningStrategyReport>,
    pub all_assertions_passed: bool,
}

#[derive(Debug, Serialize)]
pub struct MiningReportConfig {
    pub games_per_strategy: u32,
    pub encounters_per_game: u32,
    pub base_seed: u64,
}

impl MiningSimulationReport {
    pub fn from_results(config: &SimulationConfig, results: Vec<StrategyResults>) -> Self {
        let targets = mining_yield_targets();
        let mut all_pass = true;

        let strategies: Vec<MiningStrategyReport> = results
            .iter()
            .map(|r| {
                let target = targets
                    .iter()
                    .find(|t| t.strategy == r.name)
                    .cloned()
                    .unwrap_or(YieldTarget {
                        strategy: r.name.clone(),
                        target_min: 0.2,
                        target_max: 0.4,
                    });

                let yield_pass = r.avg_yield_per_durability >= target.target_min
                    && r.avg_yield_per_durability <= target.target_max;

                if !yield_pass {
                    all_pass = false;
                }

                let total_yield: i64 = r.games_results.iter().map(|g| g.yield_total).sum();
                let total_durability: i64 =
                    r.games_results.iter().map(|g| g.durability_spent).sum();

                let avg_health = if r.total_games > 0 {
                    r.health_sum_final as f64 / r.total_games as f64
                } else {
                    0.0
                };

                MiningStrategyReport {
                    name: r.name.clone(),
                    total_games: r.total_games,
                    mining: MiningReport {
                        total_encounters: r.total_encounters,
                        wins: r.wins,
                        losses: r.losses,
                        win_rate: r.win_rate(),
                        avg_rounds_per_encounter: r.avg_rounds_per_encounter,
                        avg_yield_per_durability: r.avg_yield_per_durability,
                        yield_target_min: target.target_min,
                        yield_target_max: target.target_max,
                        yield_pass,
                        total_yield,
                        total_durability,
                    },
                    total_deaths: r.total_deaths,
                    avg_encounters_before_death: r.avg_encounters_before_death,
                    avg_health_final: avg_health,
                }
            })
            .collect();

        MiningSimulationReport {
            config: MiningReportConfig {
                games_per_strategy: config.games_per_strategy,
                encounters_per_game: config.encounters_per_game,
                base_seed: config.base_seed,
            },
            strategies,
            all_assertions_passed: all_pass,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}
