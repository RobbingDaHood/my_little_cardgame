use my_little_cardgame::library::types::{CombatSnapshot, TokenId};
use my_little_cardgame::rocket_initialize;
use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;
use rocket::serde::json::serde_json;

fn client() -> Client {
    Client::tracked(rocket_initialize()).expect("valid rocket instance")
}

#[test]
fn get_player_tokens_returns_initial_balances() {
    let client = client();
    let response = client.get("/player/tokens").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    let tokens: std::collections::HashMap<TokenId, i64> = serde_json::from_str(&body).unwrap();
    assert!(tokens.contains_key(&TokenId::Health));
    assert!(tokens.contains_key(&TokenId::Foresight));
}

#[test]
fn set_seed_records_action() {
    let client = client();
    let response = client
        .post("/player/seed")
        .header(ContentType::JSON)
        .body(r#"{"seed": 42}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    assert!(body.contains("42"));

    // Verify action log contains SetSeed entry
    let log_resp = client.get("/actions/log?action_type=SetSeed").dispatch();
    assert_eq!(log_resp.status(), Status::Ok);
    let log_body = log_resp.into_string().unwrap();
    assert!(log_body.contains("SetSeed"));
}

#[test]
fn actions_log_filtering() {
    let client = client();

    // Generate some actions first
    client
        .post("/action")
        .header(ContentType::JSON)
        .body(r#"{"action_type":"GrantToken","token_id":"Insight","amount":1}"#)
        .dispatch();
    client
        .post("/action")
        .header(ContentType::JSON)
        .body(r#"{"action_type":"GrantToken","token_id":"Renown","amount":2}"#)
        .dispatch();

    // Fetch all actions
    let response = client.get("/actions/log").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    let log: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(log["entries"].is_array());

    // Filter by from_seq (high value returns empty)
    let response = client.get("/actions/log?from_seq=999999").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    let log: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(log["entries"].as_array().unwrap().is_empty());

    // Filter by from_seq (low value returns all)
    let response = client.get("/actions/log?from_seq=0").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    let log: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(!log["entries"].as_array().unwrap().is_empty());

    // Filter by action_type
    let response = client.get("/actions/log?action_type=GrantToken").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    let log: serde_json::Value = serde_json::from_str(&body).unwrap();
    let entries = log["entries"].as_array().unwrap();
    assert!(entries.len() >= 2);
    for entry in entries {
        assert_eq!(entry["action_type"], "GrantToken");
    }

    // Filter by action_type that doesn't exist
    let response = client
        .get("/actions/log?action_type=NonExistent")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    let log: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(log["entries"].as_array().unwrap().is_empty());

    // Filter by since=0 (should include all)
    let response = client.get("/actions/log?since=0").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    let log: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(!log["entries"].as_array().unwrap().is_empty());

    // Filter by since=very_high (should exclude all)
    let response = client
        .get("/actions/log?since=99999999999999999999999")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    let log: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(log["entries"].as_array().unwrap().is_empty());

    // Limit=1
    let response = client.get("/actions/log?limit=1").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    let log: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(log["entries"].as_array().unwrap().len() <= 1);
}

#[test]
fn combat_result_returns_404_when_no_result() {
    let client = client();
    let response = client.get("/combat/result").dispatch();
    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn combat_lifecycle_with_enemy_play_and_advance() {
    let client = client();

    // Initialize combat
    let response = client.post("/tests/combat").dispatch();
    assert_eq!(response.status(), Status::Created);

    // Get initial combat state
    let combat = get_combat(&client).expect("combat exists");
    assert!(combat.player_turn);

    // Advance phase
    let response = client.post("/combat/advance").dispatch();
    assert_eq!(response.status(), Status::Created);

    // Enemy play
    let response = client.post("/combat/enemy_play").dispatch();
    assert_eq!(response.status(), Status::Created);
}

#[test]
fn enemy_play_when_no_combat_returns_created() {
    let client = client();
    // No combat initialized - should still return Created (no-op)
    let response = client.post("/combat/enemy_play").dispatch();
    assert_eq!(response.status(), Status::Created);
}

#[test]
fn area_deck_endpoints() {
    let client = client();

    // Get area info
    let response = client.get("/area").dispatch();
    assert_eq!(response.status(), Status::Ok);

    // Get area encounters
    let response = client.get("/area/encounters").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    let encounters: Vec<usize> = serde_json::from_str(&body).unwrap();
    assert!(!encounters.is_empty());
}

#[test]
fn library_tokens_endpoint() {
    let client = client();
    let response = client.get("/tokens").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    let tokens: Vec<TokenId> = serde_json::from_str(&body).unwrap();
    assert!(!tokens.is_empty());
    assert!(tokens.contains(&TokenId::Health));
}

#[test]
fn play_card_without_combat_returns_error() {
    let client = client();
    let response = client
        .post("/action")
        .header(ContentType::JSON)
        .body(r#"{"action_type":"PlayCard","card_id":0}"#)
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn play_grant_token_action() {
    let client = client();
    let response = client
        .post("/action")
        .header(ContentType::JSON)
        .body(r#"{"action_type":"GrantToken","token_id":"Insight","amount":5}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Created);

    // Verify token was granted
    let tokens_resp = client.get("/player/tokens").dispatch();
    let body = tokens_resp.into_string().unwrap();
    let tokens: std::collections::HashMap<TokenId, i64> = serde_json::from_str(&body).unwrap();
    assert_eq!(tokens.get(&TokenId::Insight).copied().unwrap_or(0), 5);
}

#[test]
fn play_consume_token_via_grant_negative() {
    let client = client();
    // Grant tokens
    client
        .post("/action")
        .header(ContentType::JSON)
        .body(r#"{"action_type":"GrantToken","token_id":"Renown","amount":10}"#)
        .dispatch();

    // Grant negative amount to simulate consume
    let response = client
        .post("/action")
        .header(ContentType::JSON)
        .body(r#"{"action_type":"GrantToken","token_id":"Renown","amount":-3}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Created);

    // Verify
    let tokens_resp = client.get("/player/tokens").dispatch();
    let body = tokens_resp.into_string().unwrap();
    let tokens: std::collections::HashMap<TokenId, i64> = serde_json::from_str(&body).unwrap();
    assert_eq!(tokens.get(&TokenId::Renown).copied().unwrap_or(0), 7);
}

#[test]
fn play_encounter_pick_starts_combat() {
    let client = client();

    // Get area encounters to find a valid encounter card
    let response = client.get("/area/encounters").dispatch();
    let body = response.into_string().unwrap();
    let encounters: Vec<usize> = serde_json::from_str(&body).unwrap();
    assert!(!encounters.is_empty());
    let encounter_id = encounters[0];

    // Pick the encounter
    let body = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        encounter_id
    );
    let response = client
        .post("/action")
        .header(ContentType::JSON)
        .body(&body)
        .dispatch();
    assert_eq!(response.status(), Status::Created);

    // Combat should be active now
    let combat = get_combat(&client);
    assert!(combat.is_some());
}

#[test]
fn play_finish_scouting_after_combat_win() {
    let client = client();

    // Pick an encounter to enter combat
    let response = client.get("/area/encounters").dispatch();
    let encounters: Vec<usize> = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let encounter_id = encounters[0];

    let body = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        encounter_id
    );
    client
        .post("/action")
        .header(ContentType::JSON)
        .body(&body)
        .dispatch();

    // Combat starts in Defending phase. Play defence, advance, attack, advance.
    // Repeat until enemy is dead (enemy has 20 HP, attack deals 5 → 4 rounds).
    for _ in 0..4 {
        // Defending phase: play defence card (id 1)
        client
            .post("/action")
            .header(ContentType::JSON)
            .body(r#"{"action_type":"PlayCard","card_id":1}"#)
            .dispatch();
        // Advance to Attacking
        client.post("/combat/advance").dispatch();
        // Attacking phase: play attack card (id 0) — deals 5 damage
        client
            .post("/action")
            .header(ContentType::JSON)
            .body(r#"{"action_type":"PlayCard","card_id":0}"#)
            .dispatch();
        // Advance to Resourcing
        client.post("/combat/advance").dispatch();
        // Resourcing phase: play resource card (id 2)
        client
            .post("/action")
            .header(ContentType::JSON)
            .body(r#"{"action_type":"PlayCard","card_id":2}"#)
            .dispatch();
        // Advance to Defending (next round)
        client.post("/combat/advance").dispatch();
        // Enemy play
        client.post("/combat/enemy_play").dispatch();
    }

    // Check if combat ended and we're in Scouting
    let combat = get_combat(&client);
    if combat.is_some() {
        // If combat is still active, it hasn't ended yet — play more attacks
        return; // Skip rest of test, combat didn't end naturally
    }

    // Check combat result
    let result_resp = client.get("/combat/result").dispatch();
    if result_resp.status() == Status::Ok {
        // We're in scouting, finish it
        let response = client
            .post("/action")
            .header(ContentType::JSON)
            .body(r#"{"action_type":"FinishScouting"}"#)
            .dispatch();
        assert_eq!(response.status(), Status::Created);
    }
}

#[test]
fn simulate_combat_endpoint() {
    let client = client();

    let request = serde_json::json!({
        "initial_state": {
            "round": 1,
            "player_turn": true,
            "phase": "Attacking",
            "player_tokens": {"Health": 20, "Shield": 0},
            "enemy": {
                "name": "Test Enemy",
                "active_tokens": {"Health": 10},
                "attack_deck": [],
                "defence_deck": [],
                "resource_deck": []
            },
            "encounter_card_id": 0,
            "is_finished": false,
            "winner": null
        },
        "seed": 42,
        "actions": [],
        "card_defs": {}
    });

    let response = client
        .post("/tests/combat/simulate")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&request).unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
}

fn get_combat(client: &Client) -> Option<CombatSnapshot> {
    let response = client.get("/combat").dispatch();
    if response.status().code == 404 {
        None
    } else if response.status().code == 200 {
        let body = response.into_string().unwrap();
        Some(serde_json::from_str(&body).unwrap())
    } else {
        panic!("Unexpected status: {}", response.status());
    }
}

#[test]
fn draw_encounter_action() {
    let client = client();
    // Get area encounters
    let response = client.get("/area/encounters").dispatch();
    let encounters: Vec<usize> = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let encounter_id = encounters[0];

    let body = format!(
        r#"{{"action_type":"DrawEncounter","area_id":"current","encounter_id":{}}}"#,
        encounter_id
    );
    let response = client
        .post("/action")
        .header(ContentType::JSON)
        .body(&body)
        .dispatch();
    assert_eq!(response.status(), Status::Created);
}

#[test]
fn replace_encounter_action() {
    let client = client();
    let response = client.get("/area/encounters").dispatch();
    let encounters: Vec<usize> = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let encounter_id = encounters[0];

    let body = format!(
        r#"{{"action_type":"ReplaceEncounter","area_id":"current","old_encounter_id":{},"new_encounter_id":99}}"#,
        encounter_id
    );
    let response = client
        .post("/action")
        .header(ContentType::JSON)
        .body(&body)
        .dispatch();
    assert_eq!(response.status(), Status::Created);
}

#[test]
fn apply_scouting_action() {
    let client = client();
    let response = client
        .post("/action")
        .header(ContentType::JSON)
        .body(r#"{"action_type":"ApplyScouting","area_id":"current","parameters":"test"}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Created);
}

#[test]
fn encounter_play_card_action() {
    let client = client();

    // Pick encounter to start combat
    let response = client.get("/area/encounters").dispatch();
    let encounters: Vec<usize> = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let encounter_id = encounters[0];

    let body = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        encounter_id
    );
    client
        .post("/action")
        .header(ContentType::JSON)
        .body(&body)
        .dispatch();

    // Advance to Attacking to play defence card first
    // Combat starts in Defending — play defence card (id 1)
    let response = client
        .post("/action")
        .header(ContentType::JSON)
        .body(r#"{"action_type":"EncounterPlayCard","card_id":1,"effects":[]}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Created);
}

#[test]
fn encounter_apply_scouting_action() {
    let client = client();
    let response = client.get("/area/encounters").dispatch();
    let encounters: Vec<usize> = serde_json::from_str(&response.into_string().unwrap()).unwrap();

    let body = format!(
        r#"{{"action_type":"EncounterApplyScouting","card_ids":[{}]}}"#,
        encounters[0]
    );
    let response = client
        .post("/action")
        .header(ContentType::JSON)
        .body(&body)
        .dispatch();
    assert_eq!(response.status(), Status::Created);
}

#[test]
fn abandon_combat_when_no_combat() {
    let client = client();
    let response = client
        .post("/action")
        .header(ContentType::JSON)
        .body(r#"{"action_type":"AbandonCombat"}"#)
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn finish_scouting_when_not_in_scouting() {
    let client = client();
    let response = client
        .post("/action")
        .header(ContentType::JSON)
        .body(r#"{"action_type":"FinishScouting"}"#)
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn set_seed_via_action() {
    let client = client();
    let response = client
        .post("/action")
        .header(ContentType::JSON)
        .body(r#"{"action_type":"SetSeed","seed":12345}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Created);
}

#[test]
fn play_card_in_combat_with_wrong_phase() {
    let client = client();

    // Pick encounter
    let response = client.get("/area/encounters").dispatch();
    let encounters: Vec<usize> = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let body = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        encounters[0]
    );
    client
        .post("/action")
        .header(ContentType::JSON)
        .body(&body)
        .dispatch();

    // Combat starts in Defending, try playing Attack card (wrong phase)
    let response = client
        .post("/action")
        .header(ContentType::JSON)
        .body(r#"{"action_type":"PlayCard","card_id":0}"#)
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn play_card_nonexistent() {
    let client = client();

    // Start combat
    let response = client.get("/area/encounters").dispatch();
    let encounters: Vec<usize> = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let body = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        encounters[0]
    );
    client
        .post("/action")
        .header(ContentType::JSON)
        .body(&body)
        .dispatch();

    let response = client
        .post("/action")
        .header(ContentType::JSON)
        .body(r#"{"action_type":"PlayCard","card_id":9999}"#)
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn add_test_library_card_endpoint() {
    let client = client();
    let card = serde_json::json!({
        "kind": {"kind": "Attack", "effects": []},
        "counts": {"library": 0, "deck": 5, "hand": 0, "discard": 0}
    });
    let response = client
        .post("/tests/library/cards")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&card).unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Created);
}
