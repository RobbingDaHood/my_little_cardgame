use my_little_cardgame::rocket_initialize;
use rocket::http::uncased::Uncased;
use rocket::http::{Header, Status};
use rocket::local::blocking::Client;
use rocket::serde::json::serde_json;
use std::borrow::Cow;

#[test]
fn test_phase_enforcement_attack_in_defending_should_fail() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    // Initialize combat (starts in Defending)
    let init_response = client.post("/tests/combat").dispatch();
    assert_eq!(init_response.status(), Status::Created);

    // Try to play an Attack card (id 0) while in Defending phase -> should be BadRequest
    let action_json = r#"{ "action_type": "PlayCard", "card_id": 0 }"#;
    let response = client
        .post("/action")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(action_json)
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn test_play_defence_moves_card_to_discard() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    // Initialize combat (Defending)
    client.post("/tests/combat").dispatch();

    // Check initial Library card state for defence card (id 1)
    let resp_before = client.get("/library/cards").dispatch();
    assert_eq!(resp_before.status(), Status::Ok);
    let cards_before: serde_json::Value =
        serde_json::from_str(&resp_before.into_string().expect("body")).expect("json");
    let hand_before = cards_before[1]["counts"]["hand"].as_u64().unwrap_or(0) as u32;
    let discard_before = cards_before[1]["counts"]["discard"].as_u64().unwrap_or(0) as u32;

    // Play a defence card (id 1)
    let action_json = r#"{ "action_type": "PlayCard", "card_id": 1 }"#;
    let response = client
        .post("/action")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(action_json)
        .dispatch();
    assert_eq!(response.status(), Status::Created);

    // Check Library card state after play
    let resp_after = client.get("/library/cards").dispatch();
    assert_eq!(resp_after.status(), Status::Ok);
    let cards_after: serde_json::Value =
        serde_json::from_str(&resp_after.into_string().expect("body")).expect("json");
    let hand_after = cards_after[1]["counts"]["hand"].as_u64().unwrap_or(0) as u32;
    let discard_after = cards_after[1]["counts"]["discard"].as_u64().unwrap_or(0) as u32;

    assert_eq!(hand_after, hand_before.saturating_sub(1));
    assert_eq!(discard_after, discard_before + 1);
}

#[test]
fn test_enemy_play_adds_shield_to_enemy_in_defending() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    // Initialize combat (Defending)
    client.post("/tests/combat").dispatch();

    // Enemy plays for current phase (Defending → enemy plays defence card → shield)
    let resp = client.post("/combat/enemy_play").dispatch();
    assert_eq!(resp.status(), Status::Created);

    // Fetch combat and check enemy tokens via CombatSnapshot JSON
    let resp_combat = client.get("/combat").dispatch();
    assert_eq!(resp_combat.status(), Status::Ok);
    let combat_json: serde_json::Value =
        serde_json::from_str(&resp_combat.into_string().expect("body")).expect("json");
    // Enemy tokens are in enemy.active_tokens as a map
    let enemy_tokens = &combat_json["enemy"]["active_tokens"];
    // Expect shield token > 0 (defence card grants shield)
    let shield = enemy_tokens["Shield"].as_i64().unwrap_or(0);
    assert!(
        shield > 0,
        "Enemy should have shield tokens after defence play"
    );
}

#[test]
fn test_seed_determinism_for_enemy_selection() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    // First run: init, set seed, enemy_play
    client.post("/tests/combat").dispatch();
    let seed_body = r#"{ "seed": 42 }"#;
    let resp = client
        .post("/player/seed")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(seed_body)
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    client.post("/combat/enemy_play").dispatch();
    let resp1 = client.get("/combat").dispatch();
    let combat1: serde_json::Value =
        serde_json::from_str(&resp1.into_string().expect("body")).expect("json");

    // Capture enemy tokens after first play
    let enemy_tokens_1 = combat1["enemy"]["active_tokens"].clone();

    // Second run: reinitialize, set same seed, enemy_play
    client.post("/tests/combat").dispatch();
    let resp = client
        .post("/player/seed")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(seed_body)
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    client.post("/combat/enemy_play").dispatch();
    let resp2 = client.get("/combat").dispatch();
    let combat2: serde_json::Value =
        serde_json::from_str(&resp2.into_string().expect("body")).expect("json");
    let enemy_tokens_2 = combat2["enemy"]["active_tokens"].clone();

    assert_eq!(enemy_tokens_1, enemy_tokens_2);
}

