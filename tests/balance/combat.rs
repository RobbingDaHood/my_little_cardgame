use crate::output::SimulationReport;
use crate::runner::{SimulationConfig, SimulationRunner};
use crate::strategies::conservative::ConservativeStrategy;
use crate::strategies::greedy::GreedyStrategy;
use crate::strategies::random::RandomStrategy;

/// Combat balance simulation test.
///
/// Runs 1000 games × 3 strategies, asserts combat win rates
/// against vision.md targets (±10%):
/// - Random: 20–40%
/// - Greedy: 40–60%
/// - Conservative: 30–50%
#[test]
fn combat_balance_simulation() {
    let config = SimulationConfig {
        games_per_strategy: 1000,
        encounters_per_game: 20,
        base_seed: 42,
    };

    let random = RandomStrategy::new(7777);
    let greedy = GreedyStrategy::new();
    let conservative = ConservativeStrategy::new();

    let strategies: Vec<&dyn crate::strategies::Strategy> = vec![&random, &greedy, &conservative];

    let runner = SimulationRunner::new(config);
    let report: SimulationReport = runner.run_all(&strategies);

    // Output JSON to stdout for piping to jq/nushell
    println!("{}", report.to_json());

    // Assert all win rates within targets
    for strat in &report.strategies {
        assert!(
            strat.combat.pass,
            "Strategy '{}' combat win rate {:.1}% outside target range [{:.0}%–{:.0}%]",
            strat.name,
            strat.combat.win_rate * 100.0,
            strat.combat.target_min * 100.0,
            strat.combat.target_max * 100.0,
        );
    }

    assert!(
        report.all_assertions_passed,
        "Not all combat balance assertions passed"
    );
}
