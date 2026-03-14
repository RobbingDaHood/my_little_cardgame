use super::helpers::*;
use my_little_cardgame::rocket_initialize;
use rocket::http::Status;
use rocket::local::blocking::Client;

/// Test: GET /actions/possible returns valid actions at each game state.
#[test]
fn scenario_possible_actions_endpoint() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Before new game: only NewGame should be available
    let actions = get_json(&client, "/actions/possible");
    let action_types: Vec<&str> = actions
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|a| a.get("action_type").and_then(|v| v.as_str()))
        .collect();
    assert!(
        action_types.contains(&"NewGame"),
        "NewGame should always be available"
    );

    // Start a new game
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    // After NewGame: should have EncounterPickEncounter
    let actions = get_json(&client, "/actions/possible");
    let action_types: Vec<&str> = actions
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|a| a.get("action_type").and_then(|v| v.as_str()))
        .collect();
    assert!(
        action_types.contains(&"EncounterPickEncounter"),
        "Should be able to pick encounter after new game. Actions: {:?}",
        action_types
    );

    // EncounterPickEncounter should have a card_id field (placeholder)
    let pick_action = actions
        .as_array()
        .unwrap()
        .iter()
        .find(|a| {
            a.get("action_type")
                .and_then(|v| v.as_str())
                .map(|s| s == "EncounterPickEncounter")
                .unwrap_or(false)
        })
        .unwrap();
    assert!(
        pick_action.get("card_id").is_some(),
        "EncounterPickEncounter should expose card_id field"
    );

    // Use library/cards to find a valid encounter card to pick
    let cards = get_json(&client, "/library/cards?location=Hand&card_kind=Encounter");
    let first_encounter_id = cards.as_array().unwrap()[0]
        .get("id")
        .unwrap()
        .as_u64()
        .unwrap();

    // Pick the encounter
    let (status, _) = post_action(
        &client,
        &format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            first_encounter_id
        ),
    );
    assert_eq!(status, Status::Created);

    // In encounter: should have EncounterPlayCard
    let actions = get_json(&client, "/actions/possible");
    let action_types: Vec<&str> = actions
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|a| a.get("action_type").and_then(|v| v.as_str()))
        .collect();
    assert!(
        action_types.contains(&"EncounterPlayCard"),
        "Should be able to play cards in encounter. Actions: {:?}",
        action_types
    );

    // EncounterPlayCard should have a card_id field (placeholder)
    let play_action = actions
        .as_array()
        .unwrap()
        .iter()
        .find(|a| {
            a.get("action_type")
                .and_then(|v| v.as_str())
                .map(|s| s == "EncounterPlayCard")
                .unwrap_or(false)
        })
        .unwrap();
    assert!(
        play_action.get("card_id").is_some(),
        "EncounterPlayCard should expose card_id field"
    );
}

/// Test: GET /library/card-effects returns player and enemy effects.
#[test]
fn scenario_card_effects_endpoint() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    let effects = get_json(&client, "/library/card-effects");
    let player_effects = effects.get("player_effects").and_then(|v| v.as_array());
    let enemy_effects = effects.get("enemy_effects").and_then(|v| v.as_array());
    assert!(
        player_effects.is_some(),
        "Response should have player_effects"
    );
    assert!(
        enemy_effects.is_some(),
        "Response should have enemy_effects"
    );
    assert!(
        !player_effects.unwrap().is_empty(),
        "Should have player card effects"
    );
    assert!(
        !enemy_effects.unwrap().is_empty(),
        "Should have enemy card effects"
    );
    // Each effect should have an id and a card
    let first = &player_effects.unwrap()[0];
    assert!(first.get("id").is_some(), "Effect should have an id");
}

/// Test: GET /library/cards with various filters
#[test]
fn scenario_library_cards_filters() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    // Test card_kind filters
    for kind in &[
        "Attack",
        "Defence",
        "Resource",
        "Mining",
        "Herbalism",
        "Woodcutting",
        "Fishing",
        "Rest",
        "Encounter",
        "PlayerCardEffect",
        "EnemyCardEffect",
        "Crafting",
    ] {
        let cards = get_json(&client, &format!("/library/cards?card_kind={}", kind));
        let arr = cards.as_array().unwrap();
        assert!(!arr.is_empty(), "Should have cards of kind {}", kind);
    }

    // Test location filters
    for location in &["Library", "Deck", "Hand", "Discard"] {
        let cards = get_json(&client, &format!("/library/cards?location={}", location));
        // Just ensure the endpoint doesn't error
        assert!(
            cards.as_array().is_some(),
            "Should return array for location {}",
            location
        );
    }

    // Combined filter
    let cards = get_json(&client, "/library/cards?location=Hand&card_kind=Attack");
    let arr = cards.as_array().unwrap();
    assert!(!arr.is_empty(), "Should have Attack cards in hand");
}

/// Test: POST /action with invalid JSON returns error
#[test]
fn scenario_invalid_action_returns_error() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Invalid action type
    let (status, _) = post_action(&client, r#"{"action_type":"FakeAction"}"#);
    assert_ne!(status, Status::Created, "Invalid action should not succeed");

    // Encounter action before game start
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterPickEncounter","card_id":0}"#,
    );
    assert_ne!(
        status,
        Status::Created,
        "Picking encounter before new game should fail"
    );
}

