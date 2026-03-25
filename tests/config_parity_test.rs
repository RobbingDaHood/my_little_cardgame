//! Parity test: verify that runtime disk config loading produces identical results
//! to compile-time include_str!() loading. Gated behind --features simulation.
//!
//! This ensures the runtime loading mechanism cannot do anything a normal
//! server build cannot do — both paths produce the same Library state.

#![cfg(feature = "simulation")]

use my_little_cardgame::library::config_loader;
use rand_pcg::Lcg64Xsh32;

#[test]
fn runtime_config_loading_matches_compiled() {
    let seed: u64 = 42;

    // Load via compiled-in include_str!() (production path)
    let mut rng_compiled = Lcg64Xsh32::new(seed, 0);
    let lib_compiled = config_loader::load_library(&mut rng_compiled);
    let tokens_compiled = config_loader::load_token_balances();
    let rules_compiled = config_loader::load_game_rules();
    let combat_compiled = config_loader::load_combat_rules();
    let research_compiled = config_loader::load_research_rules();
    let crafting_compiled = config_loader::load_crafting_rules();
    let milestone_compiled = config_loader::load_milestone_rules();
    let scouting_compiled = config_loader::load_scouting_rules();
    let woodcutting_compiled = config_loader::load_woodcutting_patterns();

    // Load via runtime disk reading (simulation path)
    let mut rng_runtime = Lcg64Xsh32::new(seed, 0);
    let config_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/configurations");
    let lib_runtime = config_loader::load_library_from_disk(&mut rng_runtime, config_dir);
    let tokens_runtime = config_loader::load_token_balances_from_disk(config_dir);
    let rules_runtime = config_loader::load_game_rules_from_disk(config_dir);
    let combat_runtime = config_loader::load_combat_rules_from_disk(config_dir);
    let research_runtime = config_loader::load_research_rules_from_disk(config_dir);
    let crafting_runtime = config_loader::load_crafting_rules_from_disk(config_dir);
    let milestone_runtime = config_loader::load_milestone_rules_from_disk(config_dir);
    let scouting_runtime = config_loader::load_scouting_rules_from_disk(config_dir);
    let woodcutting_runtime = config_loader::load_woodcutting_patterns_from_disk(config_dir);

    // Compare token balances
    assert_eq!(
        tokens_compiled.len(),
        tokens_runtime.len(),
        "Token balance count mismatch"
    );
    for (token, value) in &tokens_compiled {
        let runtime_val = tokens_runtime.get(token);
        assert_eq!(
            Some(value),
            runtime_val,
            "Token {:?} mismatch: compiled={}, runtime={:?}",
            token,
            value,
            runtime_val
        );
    }

    // Compare game rules (general only)
    assert_eq!(
        format!("{:?}", rules_compiled),
        format!("{:?}", rules_runtime),
        "Game rules mismatch"
    );

    // Compare per-discipline configs
    assert_eq!(
        format!("{:?}", combat_compiled),
        format!("{:?}", combat_runtime),
        "Combat rules mismatch"
    );
    assert_eq!(
        format!("{:?}", research_compiled),
        format!("{:?}", research_runtime),
        "Research rules mismatch"
    );
    assert_eq!(
        format!("{:?}", crafting_compiled),
        format!("{:?}", crafting_runtime),
        "Crafting rules mismatch"
    );
    assert_eq!(
        format!("{:?}", milestone_compiled),
        format!("{:?}", milestone_runtime),
        "Milestone rules mismatch"
    );
    assert_eq!(
        format!("{:?}", scouting_compiled),
        format!("{:?}", scouting_runtime),
        "Scouting rules mismatch"
    );
    assert_eq!(
        format!("{:?}", woodcutting_compiled.len()),
        format!("{:?}", woodcutting_runtime.len()),
        "Woodcutting patterns count mismatch"
    );
    for (i, (c, r)) in woodcutting_compiled
        .iter()
        .zip(woodcutting_runtime.iter())
        .enumerate()
    {
        assert_eq!(
            format!("{:?}", c),
            format!("{:?}", r),
            "Woodcutting pattern {} mismatch",
            i
        );
    }

    // Compare libraries by iterating cards until get() returns None
    let mut i = 0;
    loop {
        let card_c = lib_compiled.get(i);
        let card_r = lib_runtime.get(i);
        match (card_c, card_r) {
            (Some(c), Some(r)) => {
                let c_json = serde_json::to_string(&c.kind).unwrap();
                let r_json = serde_json::to_string(&r.kind).unwrap();
                assert_eq!(c_json, r_json, "Card {} kind mismatch", i);
                assert_eq!(
                    format!("{:?}", c.counts),
                    format!("{:?}", r.counts),
                    "Card {} counts mismatch",
                    i
                );
                assert_eq!(
                    c.valid_discipline_types, r.valid_discipline_types,
                    "Card {} valid_discipline_types mismatch",
                    i
                );
            }
            (None, None) => break,
            _ => panic!(
                "Card count mismatch at index {}: compiled={}, runtime={}",
                i,
                card_c.is_some(),
                card_r.is_some()
            ),
        }
        i += 1;
    }
    assert!(i > 0, "Libraries should have at least one card");
}
