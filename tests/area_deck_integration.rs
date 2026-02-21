use my_little_cardgame::{
    area_deck::{AreaDeck, Encounter, ScoutingParams},
    library::types::ActionPayload,
};

#[test]
fn test_area_deck_draw_and_replace() {
    let mut area = AreaDeck::new("area_1".to_string(), "Forest".to_string());

    let enc1 = Encounter::new("enc_1".to_string(), "Combat".to_string());
    let enc2 = Encounter::new("enc_2".to_string(), "Combat".to_string());

    area.add_encounter(enc1);
    area.add_encounter(enc2);

    let drawn = area.draw_encounter("enc_1").unwrap();
    assert_eq!(
        drawn.state,
        my_little_cardgame::area_deck::EncounterState::Active
    );

    area.resolve_encounter("enc_1").unwrap();

    let mut new_encounter = Encounter::new("enc_3".to_string(), "Combat".to_string());
    new_encounter = new_encounter.with_affixes(vec!["fireresistant".to_string()]);

    let replacement = area.replace_encounter("enc_1", new_encounter).unwrap();
    assert_eq!(replacement.id, "enc_3");
    assert_eq!(replacement.affixes.len(), 1);
}

#[test]
fn test_deterministic_encounter_generation() {
    let mut area1 = AreaDeck::new("area_1".to_string(), "Forest".to_string());
    let mut area2 = AreaDeck::new("area_2".to_string(), "Forest".to_string());

    let enc1 = area1.generate_encounter("Combat".to_string(), 42);
    let enc2 = area2.generate_encounter("Combat".to_string(), 42);

    assert_eq!(enc1.affixes, enc2.affixes);
}

#[test]
fn test_scouting_influences_replacement_seed() {
    let params =
        ScoutingParams::new(3).with_affix_bias(vec!["fire".to_string(), "poison".to_string()]);

    let base_seed = 1000u64;
    let modified_seed = params.apply_to_seed(base_seed);

    assert_ne!(base_seed, modified_seed);
}

#[test]
fn test_encounter_with_entry_cost() {
    let encounter = Encounter::new("enc_1".to_string(), "Combat".to_string()).with_entry_cost(50);

    assert_eq!(encounter.entry_cost, Some(50));
}

#[test]
fn test_encounter_with_reward_deck() {
    let encounter = Encounter::new("enc_1".to_string(), "Combat".to_string())
        .with_reward_deck("reward_deck_1".to_string());

    assert_eq!(encounter.reward_deck_id, Some("reward_deck_1".to_string()));
}

#[test]
fn test_get_available_encounters() {
    let mut area = AreaDeck::new("area_1".to_string(), "Forest".to_string());

    let enc1 = Encounter::new("enc_1".to_string(), "Combat".to_string());
    let enc2 = Encounter::new("enc_2".to_string(), "Combat".to_string());

    area.add_encounter(enc1);
    area.add_encounter(enc2);

    area.draw_encounter("enc_1").unwrap();
    area.resolve_encounter("enc_1").unwrap();

    let available = area.get_available_encounters();
    assert_eq!(available.len(), 1);
    assert_eq!(available[0].id, "enc_2");
}

#[test]
fn test_cannot_replace_non_resolved_encounter() {
    let mut area = AreaDeck::new("area_1".to_string(), "Forest".to_string());

    let enc1 = Encounter::new("enc_1".to_string(), "Combat".to_string());
    let new_enc = Encounter::new("enc_2".to_string(), "Combat".to_string());

    area.add_encounter(enc1);

    let result = area.replace_encounter("enc_1", new_enc);
    assert!(result.is_err());
}

#[test]
fn test_multiple_scouting_params_produce_same_seed() {
    let params1 = ScoutingParams::new(3).with_affix_bias(vec!["fire".to_string()]);
    let params2 = ScoutingParams::new(3).with_affix_bias(vec!["fire".to_string()]);

    let base_seed = 54321u64;
    assert_eq!(
        params1.apply_to_seed(base_seed),
        params2.apply_to_seed(base_seed)
    );
}

#[test]
fn test_area_deck_encounter_id_increments() {
    let mut area = AreaDeck::new("area_1".to_string(), "Forest".to_string());

    let enc1 = area.generate_encounter("Combat".to_string(), 100);
    let enc2 = area.generate_encounter("Combat".to_string(), 200);

    assert_eq!(enc1.id, "encounter_0");
    assert_eq!(enc2.id, "encounter_1");
}

#[test]
fn test_action_payload_draw_encounter() {
    let payload = ActionPayload::DrawEncounter {
        area_id: "area_1".to_string(),
        encounter_id: "enc_1".to_string(),
        reason: Some("Test draw".to_string()),
    };

    match payload {
        ActionPayload::DrawEncounter {
            area_id,
            encounter_id,
            reason,
        } => {
            assert_eq!(area_id, "area_1");
            assert_eq!(encounter_id, "enc_1");
            assert_eq!(reason, Some("Test draw".to_string()));
        }
        _ => panic!("Expected DrawEncounter payload"),
    }
}

#[test]
fn test_action_payload_replace_encounter() {
    let payload = ActionPayload::ReplaceEncounter {
        area_id: "area_1".to_string(),
        old_encounter_id: "enc_1".to_string(),
        new_encounter_id: "enc_2".to_string(),
        affixes_applied: vec!["fire".to_string()],
        reason: Some("Replacement".to_string()),
    };

    match payload {
        ActionPayload::ReplaceEncounter {
            area_id,
            old_encounter_id,
            new_encounter_id,
            affixes_applied,
            reason,
        } => {
            assert_eq!(area_id, "area_1");
            assert_eq!(old_encounter_id, "enc_1");
            assert_eq!(new_encounter_id, "enc_2");
            assert_eq!(affixes_applied.len(), 1);
            assert_eq!(reason, Some("Replacement".to_string()));
        }
        _ => panic!("Expected ReplaceEncounter payload"),
    }
}
