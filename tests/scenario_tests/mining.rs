use super::helpers::*;
use my_little_cardgame::rocket_initialize;
use rocket::http::Status;
use rocket::local::blocking::Client;

/// Find mining card IDs available in the player's hand.
fn mining_hand_card_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Mining");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| c.get("id").and_then(|v| v.as_u64()).map(|v| v as usize))
        .collect()
}

/// Play one mining card. Returns true if the mining encounter is still active.
fn play_one_mining_card(client: &Client) -> bool {
    let mining_ids = mining_hand_card_ids(client);
    if mining_ids.is_empty() {
        return false;
    }
    for card_id in mining_ids {
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

const TOKENS: &str = include_str!("../configurations/tokens_default.json");
const TOKENS_LOW_DUR: &str = include_str!("../configurations/tokens_low_durability.json");
const MINING_WIN: &str = include_str!("../configurations/mining_win.json");
const MINING_LOSS: &str = include_str!("../configurations/mining_loss.json");

#[test]
fn scenario_mining_win_and_scout() {
    let client = create_test_client_from_json(42, TOKENS, &[("mining", MINING_WIN)]);

    let enc_ids = mining_encounter_ids(&client);
    assert!(!enc_ids.is_empty(), "Should have mining encounter cards");
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
        Some("Mining")
    );

    // Play mining cards
    let mut cards_played = 0;
    while play_one_mining_card(&client) {
        cards_played += 1;
        if cards_played >= 5 {
            break;
        }
    }

    // Conclude to win
    let enc = combat_state(&client);
    if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
        let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        assert_eq!(status, Status::Created);
    }

    let result = combat_result(&client);
    assert_eq!(
        result.as_deref(),
        Some("PlayerWon"),
        "Mining should be won via conclude"
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
    assert_eq!(
        status,
        Status::Created,
        "Should pick another encounter after scouting"
    );
}

#[test]
fn scenario_mining_loss_and_scout() {
    let client = create_test_client_from_json(42, TOKENS_LOW_DUR, &[("mining", MINING_LOSS)]);

    let enc_ids = mining_encounter_ids(&client);
    assert!(!enc_ids.is_empty(), "Should have mining encounters");
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc_ids[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    // Play mining cards — ore deck should deplete durability
    for _ in 0..50 {
        if !play_one_mining_card(&client) {
            break;
        }
    }

    // If still undecided, conclude to check
    let enc = combat_state(&client);
    if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
        let _ = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
    }

    let result = combat_result(&client);
    assert_eq!(
        result.as_deref(),
        Some("PlayerLost"),
        "Mining should lose from durability depletion"
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
        "Should have encounters after loss scouting"
    );

    let pick2 = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc_after[0]
    );
    let (status, _) = post_action(&client, &pick2);
    assert_eq!(status, Status::Created, "Should pick encounter after loss");
}

#[test]
fn scenario_mining_encounter_full_loop() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // 1. Start a new game with a fixed seed
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created, "NewGame should succeed");

    // 2. Verify mining encounter cards are in hand
    let mining_enc = mining_encounter_ids(&client);
    assert!(
        !mining_enc.is_empty(),
        "Should have mining encounter cards in hand"
    );

    // 3. Pick the mining encounter
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        mining_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created, "PickEncounter should succeed");

    // 4. Verify mining encounter started with light level
    let encounter = combat_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Mining"),
        "Encounter should be Mining type"
    );
    assert_eq!(
        encounter.get("outcome").and_then(|v| v.as_str()),
        Some("Undecided"),
        "Mining should be active"
    );

    // 5. Verify light level is initialized at 300
    let light_level = encounter_token(&client, "MiningLightLevel");
    assert_eq!(light_level, 300, "Light level should start at 300");

    // 6. Verify yield starts at 0
    let mining_yield = encounter_token(&client, "MiningYield");
    assert_eq!(mining_yield, 0, "Yield should start at 0");

    // 7. Verify player has MiningDurability token
    let durability = player_token(&client, "MiningDurability");
    assert_eq!(
        durability, 10000,
        "Player should start with 10000 mining durability"
    );

    // 8. Play mining cards to accumulate some yield
    let mut cards_played = 0;
    while cards_played < 5 {
        if !play_one_mining_card(&client) {
            break;
        }
        cards_played += 1;
    }

    // 9. Verify yield has accumulated (at least some cards should produce yield)
    let yield_after = encounter_token(&client, "MiningYield");
    assert!(
        yield_after > 0,
        "Yield should have accumulated after playing mining power cards, got {}",
        yield_after
    );

    // 10. Conclude the mining encounter
    let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
    assert_eq!(status, Status::Created, "Conclude should succeed");

    // 11. Verify encounter ended as PlayerWon
    let result = combat_result(&client);
    assert_eq!(result, Some("PlayerWon".to_string()));

    // 12. Verify ore reward was granted (min(stamina, yield))
    let ore = player_token(&client, "Ore");
    assert!(
        ore > 0,
        "Player should have Ore tokens after concluding mining"
    );

    // 13. Verify encounter-scoped tokens are cleaned up
    let light_after = player_token(&client, "MiningLightLevel");
    assert_eq!(
        light_after, 0,
        "Light level should be reset after encounter"
    );
    let yield_after = player_token(&client, "MiningYield");
    assert_eq!(yield_after, 0, "Yield should be reset after encounter");

    // 14. Scout after encounter
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(status, Status::Created, "ApplyScouting should succeed");
}

