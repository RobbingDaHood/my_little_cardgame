use my_little_cardgame::{
    area_deck::{AreaDeck, Encounter, EncounterState, ScoutingParams},
    library::GameState,
};
use rocket::futures::lock::Mutex;
use std::sync::Arc;

#[tokio::test]
async fn test_full_area_deck_flow_with_action_log() {
    // Initialize game state with action log
    let gs = Arc::new(Mutex::new(GameState::new()));

    // Create area deck with initial encounters
    let mut area = AreaDeck::new("forest_area".to_string());

    let enc1 = Encounter::new("enc_1".to_string(), "Combat".to_string()).with_entry_cost(10);
    let enc2 = Encounter::new("enc_2".to_string(), "Combat".to_string()).with_entry_cost(20);

    area.add_encounter(enc1);
    area.add_encounter(enc2);

    // Verify initial state
    assert_eq!(area.encounters.len(), 2);
    assert_eq!(area.get_available_encounters().len(), 2);

    // Draw first encounter (make it active)
    let drawn = area.draw_encounter("enc_1").unwrap();
    assert_eq!(drawn.state, EncounterState::Active);

    // Record draw action in action log
    let gs_lock = gs.lock().await;
    let payload = my_little_cardgame::library::types::ActionPayload::DrawEncounter {
        area_id: "forest_area".to_string(),
        encounter_id: "enc_1".to_string(),
        reason: Some("Player selected encounter".to_string()),
    };
    let entry = gs_lock.append_action("DrawEncounter", payload);
    drop(gs_lock);

    // Verify action was recorded
    assert_eq!(entry.action_type, "DrawEncounter");

    // Resolve the encounter
    area.resolve_encounter("enc_1").unwrap();
    assert_eq!(
        area.get_encounter("enc_1").unwrap().state,
        EncounterState::Resolved
    );

    // Generate replacement with scouting bias
    let scouting = ScoutingParams::new(2).with_affix_bias(vec!["blessed".to_string()]);

    let replacement_seed = scouting.apply_to_seed(42);
    let mut new_encounter = area.generate_encounter("Combat".to_string(), replacement_seed);
    new_encounter = new_encounter.with_entry_cost(15);

    // Replace resolved encounter (this puts the new encounter in place of enc_1)
    let replacement = area.replace_encounter("enc_1", new_encounter).unwrap();

    // Record replacement in action log
    let gs_lock = gs.lock().await;
    let payload = my_little_cardgame::library::types::ActionPayload::ReplaceEncounter {
        area_id: "forest_area".to_string(),
        old_encounter_id: "enc_1".to_string(),
        new_encounter_id: replacement.id.clone(),
        affixes_applied: replacement.affixes.clone(),
        reason: Some("Encounter resolved, replaced with new".to_string()),
    };
    let entry = gs_lock.append_action("ReplaceEncounter", payload);
    drop(gs_lock);

    assert_eq!(entry.action_type, "ReplaceEncounter");

    // Verify replacement state
    assert_eq!(replacement.state, EncounterState::Available);
    assert!(replacement.entry_cost.is_some());

    // Record scouting decision
    let gs_lock = gs.lock().await;
    let payload = my_little_cardgame::library::types::ActionPayload::ApplyScouting {
        area_id: "forest_area".to_string(),
        parameters: "preview_count=2,affix_bias=blessed".to_string(),
        reason: Some("Scouting applied to next replacement".to_string()),
    };
    let entry = gs_lock.append_action("ApplyScouting", payload);
    drop(gs_lock);

    assert_eq!(entry.action_type, "ApplyScouting");

    // Verify final area state - should still have 2 encounters (enc_2 and the replacement)
    assert_eq!(area.encounters.len(), 2);
    let available = area.get_available_encounters();
    assert_eq!(available.len(), 2); // enc_2 and the new encounter
}

#[tokio::test]
async fn test_deterministic_replay_from_seed() {
    // This test verifies that with the same seed + scouting params,
    // we get the same encounter replacement

    let params = ScoutingParams::new(2).with_affix_bias(vec!["poisoned".to_string()]);

    let base_seed = 999u64;
    let seed1 = params.apply_to_seed(base_seed);
    let seed2 = params.apply_to_seed(base_seed);

    // Identical seeds should produce identical results
    assert_eq!(seed1, seed2);

    let mut area1 = AreaDeck::new("area1".to_string());
    let mut area2 = AreaDeck::new("area2".to_string());

    let enc1 = area1.generate_encounter("Combat".to_string(), seed1);
    let enc2 = area2.generate_encounter("Combat".to_string(), seed2);

    // Should generate identical affixes
    assert_eq!(enc1.affixes, enc2.affixes);
}

#[test]
fn test_scouting_biases_different_seeds() {
    // Different scouting parameters should produce different seeds
    let params_fire = ScoutingParams::new(2).with_affix_bias(vec!["fire".to_string()]);
    let params_poison = ScoutingParams::new(2).with_affix_bias(vec!["poison".to_string()]);

    let base_seed = 111u64;
    let seed_fire = params_fire.apply_to_seed(base_seed);
    let seed_poison = params_poison.apply_to_seed(base_seed);

    // Different biases should result in different seeds
    assert_ne!(seed_fire, seed_poison);
}

#[test]
fn test_area_deck_reward_binding() {
    let encounter = Encounter::new("encounter_boss".to_string(), "Combat".to_string())
        .with_reward_deck("dragon_reward_deck".to_string())
        .with_entry_cost(100);

    assert_eq!(
        encounter.reward_deck_id,
        Some("dragon_reward_deck".to_string())
    );
    assert_eq!(encounter.entry_cost, Some(100));
}

#[test]
fn test_multiple_replacements_increment_ids() {
    let mut area = AreaDeck::new("test_area".to_string());

    // Generate and add 5 encounters
    let mut encounters = vec![];
    for i in 0..5 {
        let enc = area.generate_encounter("Combat".to_string(), i as u64 * 100);
        encounters.push(enc.id.clone());
        area.add_encounter(enc);
    }

    // Verify IDs are sequential
    for (i, id) in encounters.iter().enumerate() {
        assert_eq!(*id, format!("encounter_{}", i));
    }
}
