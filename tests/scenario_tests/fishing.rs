use super::helpers::*;
use my_little_cardgame::rocket_initialize;
use rocket::http::Status;
use rocket::local::blocking::Client;

const TOKENS: &str = include_str!("../configurations/tokens_default.json");
const FISHING_WIN: &str = include_str!("../configurations/fishing_win.json");
const FISHING_LOSS: &str = include_str!("../configurations/fishing_loss.json");

fn fishing_hand_card_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Fishing");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| c.get("id").and_then(|v| v.as_u64()).map(|v| v as usize))
        .collect()
}

fn fishing_encounter_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| {
            let kind = c.get("kind")?;
            let enc_kind = kind.get("encounter_kind")?;
            if enc_kind.get("encounter_type")?.as_str()? == "Fishing" {
                c.get("id")?.as_u64().map(|v| v as usize)
            } else {
                None
            }
        })
        .collect()
}

fn play_one_fishing_card(client: &Client) -> bool {
    let fc_ids = fishing_hand_card_ids(client);
    if fc_ids.is_empty() {
        return false;
    }
    let card_id = fc_ids[0];
    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        card_id
    );
    let (status, _) = post_action(client, &json);
    if status != Status::Created {
        return false;
    }
    let encounter = combat_state(client);
    encounter.get("outcome").and_then(|v| v.as_str()) == Some("Undecided")
}

#[test]
fn scenario_fishing_win_and_scout() {
    let client = create_test_client_from_json(42, TOKENS, &[("fishing", FISHING_WIN)]);

    let enc_ids = fishing_encounter_ids(&client);
    assert!(!enc_ids.is_empty(), "Should have fishing encounters");
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc_ids[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    let encounter = combat_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Fishing")
    );

    for _ in 0..50 {
        if !play_one_fishing_card(&client) {
            break;
        }
    }

    let result = combat_result(&client);
    assert_eq!(
        result.as_deref(),
        Some("PlayerWon"),
        "Should win with wide valid range"
    );

    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(status, Status::Created);

    let enc_after = encounter_hand_ids(&client);
    assert!(!enc_after.is_empty());
    let pick2 = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc_after[0]
    );
    let (status, _) = post_action(&client, &pick2);
    assert_eq!(status, Status::Created);
}

#[test]
fn scenario_fishing_loss_and_scout() {
    // fishing_loss.json: impossible valid_range (99999-99999), only 1 max_turn
    let client = create_test_client_from_json(42, TOKENS, &[("fishing", FISHING_LOSS)]);

    let enc_ids = fishing_encounter_ids(&client);
    assert!(!enc_ids.is_empty());
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc_ids[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    for _ in 0..50 {
        if !play_one_fishing_card(&client) {
            break;
        }
    }

    let result = combat_result(&client);
    assert_eq!(
        result.as_deref(),
        Some("PlayerLost"),
        "Should lose with impossible range"
    );

    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(status, Status::Created);

    let enc_after = encounter_hand_ids(&client);
    assert!(!enc_after.is_empty());
    let pick2 = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc_after[0]
    );
    let (status, _) = post_action(&client, &pick2);
    assert_eq!(status, Status::Created);
}

