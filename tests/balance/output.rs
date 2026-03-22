use serde::Serialize;

use crate::runner::{SimulationConfig, StrategyResults};

/// Win-rate target for a strategy.
#[derive(Debug, Clone, Serialize)]
pub struct WinRateTarget {
    pub strategy: String,
    pub target_min: f64,
    pub target_max: f64,
}

/// Combat balance assertions from vision.md (±10%).
pub fn combat_targets() -> Vec<WinRateTarget> {
    vec![
        WinRateTarget {
            strategy: "random".to_string(),
            target_min: 0.20,
            target_max: 0.40,
        },
        WinRateTarget {
            strategy: "greedy".to_string(),
            target_min: 0.40,
            target_max: 0.60,
        },
        WinRateTarget {
            strategy: "conservative".to_string(),
            target_min: 0.30,
            target_max: 0.50,
        },
    ]
}

/// Full simulation report — serializes to JSON for stdout.
#[derive(Debug, Serialize)]
pub struct SimulationReport {
    pub config: ReportConfig,
    pub strategies: Vec<StrategyReport>,
    pub all_assertions_passed: bool,
}

#[derive(Debug, Serialize)]
pub struct ReportConfig {
    pub games_per_strategy: u32,
    pub encounters_per_game: u32,
    pub base_seed: u64,
}

#[derive(Debug, Serialize)]
pub struct StrategyReport {
    pub name: String,
    pub total_games: u32,
    pub combat: CombatReport,
    pub total_deaths: i64,
    pub avg_encounters_before_death: f64,
    pub avg_health_final: f64,
}

#[derive(Debug, Serialize)]
pub struct CombatReport {
    pub total_encounters: u32,
    pub wins: u32,
    pub losses: u32,
    pub win_rate: f64,
    pub target_min: f64,
    pub target_max: f64,
    pub pass: bool,
    pub avg_rounds_per_encounter: f64,
}

impl SimulationReport {
    pub fn from_results(config: &SimulationConfig, results: Vec<StrategyResults>) -> Self {
        let targets = combat_targets();
        let mut all_pass = true;

        let strategies: Vec<StrategyReport> = results
            .iter()
            .map(|r| {
                let target = targets
                    .iter()
                    .find(|t| t.strategy == r.name)
                    .cloned()
                    .unwrap_or(WinRateTarget {
                        strategy: r.name.clone(),
                        target_min: 0.0,
                        target_max: 1.0,
                    });

                let win_rate = r.combat_win_rate();
                let pass = win_rate >= target.target_min && win_rate <= target.target_max;
                if !pass {
                    all_pass = false;
                }

                let avg_health = if r.total_games > 0 {
                    r.health_sum_final as f64 / r.total_games as f64
                } else {
                    0.0
                };

                StrategyReport {
                    name: r.name.clone(),
                    total_games: r.total_games,
                    combat: CombatReport {
                        total_encounters: r.total_combat_encounters,
                        wins: r.combat_wins,
                        losses: r.combat_losses,
                        win_rate,
                        target_min: target.target_min,
                        target_max: target.target_max,
                        pass,
                        avg_rounds_per_encounter: r.avg_rounds_per_encounter,
                    },
                    total_deaths: r.total_deaths,
                    avg_encounters_before_death: r.avg_encounters_before_death,
                    avg_health_final: avg_health,
                }
            })
            .collect();

        SimulationReport {
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
