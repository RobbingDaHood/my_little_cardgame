use my_little_cardgame::{
    area_deck::{AreaDeck, ScoutingParams},
    library::types::ActionPayload,
};

#[test]
fn test_area_deck_library_card_refs() {
    let mut area = AreaDeck::new("area_1".to_string());
    area.add_encounter(0);
    area.add_encounter(3);

    assert!(area.contains(0));
    assert!(area.contains(3));
    assert!(!area.contains(99));
    assert_eq!(area.encounter_card_ids.len(), 2);
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