#[test]
fn scenario_fishing_encounter_full_loop() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created, "NewGame should succeed");

    let fc_enc = fishing_encounter_ids(&client);
    assert!(
        !fc_enc.is_empty(),
        "Should have fishing encounter cards in hand"
    );

    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        fc_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created, "PickEncounter should succeed");

    let encounter = combat_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Fishing"),
        "Encounter should be Fishing type"
    );
    assert_eq!(
        encounter.get("outcome").and_then(|v| v.as_str()),
        Some("Undecided"),
        "Fishing should be active"
    );
    assert_eq!(
        encounter.get("max_turns").and_then(|v| v.as_u64()),
        Some(8),
        "max_turns should be 8"
    );
    assert_eq!(
        encounter.get("win_turns_needed").and_then(|v| v.as_u64()),
        Some(4),
        "win_turns_needed should be 4"
    );
    assert_eq!(
        encounter.get("turns_won").and_then(|v| v.as_u64()),
        Some(0),
        "No turns won yet"
    );

    let durability = player_token(&client, "FishingDurability");
    assert_eq!(
        durability, 10000,
        "Player should start with 10000 fishing durability"
    );

    // Play cards until the encounter ends
    let mut total_turns = 0;
    loop {
        let still_going = play_one_fishing_card(&client);
        total_turns += 1;
        if !still_going {
            break;
        }
        assert!(total_turns < 50, "Fishing should end within 50 turns");
    }

    let last_outcome = combat_result(&client).unwrap_or_default();
    assert!(
        last_outcome == "PlayerWon" || last_outcome == "PlayerLost",
        "Fishing should end with PlayerWon or PlayerLost (got {})",
        last_outcome
    );

    if last_outcome == "PlayerWon" {
        let fish = player_token(&client, "Fish");
        assert!(
            fish > 0,
            "Player should have Fish after winning fishing (got {})",
            fish
        );
    }

    let final_durability = player_token(&client, "FishingDurability");
    assert!(
        final_durability < 10000,
        "Durability should decrease after fishing (got {})",
        final_durability
    );

    // Should be in Scouting phase now
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(
        status,
        Status::Created,
        "Should be able to scout after fishing"
    );

    // Play multiple encounters until durability runs out
    let mut total_encounters = 1;
    loop {
        let fc_enc = fishing_encounter_ids(&client);
        if fc_enc.is_empty() {
            break;
        }
        let pick_json = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            fc_enc[0]
        );
        let (status, _) = post_action(&client, &pick_json);
        assert_eq!(status, Status::Created, "PickEncounter should succeed");

        let mut round_turns = 0;
        loop {
            let still_going = play_one_fishing_card(&client);
            round_turns += 1;
            if !still_going {
                break;
            }
            assert!(round_turns < 50, "Round should end within 50 turns");
        }
        total_encounters += 1;

        let outcome = combat_result(&client).unwrap_or_default();
        if outcome == "PlayerLost" {
            break;
        }

        let (status, _) = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
        assert_eq!(status, Status::Created, "Scouting should succeed");

        assert!(
            total_encounters < 100,
            "Should eventually run out of durability"
        );
    }

    assert!(
        total_encounters > 1,
        "With 10000 durability and cost 100 per card, should survive multiple encounters"
    );
}

#[test]
fn scenario_abort_fishing_encounter() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    let fc_enc = fishing_encounter_ids(&client);
    assert!(!fc_enc.is_empty(), "Should have fishing encounter cards");
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        fc_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created);

    let encounter = combat_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Fishing")
    );

    let (status, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    assert_eq!(status, Status::Created, "Abort should succeed");

    let result = combat_result(&client);
    assert_eq!(result, Some("PlayerLost".to_string()));

    // Should be able to scout after abort
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(
        status,
        Status::Created,
        "Should be able to scout after abort"
    );
}

#[test]
fn scenario_fishing_range_modification_cards_exist() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":100}"#);

    // Check that fishing expansion cards exist in library
    let cards = get_json(&client, "/library/cards?card_kind=Fishing");
    let card_arr = cards.as_array().expect("Should be array");

    // Should have more than the original 3 fishing cards
    assert!(
        card_arr.len() >= 10,
        "Should have at least 10 fishing cards (3 original + 7 expansion), got {}",
        card_arr.len()
    );
}

#[test]
fn scenario_fishing_encounter_initializes_range_tokens() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":600}"#);

    // Pick fishing encounter dynamically
    let fc_enc = fishing_encounter_ids(&client);
    assert!(!fc_enc.is_empty(), "Should have fishing encounter cards");
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        fc_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(
        status,
        Status::Created,
        "Pick fishing encounter should succeed"
    );

    // After starting fishing encounter, range tokens should be set in encounter_tokens
    let range_min = encounter_token(&client, "FishingRangeMin");
    let range_max = encounter_token(&client, "FishingRangeMax");
    let fish_amount = encounter_token(&client, "FishAmount");

    assert!(
        range_min > 0,
        "FishingRangeMin should be set, got {}",
        range_min
    );
    assert!(
        range_max > 0,
        "FishingRangeMax should be set, got {}",
        range_max
    );
    assert!(
        fish_amount >= 1,
        "FishAmount should be at least 1, got {}",
        fish_amount
    );
}
