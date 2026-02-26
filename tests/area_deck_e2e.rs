use my_little_cardgame::{
    area_deck::{AreaDeck, ScoutingParams},
    library::GameState,
};
use rocket::futures::lock::Mutex;
use std::sync::Arc;

#[tokio::test]
async fn test_area_deck_with_library_card_refs() {
    let gs = Arc::new(Mutex::new(GameState::new()));

    // Create area deck referencing Library card indices
    let mut area = AreaDeck::new("forest_area".to_string());
    // Library index 3 = gnome CombatEncounter card
    area.add_encounter(3);
    area.add_encounter(3);
    area.draw_to_hand(2);

    assert_eq!(area.encounter_card_ids().len(), 2);
    assert!(area.contains(3));

    // Record draw action in action log
    let gs_lock = gs.lock().await;
    let payload = my_little_cardgame::library::types::ActionPayload::DrawEncounter {
        encounter_id: "3".to_string(),
    };
    let entry = gs_lock.append_action("DrawEncounter", payload);
    drop(gs_lock);

    assert!(matches!(
        entry.payload,
        my_little_cardgame::library::types::ActionPayload::DrawEncounter { .. }
    ));

    // Record scouting decision
    let gs_lock = gs.lock().await;
    let payload =
        my_little_cardgame::library::types::ActionPayload::ApplyScouting { card_ids: vec![3] };
    let entry = gs_lock.append_action("ApplyScouting", payload);
    drop(gs_lock);

    assert!(matches!(
        entry.payload,
        my_little_cardgame::library::types::ActionPayload::ApplyScouting { .. }
    ));
}

#[tokio::test]
async fn test_deterministic_replay_from_seed() {
    let params = ScoutingParams::new(2).with_affix_bias(vec!["poisoned".to_string()]);

    let base_seed = 999u64;
    let seed1 = params.apply_to_seed(base_seed);
    let seed2 = params.apply_to_seed(base_seed);

    // Identical seeds should produce identical results
    assert_eq!(seed1, seed2);
}

#[test]
fn test_scouting_biases_different_seeds() {
    let params_fire = ScoutingParams::new(2).with_affix_bias(vec!["fire".to_string()]);
    let params_poison = ScoutingParams::new(2).with_affix_bias(vec!["poison".to_string()]);

    let base_seed = 111u64;
    let seed_fire = params_fire.apply_to_seed(base_seed);
    let seed_poison = params_poison.apply_to_seed(base_seed);

    // Different biases should result in different seeds
    assert_ne!(seed_fire, seed_poison);
}

#[test]
fn test_area_deck_contains_card_ids() {
    let mut area = AreaDeck::new("test_area".to_string());
    area.add_encounter(0);
    area.add_encounter(3);
    area.draw_to_hand(2);

    assert!(area.contains(0));
    assert!(area.contains(3));
    assert!(!area.contains(99));
    assert_eq!(area.encounter_card_ids().len(), 2);
}
