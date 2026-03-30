use super::helpers::*;
use my_little_cardgame::rocket_initialize;
use rocket::http::Status;
use rocket::local::blocking::Client;

const TOKENS: &str = include_str!("../configurations/tokens_default.json");
const HERBALISM_WIN: &str = include_str!("../configurations/herbalism_win.json");
const HERBALISM_LOSS: &str = include_str!("../configurations/herbalism_loss.json");

/// Find herbalism card IDs available in the player's hand.
fn herbalism_hand_card_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Herbalism");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| c.get("id").and_then(|v| v.as_u64()).map(|v| v as usize))
        .collect()
}

/// Play one herbalism card. Returns true if the herbalism encounter is still active.
fn play_one_herbalism_card(client: &Client) -> bool {
    let herb_ids = herbalism_hand_card_ids(client);
    if herb_ids.is_empty() {
        return false;
    }
    let card_id = herb_ids[0];
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

/// Find herbalism encounter card IDs in the encounter hand.
fn herbalism_encounter_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| {
            let kind = c.get("kind")?;
            let enc_kind = kind.get("encounter_kind")?;
            if enc_kind.get("encounter_type")?.as_str()? == "Herbalism" {
                c.get("id")?.as_u64().map(|v| v as usize)
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn scenario_herbalism_win_and_scout() {
    let client = create_test_client_from_json(42, TOKENS, &[("herbalism", HERBALISM_WIN)]);

    let enc_ids = herbalism_encounter_ids(&client);
    assert!(!enc_ids.is_empty(), "Should have herbalism encounters");
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
        Some("Herbalism")
    );

    // Play herbalism cards until encounter resolves
    for _ in 0..50 {
        if !play_one_herbalism_card(&client) {
            break;
        }
    }

    let result = combat_result(&client);
    assert_eq!(
        result.as_deref(),
        Some("PlayerWon"),
        "Herbalism should win with 1 plant remaining"
    );

    // Scout and pick another encounter
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(status, Status::Created);

    let enc_after = encounter_hand_ids(&client);
    assert!(
        !enc_after.is_empty(),
        "Should have encounters after scouting"
    );
    let pick2 = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc_after[0]
    );
    let (status, _) = post_action(&client, &pick2);
    assert_eq!(status, Status::Created);
}

#[test]
fn scenario_herbalism_loss_and_scout() {
    let client = create_test_client_from_json(42, TOKENS, &[("herbalism", HERBALISM_LOSS)]);

    let enc_ids = herbalism_encounter_ids(&client);
    assert!(!enc_ids.is_empty());
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc_ids[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    for _ in 0..50 {
        if !play_one_herbalism_card(&client) {
            break;
        }
    }

    let result = combat_result(&client);
    assert_eq!(
        result.as_deref(),
        Some("PlayerLost"),
        "Should lose with 0 plants remaining"
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
fn scenario_herbalism_encounter_full_loop() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // 1. Start a new game with a fixed seed
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created, "NewGame should succeed");

    // 2. Verify herbalism encounter cards are in hand
    let herb_enc = herbalism_encounter_ids(&client);
    assert!(
        !herb_enc.is_empty(),
        "Should have herbalism encounter cards in hand"
    );

    // 3. Pick the herbalism encounter dynamically
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        herb_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created, "PickEncounter should succeed");

    // 4. Verify herbalism encounter started
    let encounter = combat_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Herbalism"),
        "Encounter should be Herbalism type"
    );
    assert_eq!(
        encounter.get("outcome").and_then(|v| v.as_str()),
        Some("Undecided"),
        "Herbalism should be active"
    );

    // 5. Verify plant_hand has 8 cards (one per plant type in current config)
    let plant_hand = encounter
        .get("plant_hand")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    assert_eq!(plant_hand, 8, "Plant should start with 8 cards");

    // 6. Verify player has HerbalismDurability token
    let durability = player_token(&client, "HerbalismDurability");
    assert_eq!(
        durability, 10000,
        "Player should start with 10000 herbalism durability"
    );

    // 7. Play herbalism encounters in a loop until durability runs out
    let mut total_encounters = 0;
    let mut last_outcome;
    loop {
        let mut round_turns = 0;
        while play_one_herbalism_card(&client) {
            round_turns += 1;
            assert!(
                round_turns < 50,
                "Herbalism round should end within 50 turns"
            );
        }
        total_encounters += 1;

        last_outcome = combat_result(&client).unwrap_or_default();

        if last_outcome == "PlayerWon" {
            let plant = player_token(&client, "Plant");
            assert!(
                plant > 0,
                "Player should have Plant tokens after winning herbalism"
            );
        }

        if last_outcome == "PlayerLost" {
            break;
        }

        // Scout and pick another herbalism encounter
        let (status, _) = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
        assert_eq!(status, Status::Created, "ApplyScouting should succeed");

        let herb_enc = herbalism_encounter_ids(&client);
        if herb_enc.is_empty() {
            break;
        }
        let pick_json = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            herb_enc[0]
        );
        let (status, _) = post_action(&client, &pick_json);
        assert_eq!(status, Status::Created, "PickEncounter should succeed");

        assert!(
            total_encounters < 200,
            "Player should eventually lose from durability depletion"
        );
    }

    assert!(
        total_encounters >= 1,
        "Player should have completed at least one herbalism encounter"
    );

    // 8. Scout after final encounter
    if last_outcome == "PlayerLost" || last_outcome == "PlayerWon" {
        let (status, _) = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
        if last_outcome == "PlayerLost" {
            assert_eq!(
                status,
                Status::Created,
                "Should be able to scout after herbalism loss"
            );
        }
    }
}

#[test]
fn scenario_abort_herbalism_encounter() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Start game
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    // Pick herbalism encounter
    let herb_enc = herbalism_encounter_ids(&client);
    assert!(!herb_enc.is_empty());
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        herb_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created);

    // Abort it
    let (status, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    assert_eq!(status, Status::Created, "Should be able to abort herbalism");

    // Verify outcome is PlayerLost
    let last_result = combat_result(&client).unwrap_or_default();
    assert_eq!(last_result, "PlayerLost", "Abort should result in loss");
}

#[test]
fn scenario_herbalism_match_mode_cards_exist() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":200}"#);

    // Check herbalism cards
    let cards = get_json(&client, "/library/cards?card_kind=Herbalism");
    let card_arr = cards.as_array().expect("Should be array");

    // Should have original 3 + 4 expansion = 7 herbalism cards
    assert!(
        card_arr.len() >= 7,
        "Should have at least 7 herbalism cards, got {}",
        card_arr.len()
    );
}
