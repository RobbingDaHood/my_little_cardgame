use my_little_cardgame::rocket_initialize;
use rocket::http::uncased::Uncased;
use rocket::http::{Header, Status};
use rocket::local::blocking::Client;
use rocket::serde::json::serde_json;
use std::borrow::Cow;

/// Extract a token value from the serialized token map format.
fn token_value(tokens: &serde_json::Value, token_type: &str) -> i64 {
    tokens
        .as_object()
        .and_then(|obj| obj.get(token_type).and_then(|v| v.as_i64()))
        .unwrap_or(0)
}

#[test]
fn test_phase_enforcement_attack_in_defending_should_fail() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    // Initialize combat (starts in Defending)
    let init_response = client.post("/tests/combat").dispatch();
    assert_eq!(init_response.status(), Status::Created);

    // Try to play an Attack card (id 0) while in Defending phase -> should be BadRequest
    let action_json = r#"{ "action_type": "EncounterPlayCard", "card_id": 0 }"#;
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
    let action_json = r#"{ "action_type": "EncounterPlayCard", "card_id": 1 }"#;
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
    let resp = client.post("/tests/combat/enemy_play").dispatch();
    assert_eq!(resp.status(), Status::Created);

    // Fetch combat and check enemy tokens via CombatSnapshot JSON
    let resp_combat = client.get("/combat").dispatch();
    assert_eq!(resp_combat.status(), Status::Ok);
    let combat_json: serde_json::Value =
        serde_json::from_str(&resp_combat.into_string().expect("body")).expect("json");
    // Enemy tokens are in enemy.active_tokens as an array
    let enemy_tokens = &combat_json["enemy"]["active_tokens"];
    let shield = token_value(enemy_tokens, "Shield");
    assert!(
        shield > 0,
        "Enemy should have shield tokens after defence play"
    );
}

#[test]
fn test_seed_determinism_for_enemy_selection() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    // First run: init, set seed via NewGame, enemy_play
    client.post("/tests/combat").dispatch();
    client
        .post("/action")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(r#"{"action_type":"NewGame","seed":42}"#)
        .dispatch();
    client.post("/tests/combat").dispatch();
    client.post("/tests/combat/enemy_play").dispatch();
    let resp1 = client.get("/combat").dispatch();
    let combat1: serde_json::Value =
        serde_json::from_str(&resp1.into_string().expect("body")).expect("json");

    // Capture enemy tokens after first play
    let enemy_tokens_1 = combat1["enemy"]["active_tokens"].clone();

    // Second run: reinitialize, set same seed via NewGame, enemy_play
    client
        .post("/action")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(r#"{"action_type":"NewGame","seed":42}"#)
        .dispatch();
    client.post("/tests/combat").dispatch();
    client.post("/tests/combat/enemy_play").dispatch();
    let resp2 = client.get("/combat").dispatch();
    let combat2: serde_json::Value =
        serde_json::from_str(&resp2.into_string().expect("body")).expect("json");
    let enemy_tokens_2 = combat2["enemy"]["active_tokens"].clone();

    // Compare individual token values (order may differ due to HashMap iteration)
    assert_eq!(
        token_value(&enemy_tokens_1, "Health"),
        token_value(&enemy_tokens_2, "Health")
    );
    assert_eq!(
        token_value(&enemy_tokens_1, "Shield"),
        token_value(&enemy_tokens_2, "Shield")
    );
    assert_eq!(
        token_value(&enemy_tokens_1, "MaxHealth"),
        token_value(&enemy_tokens_2, "MaxHealth")
    );
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
    let resp = client.post("/tests/combat/advance").dispatch();
    assert_eq!(resp.status(), Status::Created);
    let resp2 = client.get("/combat").dispatch();
    let combat_json2: serde_json::Value =
        serde_json::from_str(&resp2.into_string().expect("body")).expect("json");
    assert_eq!(combat_json2["phase"], "Attacking");

    // advance -> Resourcing
    client.post("/tests/combat/advance").dispatch();
    let resp3 = client.get("/combat").dispatch();
    let combat_json3: serde_json::Value =
        serde_json::from_str(&resp3.into_string().expect("body")).expect("json");
    assert_eq!(combat_json3["phase"], "Resourcing");

    // advance -> Defending
    client.post("/tests/combat/advance").dispatch();
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
    let shield_before = token_value(&combat_before["enemy"]["active_tokens"], "Shield");

    // Enemy plays in Defending phase → defence card grants shield
    let resp = client.post("/tests/combat/enemy_play").dispatch();
    assert_eq!(resp.status(), Status::Created);

    let resp_after = client.get("/combat").dispatch();
    let combat_after: serde_json::Value =
        serde_json::from_str(&resp_after.into_string().expect("body")).expect("json");
    let shield_after = token_value(&combat_after["enemy"]["active_tokens"], "Shield");

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
    let health_before = token_value(&combat_before["enemy"]["active_tokens"], "Health");

    // Advance to Attacking
    client.post("/tests/combat/advance").dispatch();

    // Play attack card (id 0, deals 5 damage)
    let action_json = r#"{ "action_type": "EncounterPlayCard", "card_id": 0 }"#;
    let resp = client
        .post("/action")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(action_json)
        .dispatch();
    assert_eq!(resp.status(), Status::Created);

    // Enemy should have lost at least 5 health (may have also been hit by enemy actions)
    let resp_after = client.get("/combat").dispatch();
    if resp_after.status() == Status::Ok {
        let combat_after: serde_json::Value =
            serde_json::from_str(&resp_after.into_string().expect("body")).expect("json");
        let health_after = token_value(&combat_after["enemy"]["active_tokens"], "Health");
        assert!(health_after < health_before);
    }
    // If combat ended, that's fine too — the attack dealt enough damage
}

#[test]
fn test_player_kills_enemy_and_combat_ends() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Initialize combat (starts in Defending)
    client.post("/tests/combat").dispatch();

    // Play cards cycling through phases until combat ends
    // Phase cycle: Defending(1) -> Attacking(0) -> Resourcing(2) -> Defending(1) ...
    // Card ids: 0=Attack, 1=Defence, 2=Resource
    let phase_cards = [1, 0, 2]; // Defence, Attack, Resource
    for (phase_idx, _) in (0..30).enumerate() {
        let card_id = phase_cards[phase_idx % 3];
        let action_json = format!(
            r#"{{ "action_type": "EncounterPlayCard", "card_id": {} }}"#,
            card_id
        );
        let resp = client
            .post("/action")
            .header(Header {
                name: Uncased::from("Content-Type"),
                value: Cow::from("application/json"),
            })
            .body(&action_json)
            .dispatch();
        if resp.status() != Status::Created {
            break;
        }
        // Check if combat ended
        let combat_resp = client.get("/combat").dispatch();
        if combat_resp.status() == Status::NotFound {
            break;
        }
    }

    // Combat should be ended (GET /combat -> 404)
    let resp = client.get("/combat").dispatch();
    assert_eq!(resp.status(), Status::NotFound);

    // Combat result should be recorded
    let resp = client.get("/combat/result").dispatch();
    assert_eq!(resp.status(), Status::Ok);
}
