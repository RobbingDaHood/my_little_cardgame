use crate::fishing::driver::FishingDisciplineDriver;
use crate::fishing::output::FishingSimulationReport;
use crate::fishing::strategies::conservative::ConservativeFishingStrategy;
use crate::fishing::strategies::greedy::GreedyFishingStrategy;
use crate::fishing::strategies::random::RandomFishingStrategy;
use crate::fishing::strategies::tactician::TacticianFishingStrategy;
use crate::runner::{SimulationConfig, SimulationRunner};

#[test]
fn fishing_balance_simulation() {
    let config = SimulationConfig {
        games_per_strategy: 30,
        encounters_per_game: 20,
        base_seed: 42,
        max_actions_per_encounter: 200,
    };

    let random = RandomFishingStrategy::new(7777);
    let greedy = GreedyFishingStrategy;
    let conservative = ConservativeFishingStrategy;
    let tactician = TacticianFishingStrategy;

    let strategies: Vec<&dyn crate::strategies::Strategy> =
        vec![&random, &greedy, &conservative, &tactician];

    let discipline = FishingDisciplineDriver;
    let runner = SimulationRunner::new(config.clone());
    let results = runner.run_all(&strategies, &discipline);
    let report = FishingSimulationReport::from_results(&config, results);

    println!("{}", report.to_json());

    for strat in &report.strategies {
        assert!(
            strat.fishing.yield_pass,
            "Strategy '{}' yield/durability {:.3} outside target [{:.1}–{:.1}] (total yield: {}, total durability: {})",
            strat.name,
            strat.fishing.avg_yield_per_durability,
            strat.fishing.yield_target_min,
            strat.fishing.yield_target_max,
            strat.fishing.total_yield,
            strat.fishing.total_durability,
        );
    }

    assert!(
        report.all_assertions_passed,
        "Not all fishing balance assertions passed"
    );
}
