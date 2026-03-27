use serde::Serialize;

use crate::output::ReportConfig;
use crate::runner::{SimulationConfig, StrategyResults};

/// Yield-per-durability target for a woodcutting strategy.
#[derive(Debug, Clone)]
pub struct YieldTarget {
    pub strategy: String,
    pub target_min: f64,
    pub target_max: f64,
}

/// Woodcutting balance targets: yield per durability ratio.
/// Cross-discipline target: 0.2–0.4 yield per durability.
/// All gathering disciplines share this target band.
pub fn woodcutting_yield_targets() -> Vec<YieldTarget> {
    vec![
        YieldTarget {
            strategy: "random".to_string(),
            target_min: 0.2,
            target_max: 0.4,
        },
        YieldTarget {
            strategy: "greedy".to_string(),
            target_min: 0.2,
            target_max: 0.4,
        },
        YieldTarget {
            strategy: "conservative".to_string(),
            target_min: 0.2,
            target_max: 0.4,
        },
        YieldTarget {
            strategy: "pattern_builder".to_string(),
            target_min: 0.2,
            target_max: 0.4,
        },
    ]
}

/// Woodcutting-specific metrics for a strategy.
#[derive(Debug, Serialize)]
pub struct WoodcuttingReport {
    pub total_encounters: u32,
    pub wins: u32,
    pub losses: u32,
    pub win_rate: f64,
    pub yield_per_durability: f64,
    pub target_min: f64,
    pub target_max: f64,
    pub yield_pass: bool,
    pub avg_rounds: f64,
}

/// Per-strategy report including woodcutting metrics.
#[derive(Debug, Serialize)]
pub struct WoodcuttingStrategyReport {
    pub name: String,
    pub total_games: u32,
    pub woodcutting: WoodcuttingReport,
    pub total_deaths: i64,
    pub avg_encounters_before_death: f64,
    pub avg_health_final: f64,
}

/// Full woodcutting simulation report.
#[derive(Debug, Serialize)]
pub struct WoodcuttingSimulationReport {
    pub config: ReportConfig,
    pub strategies: Vec<WoodcuttingStrategyReport>,
    pub all_assertions_passed: bool,
}

impl WoodcuttingSimulationReport {
    pub fn from_results(config: &SimulationConfig, results: Vec<StrategyResults>) -> Self {
        let targets = woodcutting_yield_targets();
        let mut all_pass = true;

        let strategies: Vec<WoodcuttingStrategyReport> = results
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

                let avg_health = if r.total_games > 0 {
                    r.health_sum_final as f64 / r.total_games as f64
                } else {
                    0.0
                };

                WoodcuttingStrategyReport {
                    name: r.name.clone(),
                    total_games: r.total_games,
                    woodcutting: WoodcuttingReport {
                        total_encounters: r.total_encounters,
                        wins: r.wins,
                        losses: r.losses,
                        win_rate: r.win_rate(),
                        yield_per_durability: r.avg_yield_per_durability,
                        target_min: target.target_min,
                        target_max: target.target_max,
                        yield_pass,
                        avg_rounds: r.avg_rounds_per_encounter,
                    },
                    total_deaths: r.total_deaths,
                    avg_encounters_before_death: r.avg_encounters_before_death,
                    avg_health_final: avg_health,
                }
            })
            .collect();

        WoodcuttingSimulationReport {
            config: ReportConfig {
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
