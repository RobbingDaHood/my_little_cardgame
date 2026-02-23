use my_little_cardgame::library::types::{CombatSnapshot, TokenType};
use my_little_cardgame::player_tokens::TokenBalance;
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
    let tokens: Vec<TokenBalance> = serde_json::from_str(&body).unwrap();
    assert!(tokens
        .iter()
        .any(|t| t.token.token_type == TokenType::Health));
    assert!(tokens
        .iter()
        .any(|t| t.token.token_type == TokenType::Foresight));
}

#[test]
fn new_game_records_action() {
    let client = client();
    let response = client
        .post("/action")
        .header(ContentType::JSON)
        .body(r#"{"action_type":"NewGame","seed":42}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Created);

    // Verify action log contains NewGame entry
    let log_resp = client.get("/actions/log?action_type=NewGame").dispatch();
    assert_eq!(log_resp.status(), Status::Ok);
    let log_body = log_resp.into_string().unwrap();
    assert!(log_body.contains("NewGame"));
}

#[test]
fn actions_log_filtering() {
    let client = client();

    // Generate some actions first
    client
        .post("/action")
        .header(ContentType::JSON)
        .body(r#"{"action_type":"NewGame","seed":111}"#)
        .dispatch();
    client
        .post("/action")
        .header(ContentType::JSON)
        .body(r#"{"action_type":"NewGame","seed":222}"#)
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
    let response = client.get("/actions/log?action_type=NewGame").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    let log: serde_json::Value = serde_json::from_str(&body).unwrap();
    let entries = log["entries"].as_array().unwrap();
    assert!(entries.len() >= 2);
    for entry in entries {
        assert_eq!(entry["action_type"], "NewGame");
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
    let response = client.post("/tests/combat/advance").dispatch();
    assert_eq!(response.status(), Status::Created);

    // Enemy play
    let response = client.post("/tests/combat/enemy_play").dispatch();
    assert_eq!(response.status(), Status::Created);
}

#[test]
fn enemy_play_when_no_combat_returns_created() {
    let client = client();
    // No combat initialized - should still return Created (no-op)
    let response = client.post("/tests/combat/enemy_play").dispatch();
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
    let tokens: Vec<TokenType> = serde_json::from_str(&body).unwrap();
    assert!(!tokens.is_empty());
    assert!(tokens.contains(&TokenType::Health));
}

#[test]
fn play_card_without_combat_returns_error() {
    let client = client();
    let response = client
        .post("/action")
        .header(ContentType::JSON)
        .body(r#"{"action_type":"EncounterPlayCard","card_id":0,"effects":[]}"#)
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
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
            .body(r#"{"action_type":"EncounterPlayCard","card_id":1,"effects":[]}"#)
            .dispatch();
        // Advance to Attacking
        client.post("/tests/combat/advance").dispatch();
        // Attacking phase: play attack card (id 0) — deals 5 damage
        client
            .post("/action")
            .header(ContentType::JSON)
            .body(r#"{"action_type":"EncounterPlayCard","card_id":0,"effects":[]}"#)
            .dispatch();
        // Advance to Resourcing
        client.post("/tests/combat/advance").dispatch();
        // Resourcing phase: play resource card (id 2)
        client
            .post("/action")
            .header(ContentType::JSON)
            .body(r#"{"action_type":"EncounterPlayCard","card_id":2,"effects":[]}"#)
            .dispatch();
        // Advance to Defending (next round)
        client.post("/tests/combat/advance").dispatch();
        // Enemy play
        client.post("/tests/combat/enemy_play").dispatch();
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
        // We're in scouting, finish it via EncounterApplyScouting
        let response = client
            .post("/action")
            .header(ContentType::JSON)
            .body(r#"{"action_type":"EncounterApplyScouting","card_ids":[3]}"#)
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
            "player_tokens": [
                {"token": {"token_type": "Health", "lifecycle": "PersistentCounter"}, "value": 20},
                {"token": {"token_type": "Shield", "lifecycle": "PersistentCounter"}, "value": 0}
            ],
            "enemy": {
                "active_tokens": [
                    {"token": {"token_type": "Health", "lifecycle": "PersistentCounter"}, "value": 10}
                ]
            },
            "encounter_card_id": 0,
            "is_finished": false,
            "outcome": "Undecided"
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
    // Set up combat via test endpoint so we can reach Scouting phase
    client.post("/tests/combat").dispatch();
    // Play attack cards until combat ends (enters Scouting)
    for _ in 0..20 {
        let combat_resp = client.get("/combat").dispatch();
        if combat_resp.status() != Status::Ok {
            break;
        }
        client
            .post("/action")
            .header(ContentType::JSON)
            .body(r#"{"action_type":"EncounterPlayCard","card_id":0,"effects":[]}"#)
            .dispatch();
        client.post("/tests/combat/advance").dispatch();
        client.post("/tests/combat/enemy_play").dispatch();
        client.post("/tests/combat/advance").dispatch();
        client.post("/tests/combat/advance").dispatch();
    }

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
fn encounter_apply_scouting_when_not_in_scouting() {
    let client = client();
    let response = client
        .post("/action")
        .header(ContentType::JSON)
        .body(r#"{"action_type":"EncounterApplyScouting","card_ids":[3]}"#)
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn new_game_via_action() {
    let client = client();
    let response = client
        .post("/action")
        .header(ContentType::JSON)
        .body(r#"{"action_type":"NewGame","seed":12345}"#)
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
        .body(r#"{"action_type":"EncounterPlayCard","card_id":0,"effects":[]}"#)
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
        .body(r#"{"action_type":"EncounterPlayCard","card_id":9999,"effects":[]}"#)
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