#[test]
fn scenario_abort_mining_encounter() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // 1. Start a new game
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    // 2. Pick a mining encounter
    let mining_enc = mining_encounter_ids(&client);
    assert!(!mining_enc.is_empty(), "Should have mining encounter cards");
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        mining_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created);

    // 3. Verify mining encounter is active
    let encounter = combat_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Mining")
    );

    // 4. Abort the encounter
    let (status, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    assert_eq!(status, Status::Created, "Abort should succeed");

    // 5. Verify encounter result is PlayerLost
    let result = combat_result(&client);
    assert_eq!(result, Some("PlayerLost".to_string()));

    // 6. Verify encounter-scoped tokens are cleaned up
    let light_after = player_token(&client, "MiningLightLevel");
    assert_eq!(light_after, 0, "Light level should be reset after abort");
    let yield_after = player_token(&client, "MiningYield");
    assert_eq!(yield_after, 0, "Yield should be reset after abort");

    // 7. Verify can scout after abort
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(
        status,
        Status::Created,
        "Should be able to scout after abort"
    );

    // 8. Verify aborting combat is rejected
    let (status2, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status2, Status::Created);
    let combat_enc = combat_encounter_ids(&client);
    assert!(!combat_enc.is_empty(), "Should have combat encounter cards");
    let pick_combat_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        combat_enc[0]
    );
    let (status2, _) = post_action(&client, &pick_combat_json);
    assert_eq!(status2, Status::Created);
    let (status2, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    assert_eq!(
        status2,
        Status::BadRequest,
        "Should not be able to abort combat"
    );
}

#[test]
fn scenario_mining_then_combat_coexist() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Start game and verify both combat and mining encounters exist in hand
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":100}"#);
    assert_eq!(status, Status::Created);

    let combat_enc = combat_encounter_ids(&client);
    let mining_enc = mining_encounter_ids(&client);
    assert!(!combat_enc.is_empty(), "Should have combat encounter cards");
    assert!(!mining_enc.is_empty(), "Should have mining encounter cards");

    // Do a mining encounter first — use conclude to end it cleanly
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        mining_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created);

    // Play a few mining cards, then conclude
    let mut turns = 0;
    while play_one_mining_card(&client) {
        turns += 1;
        if turns >= 3 {
            break;
        }
    }

    // If encounter is still active, conclude it
    let encounter = combat_state(&client);
    if encounter.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
        let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        assert_eq!(status, Status::Created, "Conclude should succeed");
    }

    // Scout after mining
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(status, Status::Created);

    // Now do a combat encounter
    let combat_enc = combat_encounter_ids(&client);
    assert!(
        !combat_enc.is_empty(),
        "Should still have combat encounter cards"
    );
    let pick_combat_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        combat_enc[0]
    );
    let (status, _) = post_action(&client, &pick_combat_json);
    assert_eq!(status, Status::Created);

    // Verify combat started correctly
    let encounter = combat_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Combat"),
        "Should now be in a combat encounter"
    );
    assert!(player_health(&client) > 0, "Player should have health");

    // Play one round of combat to verify it works after mining
    play_one_round(&client);
}

#[test]
fn scenario_mining_expansion_cards_exist() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":400}"#);

    // Check mining cards
    let cards = get_json(&client, "/library/cards?card_kind=Mining");
    let card_arr = cards.as_array().expect("Should be array");

    // Should have 8 mining cards (power, light, rest varieties)
    assert!(
        card_arr.len() >= 8,
        "Should have at least 8 mining cards, got {}",
        card_arr.len()
    );
}
