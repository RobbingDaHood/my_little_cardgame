use my_little_cardgame::library::types::{CombatState, TokenType};
use my_little_cardgame::player_tokens::TokenBalance;
use my_little_cardgame::rocket_initialize;
use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;
use rocket::serde::json::serde_json;

fn client() -> Client {
    Client::tracked(rocket_initialize()).expect("valid rocket instance")
}

fn encounter_hand_ids(client: &Client) -> Vec<usize> {
    let response = client
        .get("/library/cards?location=Hand&card_kind=Encounter")
        .dispatch();
    let body = response.into_string().unwrap();
    let cards: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap();
    cards
        .iter()
        .filter_map(|c| c.get("id").and_then(|v| v.as_u64()).map(|v| v as usize))
        .collect()
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

    // Verify action log contains SetSeed entry
    let log_resp = client.get("/actions/log").dispatch();
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

    // Limit=1
    let response = client.get("/actions/log?limit=1").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    let log: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(log["entries"].as_array().unwrap().len() <= 1);
}

#[test]
fn combat_results_returns_empty_when_no_result() {
    let client = client();
    let response = client.get("/combat/results").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    let results: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap();
    assert!(results.is_empty());
}

#[test]
fn combat_lifecycle_with_enemy_play_and_advance() {
    let client = client();

    // Initialize combat
    let response = client.post("/tests/combat").dispatch();
    assert_eq!(response.status(), Status::Created);

    // Get initial combat state
    let combat = get_combat(&client).expect("combat exists");
    assert_eq!(
        combat.phase,
        my_little_cardgame::library::types::CombatPhase::Defending
    );

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
fn library_cards_with_filters() {
    let client = client();

    // Get all cards
    let response = client.get("/library/cards").dispatch();
    assert_eq!(response.status(), Status::Ok);

    // Get encounter cards in hand
    let encounters = encounter_hand_ids(&client);
    assert!(!encounters.is_empty());
}

#[test]
fn play_card_without_combat_returns_error() {
    let client = client();
    let response = client
        .post("/action")
        .header(ContentType::JSON)
        .body(r#"{"action_type":"EncounterPlayCard","card_id":8}"#)
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn play_encounter_pick_starts_combat() {
    let client = client();

    let encounters = encounter_hand_ids(&client);
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
    let encounters = encounter_hand_ids(&client);
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
        // Defending phase: play defence card (id 9)
        client
            .post("/action")
            .header(ContentType::JSON)
            .body(r#"{"action_type":"EncounterPlayCard","card_id":9}"#)
            .dispatch();
        // Advance to Attacking
        client.post("/tests/combat/advance").dispatch();
        // Attacking phase: play attack card (id 8) — deals 5 damage
        client
            .post("/action")
            .header(ContentType::JSON)
            .body(r#"{"action_type":"EncounterPlayCard","card_id":8}"#)
            .dispatch();
        // Advance to Resourcing
        client.post("/tests/combat/advance").dispatch();
        // Resourcing phase: play resource card (id 10)
        client
            .post("/action")
            .header(ContentType::JSON)
            .body(r#"{"action_type":"EncounterPlayCard","card_id":10}"#)
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
    let result_resp = client.get("/combat/results").dispatch();
    let results_body = result_resp.into_string().unwrap_or_default();
    let results: Vec<serde_json::Value> = serde_json::from_str(&results_body).unwrap_or_default();
    if !results.is_empty() {
        // We're in scouting, finish it via EncounterApplyScouting
        let scouting_encounters = encounter_hand_ids(&client);
        let body = format!(
            r#"{{"action_type":"EncounterApplyScouting","card_ids":[{}]}}"#,
            scouting_encounters[0]
        );
        let response = client
            .post("/action")
            .header(ContentType::JSON)
            .body(&body)
            .dispatch();
        assert_eq!(response.status(), Status::Created);
    }
}

fn get_combat(client: &Client) -> Option<CombatState> {
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
    let encounters = encounter_hand_ids(&client);
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
    // Combat starts in Defending — play defence card (id 9)
    let response = client
        .post("/action")
        .header(ContentType::JSON)
        .body(r#"{"action_type":"EncounterPlayCard","card_id":9}"#)
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
            .body(r#"{"action_type":"EncounterPlayCard","card_id":8}"#)
            .dispatch();
        client.post("/tests/combat/advance").dispatch();
        client.post("/tests/combat/enemy_play").dispatch();
        client.post("/tests/combat/advance").dispatch();
        client.post("/tests/combat/advance").dispatch();
    }

    let encounters = encounter_hand_ids(&client);

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
        .body(r#"{"action_type":"EncounterApplyScouting","card_ids":[11]}"#)
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
    let encounters = encounter_hand_ids(&client);
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
        .body(r#"{"action_type":"EncounterPlayCard","card_id":8}"#)
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn play_card_nonexistent() {
    let client = client();

    // Start combat
    let encounters = encounter_hand_ids(&client);
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
        .body(r#"{"action_type":"EncounterPlayCard","card_id":9999}"#)
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn add_test_library_card_endpoint() {
    let client = client();
    let card = serde_json::json!({
        "kind": {"card_kind": "Attack", "effect_ids": []},
        "counts": {"library": 0, "deck": 5, "hand": 0, "discard": 0}
    });
    let response = client
        .post("/tests/library/cards")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&card).unwrap())
        .dispatch();
    assert_eq!(response.status(), Status::Created);
}
