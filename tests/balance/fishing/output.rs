use serde::Serialize;

use crate::runner::{SimulationConfig, StrategyResults};

/// Yield-per-durability target for a fishing strategy.
#[derive(Debug, Clone)]
pub struct YieldTarget {
    pub strategy: String,
    pub target_min: f64,
    pub target_max: f64,
}

/// Fishing balance targets from docs/vision/balances/fishing_balance.md:
///   Goal: 0.2–0.4 yield per durability for simple strategies.
///   Tactician should be significantly better via encounter selection.
pub fn fishing_yield_targets() -> Vec<YieldTarget> {
    vec![
        YieldTarget {
            strategy: "FishingRandom".to_string(),
            target_min: 0.15,
            target_max: 0.60,
        },
        YieldTarget {
            strategy: "FishingGreedy".to_string(),
            target_min: 0.15,
            target_max: 0.60,
        },
        YieldTarget {
            strategy: "FishingConservative".to_string(),
            target_min: 0.15,
            target_max: 0.60,
        },
        YieldTarget {
            strategy: "FishingTactician".to_string(),
            target_min: 1.0,
            target_max: 3.0,
        },
    ]
}

#[derive(Debug, Serialize)]
pub struct FishingReport {
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

#[derive(Debug, Serialize)]
pub struct FishingStrategyReport {
    pub name: String,
    pub total_games: u32,
    pub fishing: FishingReport,
    pub total_deaths: i64,
    pub avg_encounters_before_death: f64,
    pub avg_health_final: f64,
}

#[derive(Debug, Serialize)]
pub struct FishingSimulationReport {
    pub config: FishingReportConfig,
    pub strategies: Vec<FishingStrategyReport>,
    pub all_assertions_passed: bool,
}

#[derive(Debug, Serialize)]
pub struct FishingReportConfig {
    pub games_per_strategy: u32,
    pub encounters_per_game: u32,
    pub base_seed: u64,
}

impl FishingSimulationReport {
    pub fn from_results(config: &SimulationConfig, results: Vec<StrategyResults>) -> Self {
        let targets = fishing_yield_targets();
        let mut all_pass = true;

        let strategies: Vec<FishingStrategyReport> = results
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

                FishingStrategyReport {
                    name: r.name.clone(),
                    total_games: r.total_games,
                    fishing: FishingReport {
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

        FishingSimulationReport {
            config: FishingReportConfig {
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
