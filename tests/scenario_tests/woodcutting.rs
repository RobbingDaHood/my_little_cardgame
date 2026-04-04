use super::helpers::*;
use my_little_cardgame::rocket_initialize;
use rocket::http::Status;
use rocket::local::blocking::Client;

const TOKENS: &str = include_str!("../configurations/tokens_default.json");
const TOKENS_LOW_DUR: &str = include_str!("../configurations/tokens_low_durability.json");
const WOODCUTTING_WIN: &str = include_str!("../configurations/woodcutting_win.json");
const WOODCUTTING_LOSS: &str = include_str!("../configurations/woodcutting_loss.json");

fn woodcutting_hand_card_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Woodcutting");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| c.get("id").and_then(|v| v.as_u64()).map(|v| v as usize))
        .collect()
}

fn play_one_woodcutting_card(client: &Client) -> bool {
    let wc_ids = woodcutting_hand_card_ids(client);
    if wc_ids.is_empty() {
        return false;
    }
    for card_id in wc_ids {
        let json = format!(
            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
            card_id
        );
        let (status, _) = post_action(client, &json);
        if status == Status::Created {
            let encounter = combat_state(client);
            return encounter.get("outcome").and_then(|v| v.as_str()) == Some("Undecided");
        }
    }
    false
}

fn woodcutting_encounter_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| {
            let kind = c.get("kind")?;
            let enc_kind = kind.get("encounter_kind")?;
            if enc_kind.get("encounter_type")?.as_str()? == "Woodcutting" {
                c.get("id")?.as_u64().map(|v| v as usize)
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn scenario_woodcutting_win_and_scout() {
    let client = create_test_client_from_json(42, TOKENS, &[("woodcutting", WOODCUTTING_WIN)]);

    let enc_ids = woodcutting_encounter_ids(&client);
    assert!(!enc_ids.is_empty(), "Should have woodcutting encounters");
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
        Some("Woodcutting")
    );

    // Play woodcutting cards until encounter ends
    for _ in 0..50 {
        if !play_one_woodcutting_card(&client) {
            break;
        }
    }

    let result = combat_result(&client);
    assert_eq!(
        result.as_deref(),
        Some("PlayerWon"),
        "Should win with 8 plays"
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
fn scenario_woodcutting_loss_and_scout() {
    // woodcutting_loss.json has cards costing 99999 Durability each — unpayable
    let client =
        create_test_client_from_json(42, TOKENS_LOW_DUR, &[("woodcutting", WOODCUTTING_LOSS)]);

    let enc_ids = woodcutting_encounter_ids(&client);
    assert!(!enc_ids.is_empty());
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc_ids[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    for _ in 0..50 {
        if !play_one_woodcutting_card(&client) {
            break;
        }
    }

    let result = combat_result(&client);
    assert_eq!(
        result.as_deref(),
        Some("PlayerLost"),
        "Should lose with unpayable cards"
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
fn scenario_woodcutting_encounter_full_loop() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created, "NewGame should succeed");

    let wc_enc = woodcutting_encounter_ids(&client);
    assert!(
        !wc_enc.is_empty(),
        "Should have woodcutting encounter cards in hand"
    );

    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        wc_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created, "PickEncounter should succeed");

    let encounter = combat_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Woodcutting"),
        "Encounter should be Woodcutting type"
    );
    assert_eq!(
        encounter.get("outcome").and_then(|v| v.as_str()),
        Some("Undecided"),
        "Woodcutting should be active"
    );
    assert_eq!(
        encounter.get("max_plays").and_then(|v| v.as_u64()),
        Some(8),
        "max_plays should be 8"
    );
    assert_eq!(
        encounter
            .get("played_cards")
            .and_then(|v| v.as_array())
            .map(|a| a.len()),
        Some(0),
        "No cards played yet"
    );

    let durability = player_token(&client, "WoodcuttingDurability");
    assert_eq!(
        durability, 10000,
        "Player should start with 10000 woodcutting durability"
    );

    // Play 8 cards (the encounter should auto-complete after 8)
    let mut total_turns = 0;
    loop {
        let still_going = play_one_woodcutting_card(&client);
        total_turns += 1;
        if !still_going {
            break;
        }
        assert!(total_turns < 50, "Woodcutting should end within 50 turns");
    }

    // After 8 cards, should always win
    let last_outcome = combat_result(&client).unwrap_or_default();
    assert_eq!(
        last_outcome, "PlayerWon",
        "Woodcutting should always win after 8 plays"
    );

    let lumber = player_token(&client, "Lumber");
    assert!(
        lumber > 0,
        "Player should have Lumber after winning woodcutting (got {})",
        lumber
    );

    // Verify durability was consumed (8 cards x 1 durability each = 8)
    let final_durability = player_token(&client, "WoodcuttingDurability");
    assert!(
        final_durability < 10000,
        "Durability should decrease after woodcutting (got {})",
        final_durability
    );

    // Should be in Scouting phase now; can scout and pick another encounter
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(status, Status::Created, "Should be able to scout after win");

    // Play multiple encounters until durability runs out
    let mut total_encounters = 1; // Already did one
    loop {
        let wc_enc = woodcutting_encounter_ids(&client);
        if wc_enc.is_empty() {
            break;
        }
        let pick_json = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            wc_enc[0]
        );
        let (status, _) = post_action(&client, &pick_json);
        assert_eq!(status, Status::Created, "PickEncounter should succeed");

        let mut round_turns = 0;
        loop {
            let still_going = play_one_woodcutting_card(&client);
            round_turns += 1;
            if !still_going {
                break;
            }
            assert!(round_turns < 50, "Round should end within 50 turns");
        }
        total_encounters += 1;

        let outcome = combat_result(&client).unwrap_or_default();
        if outcome == "PlayerLost" {
            // Loss can occur from durability exhaustion or deck exhaustion depending on deck size.
            let final_durability = player_token(&client, "WoodcuttingDurability");
            assert!(
                final_durability < 10000,
                "Durability should decrease during woodcutting (got {})",
                final_durability
            );
            break;
        }

        // If encounter is still active (stuck on unplayable cost cards), abort it
        let encounter = combat_state(&client);
        if encounter.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
            let (status, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
            assert_eq!(status, Status::Created, "Abort should succeed when stuck");
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
fn scenario_abort_woodcutting_encounter() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    let wc_enc = woodcutting_encounter_ids(&client);
    assert!(
        !wc_enc.is_empty(),
        "Should have woodcutting encounter cards"
    );
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        wc_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created);

    let encounter = combat_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Woodcutting")
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
fn scenario_woodcutting_expansion_cards_exist() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":300}"#);

    // Check woodcutting cards
    let cards = get_json(&client, "/library/cards?card_kind=Woodcutting");
    let card_arr = cards.as_array().expect("Should be array");

    // Should have original 4 + 1 cost + 5 expansion = 10 woodcutting cards
    assert!(
        card_arr.len() >= 10,
        "Should have at least 10 woodcutting cards, got {}",
        card_arr.len()
    );
}
