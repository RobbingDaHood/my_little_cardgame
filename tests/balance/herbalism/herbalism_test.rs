use crate::game_driver::{
    get_json, get_possible_actions, get_snapshot, post_action, DisciplineDriver,
};
use crate::herbalism::driver::{
    find_herbalism_encounter, get_herbalism_encounter_ids, HerbalismDisciplineDriver,
};
use crate::herbalism::output::HerbalismSimulationReport;
use crate::herbalism::strategies::conservative::ConservativeStrategy;
use crate::herbalism::strategies::greedy::GreedyStrategy;
use crate::herbalism::strategies::random::RandomStrategy;
use crate::herbalism::strategies::tactician::TacticianStrategy;
use crate::runner::{SimulationConfig, SimulationRunner};

/// Diagnostic: trace one full game to understand herbalism encounter flow
#[test]
fn herbalism_greedy_diagnostic() {
    let client = rocket::local::blocking::Client::tracked(my_little_cardgame::rocket_initialize())
        .expect("valid rocket");

    let new_game = serde_json::json!({"action_type": "NewGame", "seed": 42_u64});
    post_action(&client, &new_game);

    let discipline = HerbalismDisciplineDriver;

    let herb_ids = get_herbalism_encounter_ids(&client);
    eprintln!("Herbalism encounter IDs: {:?}", herb_ids);

    if let Some(id) = find_herbalism_encounter(&client) {
        let pick = serde_json::json!({"action_type": "EncounterPickEncounter", "card_id": id});
        eprintln!("Picking encounter card {}", id);
        post_action(&client, &pick);

        let greedy = GreedyStrategy::new();
        let rounds = discipline.play_encounter(&client, &greedy, 200);
        eprintln!("Encounter played: {} rounds", rounds);

        let snapshot = get_snapshot(&client);
        eprintln!(
            "After encounter - has encounter: {}",
            snapshot.encounter.is_some()
        );
        eprintln!(
            "  durability: {}, plant tokens: {}",
            snapshot.herbalism_durability(),
            snapshot.plant_tokens()
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

/// Runs herbalism simulation across 4 strategies and asserts:
/// - Yield per durability within exploration band
/// - Tactician should achieve higher yield/durability than simple strategies
#[test]
fn herbalism_balance_simulation() {
    let config = SimulationConfig {
        games_per_strategy: 5,
        encounters_per_game: 20,
        base_seed: 42,
        max_actions_per_encounter: 200,
    };

    let random = RandomStrategy::new(7777);
    let greedy = GreedyStrategy::new();
    let conservative = ConservativeStrategy::new();
    let tactician = TacticianStrategy::new();

    let strategies: Vec<&dyn crate::strategies::Strategy> =
        vec![&random, &greedy, &conservative, &tactician];

    let discipline = HerbalismDisciplineDriver;
    let runner = SimulationRunner::new(config.clone());
    let results = runner.run_all(&strategies, &discipline);
    let report = HerbalismSimulationReport::from_results(&config, results);

    println!("{}", report.to_json());

    for strat in &report.strategies {
        assert!(
            strat.herbalism.yield_pass,
            "Strategy '{}' yield/durability {:.3} outside target [{:.1}–{:.1}] (total yield: {}, total durability: {})",
            strat.name,
            strat.herbalism.avg_yield_per_durability,
            strat.herbalism.yield_target_min,
            strat.herbalism.yield_target_max,
            strat.herbalism.total_yield,
            strat.herbalism.total_durability,
        );
    }

    assert!(
        report.all_assertions_passed,
        "Not all herbalism balance assertions passed"
    );
}
