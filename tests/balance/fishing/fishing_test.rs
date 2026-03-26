use super::output::{fishing_win_rate_targets, fishing_yield_targets};
use super::runner::{FishingSimulationConfig, FishingSimulationRunner};
use super::strategies::conservative::ConservativeFishingStrategy;
use super::strategies::greedy::GreedyFishingStrategy;
use super::strategies::random::RandomFishingStrategy;
use super::strategies::tactician::TacticianFishingStrategy;
use crate::strategies::Strategy;

#[test]
fn fishing_balance_simulation() {
    let strategies: Vec<Box<dyn Strategy>> = vec![
        Box::new(RandomFishingStrategy::new(99)),
        Box::new(GreedyFishingStrategy),
        Box::new(ConservativeFishingStrategy),
        Box::new(TacticianFishingStrategy),
    ];

    let config = FishingSimulationConfig {
        num_games: 10,
        max_encounters_per_game: 50,
        max_actions_per_encounter: 200,
        base_seed: 42,
    };

    let runner = FishingSimulationRunner {
        config,
        yield_targets: fishing_yield_targets(),
        win_rate_targets: fishing_win_rate_targets(),
    };

    let report = runner.run_all_strategies(&strategies);

    println!(
        "\n\nFishing Simulation Report:\n{}",
        serde_json::to_string_pretty(&report).unwrap_or_default()
    );

    assert!(
        report.all_pass,
        "Fishing balance simulation failed! Check yield_per_durability targets."
    );
}
