use super::helpers::*;
use rocket::http::Status;

const TOKENS: &str = include_str!("../configurations/tokens_default.json");
const TOKENS_LOW_HP: &str = include_str!("../configurations/tokens_low_health.json");
const SHARED: &str = include_str!("../configurations/shared_effects.json");
const COMBAT_WIN: &str = include_str!("../configurations/combat_win.json");
const COMBAT_LOSS: &str = include_str!("../configurations/combat_loss.json");

#[test]
fn scenario_combat_win_and_scout() {
    let client =
        create_test_client_from_json(42, TOKENS, &[("shared", SHARED), ("combat", COMBAT_WIN)]);
    // No NewGame needed — GameState is already initialized with custom config.

    let enc_ids = combat_encounter_ids(&client);
    assert!(!enc_ids.is_empty(), "Should have combat encounter cards");
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc_ids[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    let combat = combat_state(&client);
    assert_eq!(
        combat.get("outcome").and_then(|v| v.as_str()),
        Some("Undecided")
    );

    for _ in 0..50 {
        if !play_one_round(&client) {
            break;
        }
    }

    let result = combat_result(&client);
    assert_eq!(
        result.as_deref(),
        Some("PlayerWon"),
        "Player should win against weak enemy"
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

    let combat2 = combat_state(&client);
    assert_eq!(
        combat2.get("outcome").and_then(|v| v.as_str()),
        Some("Undecided")
    );
}

#[test]
fn scenario_combat_loss_and_scout() {
    let client = create_test_client_from_json(
        42,
        TOKENS_LOW_HP,
        &[("shared", SHARED), ("combat", COMBAT_LOSS)],
    );
    // No NewGame needed — GameState is already initialized with custom config.

    let enc_ids = combat_encounter_ids(&client);
    assert!(!enc_ids.is_empty(), "Should have combat encounters");
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc_ids[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    for _ in 0..50 {
        if !play_one_round(&client) {
            break;
        }
    }

    let result = combat_result(&client);
    assert_eq!(
        result.as_deref(),
        Some("PlayerLost"),
        "Player should lose against massive enemy"
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