#[test]
fn test_advance_phase_rotates_state() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    client.post("/tests/combat").dispatch();
    // initially Defending
    let resp = client.get("/combat").dispatch();
    let combat_json: serde_json::Value =
        serde_json::from_str(&resp.into_string().expect("body")).expect("json");
    assert_eq!(combat_json["phase"], "Defending");

    // advance -> Attacking
    let resp = client.post("/combat/advance").dispatch();
    assert_eq!(resp.status(), Status::Created);
    let resp2 = client.get("/combat").dispatch();
    let combat_json2: serde_json::Value =
        serde_json::from_str(&resp2.into_string().expect("body")).expect("json");
    assert_eq!(combat_json2["phase"], "Attacking");

    // advance -> Resourcing
    client.post("/combat/advance").dispatch();
    let resp3 = client.get("/combat").dispatch();
    let combat_json3: serde_json::Value =
        serde_json::from_str(&resp3.into_string().expect("body")).expect("json");
    assert_eq!(combat_json3["phase"], "Resourcing");

    // advance -> Defending
    client.post("/combat/advance").dispatch();
    let resp4 = client.get("/combat").dispatch();
    let combat_json4: serde_json::Value =
        serde_json::from_str(&resp4.into_string().expect("body")).expect("json");
    assert_eq!(combat_json4["phase"], "Defending");
}

#[test]
fn test_enemy_play_applies_effects() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    client.post("/tests/combat").dispatch();

    // Get enemy tokens before play
    let resp_before = client.get("/combat").dispatch();
    let combat_before: serde_json::Value =
        serde_json::from_str(&resp_before.into_string().expect("body")).expect("json");
    let shield_before = combat_before["enemy"]["active_tokens"]["Shield"]
        .as_i64()
        .unwrap_or(0);

    // Enemy plays in Defending phase → defence card grants shield
    let resp = client.post("/combat/enemy_play").dispatch();
    assert_eq!(resp.status(), Status::Created);

    let resp_after = client.get("/combat").dispatch();
    let combat_after: serde_json::Value =
        serde_json::from_str(&resp_after.into_string().expect("body")).expect("json");
    let shield_after = combat_after["enemy"]["active_tokens"]["Shield"]
        .as_i64()
        .unwrap_or(0);

    // Enemy defence card grants 2 shield
    assert_eq!(shield_after, shield_before + 2);
}

#[test]
fn test_player_card_damages_enemy() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    client.post("/tests/combat").dispatch();

    // Get enemy health before attack
    let resp_before = client.get("/combat").dispatch();
    let combat_before: serde_json::Value =
        serde_json::from_str(&resp_before.into_string().expect("body")).expect("json");
    let health_before = combat_before["enemy"]["active_tokens"]["Health"]
        .as_i64()
        .unwrap_or(0);

    // Advance to Attacking
    client.post("/combat/advance").dispatch();

    // Play attack card (id 0, deals 5 damage)
    let action_json = r#"{ "action_type": "PlayCard", "card_id": 0 }"#;
    let resp = client
        .post("/action")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(action_json)
        .dispatch();
    assert_eq!(resp.status(), Status::Created);

    // Enemy should have lost 5 health
    let resp_after = client.get("/combat").dispatch();
    let combat_after: serde_json::Value =
        serde_json::from_str(&resp_after.into_string().expect("body")).expect("json");
    let health_after = combat_after["enemy"]["active_tokens"]["Health"]
        .as_i64()
        .unwrap_or(0);

    assert_eq!(health_after, health_before - 5);
}

#[test]
fn test_player_kills_enemy_and_combat_ends() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Initialize combat and advance to Attacking
    client.post("/tests/combat").dispatch();
    client.post("/combat/advance").dispatch(); // Defending -> Attacking

    // Play the attack card 4 times (Library id 0, deals 5 damage each, gnome has 20 HP)
    let action_json = r#"{ "action_type": "PlayCard", "card_id": 0 }"#;
    for _ in 0..4 {
        let resp = client
            .post("/action")
            .header(Header {
                name: Uncased::from("Content-Type"),
                value: Cow::from("application/json"),
            })
            .body(action_json)
            .dispatch();
        assert_eq!(resp.status(), Status::Created);
    }

    // Combat should be ended (GET /combat -> 404)
    let resp = client.get("/combat").dispatch();
    assert_eq!(resp.status(), Status::NotFound);

    // Combat result should be recorded
    let resp = client.get("/combat/result").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let result_json: serde_json::Value =
        serde_json::from_str(&resp.into_string().expect("body")).expect("json");
    assert_eq!(result_json["winner"], "Player");
}
