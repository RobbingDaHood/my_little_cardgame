use crate::runner::{SimulationConfig, SimulationRunner};
use crate::woodcutting::driver::WoodcuttingDisciplineDriver;
use crate::woodcutting::output::WoodcuttingSimulationReport;
use crate::woodcutting::strategies::conservative::ConservativeStrategy;
use crate::woodcutting::strategies::greedy::GreedyStrategy;
use crate::woodcutting::strategies::pattern_builder::PatternBuilderStrategy;
use crate::woodcutting::strategies::random::RandomStrategy;

#[test]
fn woodcutting_balance_simulation() {
    let config = SimulationConfig {
        games_per_strategy: 10,
        encounters_per_game: 50,
        base_seed: 42,
        max_actions_per_encounter: 200,
    }
    .with_env_overrides();

    let random = RandomStrategy::new(7777);
    let greedy = GreedyStrategy::new();
    let conservative = ConservativeStrategy::new();
    let pattern_builder = PatternBuilderStrategy::new();

    let strategies: Vec<&dyn crate::strategies::Strategy> =
        vec![&random, &greedy, &conservative, &pattern_builder];

    let discipline = WoodcuttingDisciplineDriver;
    let runner = SimulationRunner::new(config.clone());
    let results = runner.run_all(&strategies, &discipline);
    let report = WoodcuttingSimulationReport::from_results(&config, results);

    println!("{}", report.to_json());

    // Primary assertion: yield per durability within target band
    for strat in &report.strategies {
        assert!(
            strat.woodcutting.yield_pass,
            "Strategy '{}' yield/durability {:.3} outside target [{:.1}–{:.1}]",
            strat.name,
            strat.woodcutting.yield_per_durability,
            strat.woodcutting.target_min,
            strat.woodcutting.target_max,
        );
    }

    assert!(
        report.all_assertions_passed,
        "Not all woodcutting balance assertions passed"
    );
}
