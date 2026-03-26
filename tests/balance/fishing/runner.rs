use rocket::local::blocking::Client;
use serde::Serialize;

use super::game_driver::{FishingGameDriver, FishingGameResult};
use super::output::{
    build_fishing_report, FishingSimulationReport, FishingStrategyReport, FishingWinRateTarget,
    YieldEfficiencyTarget,
};
use crate::strategies::Strategy;

#[derive(Debug, Clone)]
pub struct FishingSimulationConfig {
    pub num_games: u32,
    pub max_encounters_per_game: u32,
    pub max_actions_per_encounter: u32,
    pub base_seed: u64,
}

#[derive(Debug, Serialize)]
pub struct FishingSimulationRunnerReport {
    pub strategies: Vec<FishingStrategyReport>,
    pub all_pass: bool,
}

pub struct FishingSimulationRunner {
    pub config: FishingSimulationConfig,
    pub yield_targets: Vec<YieldEfficiencyTarget>,
    pub win_rate_targets: Vec<FishingWinRateTarget>,
}

impl FishingSimulationRunner {
    pub fn run_strategy(
        &self,
        strategy: &dyn Strategy,
    ) -> (Vec<FishingGameResult>, FishingStrategyReport) {
        let driver = FishingGameDriver {
            max_encounters: self.config.max_encounters_per_game,
            max_actions_per_encounter: self.config.max_actions_per_encounter,
        };

        let client =
            Client::tracked(my_little_cardgame::rocket_initialize()).expect("valid rocket");

        let mut results = Vec::new();
        for game_idx in 0..self.config.num_games {
            let seed = self.config.base_seed + game_idx as u64;
            let result = driver.play_game(&client, seed, strategy);
            results.push(result);
        }

        let report = build_fishing_report(
            strategy.name(),
            &results,
            &self.yield_targets,
            &self.win_rate_targets,
        );

        (results, report)
    }

    pub fn run_all_strategies(&self, strategies: &[Box<dyn Strategy>]) -> FishingSimulationReport {
        let mut all_reports = Vec::new();
        let mut all_pass = true;

        for strategy in strategies {
            let (_results, report) = self.run_strategy(strategy.as_ref());

            println!(
                "\n=== {} ===\n  encounters: {} | wins: {} | losses: {} | win_rate: {:.3}\n  avg_fish: {:.0} | avg_durability_spent: {:.0} | yield_per_durability: {:.4}\n  yield_pass: {} | win_rate_pass: {}",
                report.strategy_name,
                report.total_encounters,
                report.total_wins,
                report.total_losses,
                report.avg_win_rate,
                report.avg_fish_earned,
                report.avg_durability_spent,
                report.avg_yield_per_durability,
                report.yield_pass,
                report.win_rate_pass,
            );

            if !report.yield_pass || !report.win_rate_pass {
                all_pass = false;
            }
            all_reports.push(report);
        }

        FishingSimulationReport {
            strategies: all_reports,
            all_pass,
        }
    }
}
