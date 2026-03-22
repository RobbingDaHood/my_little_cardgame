use crate::game_driver::{GameDriver, GameResult};
use crate::output::SimulationReport;
use crate::strategies::Strategy;

/// Configuration for a simulation run.
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    pub games_per_strategy: u32,
    pub encounters_per_game: u32,
    pub base_seed: u64,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            games_per_strategy: 1000,
            encounters_per_game: 20,
            base_seed: 12345,
        }
    }
}

/// Per-strategy aggregated results.
#[derive(Debug, Clone)]
pub struct StrategyResults {
    pub name: String,
    pub total_games: u32,
    pub combat_wins: u32,
    pub combat_losses: u32,
    pub total_combat_encounters: u32,
    pub total_deaths: i64,
    pub avg_encounters_before_death: f64,
    pub avg_rounds_per_encounter: f64,
    pub health_sum_final: i64,
    pub games_results: Vec<GameResult>,
}

impl StrategyResults {
    pub fn combat_win_rate(&self) -> f64 {
        if self.total_combat_encounters == 0 {
            return 0.0;
        }
        self.combat_wins as f64 / self.total_combat_encounters as f64
    }
}

/// Runs simulation games for multiple strategies and collects results.
pub struct SimulationRunner {
    pub config: SimulationConfig,
}

impl SimulationRunner {
    pub fn new(config: SimulationConfig) -> Self {
        Self { config }
    }

    pub fn run_strategy(&self, strategy: &dyn Strategy) -> StrategyResults {
        let driver = GameDriver::new(self.config.encounters_per_game);
        let mut results = StrategyResults {
            name: strategy.name().to_string(),
            total_games: self.config.games_per_strategy,
            combat_wins: 0,
            combat_losses: 0,
            total_combat_encounters: 0,
            total_deaths: 0,
            avg_encounters_before_death: 0.0,
            avg_rounds_per_encounter: 0.0,
            health_sum_final: 0,
            games_results: Vec::new(),
        };

        let mut total_rounds: u64 = 0;
        let mut total_encounter_count: u64 = 0;

        for i in 0..self.config.games_per_strategy {
            let seed = self.config.base_seed + i as u64;
            let game_result = driver.play_game(seed, strategy);

            results.combat_wins += game_result.combat_wins;
            results.combat_losses += game_result.combat_losses;
            results.total_combat_encounters += game_result.combat_wins + game_result.combat_losses;
            results.total_deaths += game_result.deaths;
            results.health_sum_final += game_result.final_health;

            for &rounds in &game_result.rounds_per_encounter {
                total_rounds += rounds as u64;
                total_encounter_count += 1;
            }

            results.games_results.push(game_result);
        }

        if total_encounter_count > 0 {
            results.avg_rounds_per_encounter = total_rounds as f64 / total_encounter_count as f64;
        }
        if results.total_deaths > 0 {
            results.avg_encounters_before_death =
                results.total_combat_encounters as f64 / results.total_deaths as f64;
        }

        results
    }

    pub fn run_all(&self, strategies: &[&dyn Strategy]) -> SimulationReport {
        let mut strategy_results = Vec::new();
        for strategy in strategies {
            eprintln!(
                "Running {} strategy ({} games)...",
                strategy.name(),
                self.config.games_per_strategy
            );
            let result = self.run_strategy(*strategy);
            eprintln!(
                "  Combat win rate: {:.1}% ({}/{})",
                result.combat_win_rate() * 100.0,
                result.combat_wins,
                result.total_combat_encounters
            );
            strategy_results.push(result);
        }
        SimulationReport::from_results(&self.config, strategy_results)
    }
}
