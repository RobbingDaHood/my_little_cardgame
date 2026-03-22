use serde::Serialize;

use crate::combat::output::{
    combat_streak_targets, combat_targets, CombatReport, WinRateTarget, WinStreakTarget,
};
use crate::runner::{SimulationConfig, StrategyResults};

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

impl SimulationReport {
    pub fn from_results(config: &SimulationConfig, results: Vec<StrategyResults>) -> Self {
        let targets = combat_targets();
        let streak_targets = combat_streak_targets();
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

                let streak_target = streak_targets
                    .iter()
                    .find(|t| t.strategy == r.name)
                    .cloned()
                    .unwrap_or(WinStreakTarget {
                        strategy: r.name.clone(),
                        target_min_streak: 0.0,
                        target_max_streak: f64::MAX,
                    });

                let win_rate = r.combat_win_rate();
                let pass = win_rate >= target.target_min && win_rate <= target.target_max;
                let streak_pass = r.overall_avg_streak >= streak_target.target_min_streak
                    && r.overall_avg_streak <= streak_target.target_max_streak;
                let rounds_pass = r.avg_rounds_per_encounter >= 3.0;

                if !streak_pass || !rounds_pass {
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
                        avg_max_win_streak: r.avg_max_win_streak,
                        overall_avg_streak: r.overall_avg_streak,
                        streak_target_min: streak_target.target_min_streak,
                        streak_target_max: streak_target.target_max_streak,
                        streak_pass,
                        rounds_pass,
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
