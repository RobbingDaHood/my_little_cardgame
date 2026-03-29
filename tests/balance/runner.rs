use crate::game_driver::{DisciplineDriver, GameDriver, GameResult};
use crate::strategies::Strategy;

/// Configuration for a simulation run.
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    pub games_per_strategy: u32,
    pub encounters_per_game: u32,
    pub base_seed: u64,
    /// Max actions per encounter before timeout. Defaults to 200 (sufficient for combat).
    /// Mining encounters are much longer (40-200+ rounds) and need higher limits.
    pub max_actions_per_encounter: u32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            games_per_strategy: 1000,
            encounters_per_game: 20,
            base_seed: 12345,
            max_actions_per_encounter: 200,
        }
    }
}

/// Per-strategy aggregated results.
#[derive(Debug, Clone)]
pub struct StrategyResults {
    pub name: String,
    pub total_games: u32,
    pub wins: u32,
    pub losses: u32,
    pub total_encounters: u32,
    pub total_deaths: i64,
    pub avg_encounters_before_death: f64,
    pub avg_rounds_per_encounter: f64,
    pub health_sum_final: i64,
    pub games_results: Vec<GameResult>,
    /// Average of per-game max win streaks.
    pub avg_max_win_streak: f64,
    /// Mean of all individual win streaks across all games.
    pub overall_avg_streak: f64,
    /// Discipline-specific: average yield per durability across all games.
    pub avg_yield_per_durability: f64,
    /// Total cross-discipline resource consumed (e.g., Lumber in mining).
    pub total_cross_resource_consumed: i64,
}

impl StrategyResults {
    pub fn win_rate(&self) -> f64 {
        if self.total_encounters == 0 {
            return 0.0;
        }
        self.wins as f64 / self.total_encounters as f64
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

    pub fn run_strategy(
        &self,
        strategy: &dyn Strategy,
        discipline: &dyn DisciplineDriver,
    ) -> StrategyResults {
        let driver = GameDriver::with_max_actions(
            self.config.encounters_per_game,
            self.config.max_actions_per_encounter,
        );
        let mut results = StrategyResults {
            name: strategy.name().to_string(),
            total_games: self.config.games_per_strategy,
            wins: 0,
            losses: 0,
            total_encounters: 0,
            total_deaths: 0,
            avg_encounters_before_death: 0.0,
            avg_rounds_per_encounter: 0.0,
            health_sum_final: 0,
            games_results: Vec::new(),
            avg_max_win_streak: 0.0,
            overall_avg_streak: 0.0,
            avg_yield_per_durability: 0.0,
            total_cross_resource_consumed: 0,
        };

        let mut total_rounds: u64 = 0;
        let mut total_encounter_count: u64 = 0;
        let mut sum_max_streaks: u64 = 0;
        let mut all_streaks: Vec<u32> = Vec::new();
        let mut total_yield: i64 = 0;
        let mut total_durability: i64 = 0;
        let mut total_cross_resource: i64 = 0;

        for i in 0..self.config.games_per_strategy {
            let seed = self.config.base_seed + i as u64;
            let game_result = driver.play_game(seed, strategy, discipline);

            results.wins += game_result.wins;
            results.losses += game_result.losses;
            results.total_encounters += game_result.total_encounters;
            results.total_deaths += game_result.deaths;
            results.health_sum_final += game_result.final_health;

            total_yield += game_result.yield_total;
            total_durability += game_result.durability_spent;
            total_cross_resource += game_result.cross_resource_consumed;

            for &rounds in &game_result.rounds_per_encounter {
                total_rounds += rounds as u64;
                total_encounter_count += 1;
            }

            sum_max_streaks += game_result.max_win_streak as u64;
            all_streaks.extend_from_slice(&game_result.win_streaks);

            results.games_results.push(game_result);
        }

        if total_encounter_count > 0 {
            results.avg_rounds_per_encounter = total_rounds as f64 / total_encounter_count as f64;
        }
        if results.total_deaths > 0 {
            results.avg_encounters_before_death =
                results.total_encounters as f64 / results.total_deaths as f64;
        }
        if results.total_games > 0 {
            results.avg_max_win_streak = sum_max_streaks as f64 / results.total_games as f64;
        }
        if !all_streaks.is_empty() {
            let streak_sum: u64 = all_streaks.iter().map(|&s| s as u64).sum();
            results.overall_avg_streak = streak_sum as f64 / all_streaks.len() as f64;
        }
        results.total_cross_resource_consumed = total_cross_resource;
        if total_durability > 0 {
            results.avg_yield_per_durability = total_yield as f64 / total_durability as f64;
        }

        results
    }

    pub fn run_all(
        &self,
        strategies: &[&dyn Strategy],
        discipline: &dyn DisciplineDriver,
    ) -> Vec<StrategyResults> {
        let mut strategy_results = Vec::new();
        for strategy in strategies {
            eprintln!(
                "Running {} strategy ({} games)...",
                strategy.name(),
                self.config.games_per_strategy
            );
            let result = self.run_strategy(*strategy, discipline);
            eprintln!(
                "  Win rate: {:.1}% ({}/{}), avg max streak: {:.1}, yield/durability: {:.3}",
                result.win_rate() * 100.0,
                result.wins,
                result.total_encounters,
                result.avg_max_win_streak,
                result.avg_yield_per_durability
            );
            strategy_results.push(result);
        }
        strategy_results
    }
}
