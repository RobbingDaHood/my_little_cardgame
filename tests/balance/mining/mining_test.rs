use crate::game_driver::{
    get_json, get_possible_actions, get_snapshot, post_action, DisciplineDriver,
};
use crate::mining::driver::{
    find_mining_encounter, get_mining_encounter_ids, MiningDisciplineDriver,
};
use crate::mining::output::MiningSimulationReport;
use crate::mining::strategies::conservative::ConservativeStrategy;
use crate::mining::strategies::greedy::GreedyStrategy;
use crate::mining::strategies::random::RandomStrategy;
use crate::mining::strategies::tactician::TacticianStrategy;
use crate::runner::{SimulationConfig, SimulationRunner};

/// Diagnostic: trace one full game to understand encounter flow
#[test]
fn mining_greedy_diagnostic() {
    let client = rocket::local::blocking::Client::tracked(my_little_cardgame::rocket_initialize())
        .expect("valid rocket");

    let new_game = serde_json::json!({"action_type": "NewGame", "seed": 42_u64});
    post_action(&client, &new_game);

    let discipline = MiningDisciplineDriver;

    // Check initial state
    let mining_ids = get_mining_encounter_ids(&client);
    eprintln!("Mining encounter IDs: {:?}", mining_ids);

    // Pick a mining encounter
    if let Some(id) = find_mining_encounter(&client) {
        let pick = serde_json::json!({"action_type": "EncounterPickEncounter", "card_id": id});
        eprintln!("Picking encounter card {}", id);
        post_action(&client, &pick);

        // Play using actual encounter function with tactician
        let tactician = TacticianStrategy::new();
        let rounds = discipline.play_encounter(&client, &tactician, 200);
        eprintln!("Encounter played: {} rounds", rounds);

        // Check state after encounter
        let snapshot = get_snapshot(&client);
        eprintln!(
            "After encounter - has encounter: {}",
            snapshot.encounter.is_some()
        );
        if let Some(enc) = &snapshot.encounter {
            eprintln!("  state_type: {:?}", enc.get("encounter_state_type"));
            eprintln!("  outcome: {:?}", enc.get("outcome"));
        }
        eprintln!(
            "  light: {}, yield: {}, durability: {}",
            snapshot.mining_light_level(),
            snapshot.mining_yield(),
            snapshot.mining_durability()
        );

        let results_after = get_json(&client, "/encounter/results");
        eprintln!("Results after: {}", results_after);

        let possible2 = get_possible_actions(&client);
        let action_types2: Vec<String> = possible2
            .iter()
            .filter_map(|a| {
                a.get("action_type")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .collect();
        eprintln!("Possible actions after: {:?}", action_types2);
    }
}

/// Runs mining simulation across 4 strategies and asserts:
/// - Yield per durability within observed exploration band
/// - Tactician should achieve higher yield/durability than simple strategies
#[test]
fn mining_balance_simulation() {
    let config = SimulationConfig {
        games_per_strategy: 3,
        encounters_per_game: 20,
        base_seed: 42,
        max_actions_per_encounter: 1000,
    };

    let random = RandomStrategy::new(7777);
    let greedy = GreedyStrategy::new();
    let conservative = ConservativeStrategy::new();
    let tactician = TacticianStrategy::new();

    let strategies: Vec<&dyn crate::strategies::Strategy> =
        vec![&random, &greedy, &conservative, &tactician];

    let discipline = MiningDisciplineDriver;
    let runner = SimulationRunner::new(config.clone());
    let results = runner.run_all(&strategies, &discipline);
    let report = MiningSimulationReport::from_results(&config, results);

    println!("{}", report.to_json());

    // Primary assertion: yield per durability within target band
    for strat in &report.strategies {
        assert!(
            strat.mining.yield_pass,
            "Strategy '{}' yield/durability {:.3} outside target [{:.1}–{:.1}] (total yield: {}, total durability: {})",
            strat.name,
            strat.mining.avg_yield_per_durability,
            strat.mining.yield_target_min,
            strat.mining.yield_target_max,
            strat.mining.total_yield,
            strat.mining.total_durability,
        );
    }

    assert!(
        report.all_assertions_passed,
        "Not all mining balance assertions passed"
    );
}
