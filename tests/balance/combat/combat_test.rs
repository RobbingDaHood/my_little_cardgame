use crate::combat::strategies::conservative::ConservativeStrategy;
use crate::combat::strategies::greedy::GreedyStrategy;
use crate::combat::strategies::random::RandomStrategy;
use crate::combat::strategies::tactician_conservative::TacticianConservativeStrategy;
use crate::combat::strategies::tactician_greedy::TacticianGreedyStrategy;
use crate::output::SimulationReport;
use crate::runner::{SimulationConfig, SimulationRunner};

/// Runs games across 5 strategies and asserts:
/// - Win streak targets based on overall_avg_streak: simple ~3-5, tactician ~10+
/// - Avg rounds per encounter ≥ 3
#[test]
fn combat_balance_simulation() {
    let config = SimulationConfig {
        games_per_strategy: 10,
        encounters_per_game: 50,
        base_seed: 42,
    };

    let random = RandomStrategy::new(7777);
    let greedy = GreedyStrategy::new();
    let conservative = ConservativeStrategy::new();
    let tactician_greedy = TacticianGreedyStrategy::new();
    let tactician_conservative = TacticianConservativeStrategy::new();

    let strategies: Vec<&dyn crate::strategies::Strategy> = vec![
        &random,
        &greedy,
        &conservative,
        &tactician_greedy,
        &tactician_conservative,
    ];

    let runner = SimulationRunner::new(config);
    let report: SimulationReport = runner.run_all(&strategies);

    // Output JSON to stdout for piping to jq/nushell
    println!("{}", report.to_json());

    // Assert streak targets and round duration for all strategies
    for strat in &report.strategies {
        assert!(
            strat.combat.streak_pass,
            "Strategy '{}' overall avg streak {:.1} outside target [{:.1}–{:.1}]",
            strat.name,
            strat.combat.overall_avg_streak,
            strat.combat.streak_target_min,
            strat.combat.streak_target_max,
        );

        assert!(
            strat.combat.rounds_pass,
            "Strategy '{}' avg rounds/encounter {:.1} is below minimum 3.0",
            strat.name, strat.combat.avg_rounds_per_encounter,
        );
    }

    assert!(
        report.all_assertions_passed,
        "Not all combat balance assertions passed"
    );
}