/// Test: GET /actions/log returns the action log
#[test]
fn scenario_actions_log_endpoint() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Log before game should have entries field
    let log = get_json(&client, "/actions/log");
    assert!(
        log.get("entries").and_then(|v| v.as_array()).is_some(),
        "Log should have entries array"
    );

    // Start a game and check log grows
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    let log = get_json(&client, "/actions/log");
    let entries = log.get("entries").and_then(|v| v.as_array()).unwrap();
    assert!(!entries.is_empty(), "Log should have entries after NewGame");

    // Each entry should have a seq and payload
    let first = &entries[0];
    assert!(first.get("seq").is_some(), "Entry should have seq");
    assert!(first.get("payload").is_some(), "Entry should have payload");

    // Test from_seq and limit filters
    let log_filtered = get_json(&client, "/actions/log?from_seq=0&limit=1");
    let filtered_entries = log_filtered
        .get("entries")
        .and_then(|v| v.as_array())
        .unwrap();
    assert!(
        filtered_entries.len() <= 1,
        "Limit should restrict entries count"
    );
    assert!(
        log_filtered.get("limit").is_some(),
        "Response should include limit"
    );
}

/// Test: Possible actions during scouting phase
#[test]
fn scenario_possible_actions_during_scouting() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    // Pick an encounter and play through combat to reach scouting
    let cards = get_json(&client, "/library/cards?location=Hand&card_kind=Encounter");
    let enc_id = cards.as_array().unwrap()[0]
        .get("id")
        .unwrap()
        .as_u64()
        .unwrap();
    let (status, _) = post_action(
        &client,
        &format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            enc_id
        ),
    );
    assert_eq!(status, Status::Created);

    // Play cards until combat concludes (need to reach scouting)
    loop {
        let enc = get_json(&client, "/encounter");
        if enc.is_null() || enc.get("state").is_none() {
            break;
        }

        let actions = get_json(&client, "/actions/possible");
        let action_types: Vec<&str> = actions
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|a| a.get("action_type").and_then(|v| v.as_str()))
            .collect();

        if action_types.contains(&"EncounterApplyScouting") {
            // We're in scouting phase — verify the action shape
            let scouting_action = actions
                .as_array()
                .unwrap()
                .iter()
                .find(|a| {
                    a.get("action_type")
                        .and_then(|v| v.as_str())
                        .map(|s| s == "EncounterApplyScouting")
                        .unwrap_or(false)
                })
                .unwrap();
            assert!(
                scouting_action.get("card_ids").is_some(),
                "EncounterApplyScouting should have card_ids field"
            );
            break;
        }

        if action_types.contains(&"EncounterPlayCard") {
            let hand = get_json(&client, "/library/cards?location=Hand&card_kind=Attack");
            if let Some(arr) = hand.as_array() {
                if let Some(card) = arr.first() {
                    let card_id = card.get("id").unwrap().as_u64().unwrap();
                    let _ = post_action(
                        &client,
                        &format!(
                            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                            card_id
                        ),
                    );
                    continue;
                }
            }
            // Try defence cards
            let hand = get_json(&client, "/library/cards?location=Hand&card_kind=Defence");
            if let Some(arr) = hand.as_array() {
                if let Some(card) = arr.first() {
                    let card_id = card.get("id").unwrap().as_u64().unwrap();
                    let _ = post_action(
                        &client,
                        &format!(
                            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                            card_id
                        ),
                    );
                    continue;
                }
            }
            // Try resource cards
            let hand = get_json(&client, "/library/cards?location=Hand&card_kind=Resource");
            if let Some(arr) = hand.as_array() {
                if let Some(card) = arr.first() {
                    let card_id = card.get("id").unwrap().as_u64().unwrap();
                    let _ = post_action(
                        &client,
                        &format!(
                            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                            card_id
                        ),
                    );
                    continue;
                }
            }
            break;
        }

        if action_types.contains(&"EncounterConcludeEncounter") {
            let _ = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
            continue;
        }
        break;
    }
}

/// Test: Possible actions during crafting encounter
#[test]
fn scenario_possible_actions_during_crafting() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    // Find crafting encounter
    let cards = get_json(&client, "/library/cards?location=Hand&card_kind=Encounter");
    let arr = cards.as_array().unwrap();
    for card in arr {
        let enc_id = card.get("id").unwrap().as_u64().unwrap();
        let kind = card.get("kind").unwrap();
        if let Some(ek) = kind.get("encounter_kind") {
            if ek.get("Crafting").is_some() || format!("{:?}", ek).contains("Crafting") {
                let (status, _) = post_action(
                    &client,
                    &format!(
                        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
                        enc_id
                    ),
                );
                if status == Status::Created {
                    let actions = get_json(&client, "/actions/possible");
                    let action_types: Vec<&str> = actions
                        .as_array()
                        .unwrap()
                        .iter()
                        .filter_map(|a| a.get("action_type").and_then(|v| v.as_str()))
                        .collect();
                    // In crafting encounter, should have craft-specific actions
                    assert!(
                        action_types.contains(&"EncounterCraftSwap")
                            || action_types.contains(&"EncounterAbort"),
                        "Should have crafting or abort actions. Actions: {:?}",
                        action_types
                    );
                    return;
                }
            }
        }
    }
    // If no crafting encounter in hand, that's OK
}

#[test]
fn scenario_max_handsize_tokens_initialized() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":500}"#);

    // Verify max handsize tokens are initialized to 5
    for token_name in &[
        "AttackMaxHand",
        "DefenceMaxHand",
        "ResourceMaxHand",
        "MiningMaxHand",
        "HerbalismMaxHand",
        "WoodcuttingMaxHand",
        "FishingMaxHand",
    ] {
        let val = player_token(&client, token_name);
        assert_eq!(
            val, 5,
            "{} should be initialized to 5, got {}",
            token_name, val
        );
    }
}
