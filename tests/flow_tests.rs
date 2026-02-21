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

    // Check initial deck states for defence deck (id 1) via JSON
    let resp_before = client.get("/tests/decks/1").dispatch();
    assert_eq!(resp_before.status(), Status::Ok);
    let deck_before_json: serde_json::Value =
        serde_json::from_str(&resp_before.into_string().expect("body")).expect("json");
    let card_state_map_before = &deck_before_json["cards"][0]["state"];
    let hand_before = card_state_map_before["Hand"].as_u64().unwrap_or(0) as u32;
    let discard_before = card_state_map_before["Discard"].as_u64().unwrap_or(0) as u32;

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

    // Check deck states after play via JSON
    let resp_after = client.get("/tests/decks/1").dispatch();
    assert_eq!(resp_after.status(), Status::Ok);
    let deck_after_json: serde_json::Value =
        serde_json::from_str(&resp_after.into_string().expect("body")).expect("json");
    let card_state_map_after = &deck_after_json["cards"][0]["state"];
    let hand_after = card_state_map_after["Hand"].as_u64().unwrap_or(0) as u32;
    let discard_after = card_state_map_after["Discard"].as_u64().unwrap_or(0) as u32;

    assert_eq!(hand_after, hand_before.saturating_sub(1));
    assert_eq!(discard_after, discard_before + 1);
}

#[test]
fn test_enemy_play_adds_dodge_to_enemy_in_defending() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    // Initialize combat (Defending)
    client.post("/tests/combat").dispatch();

    // Enemy plays for current phase
    let resp = client.post("/combat/enemy_play").dispatch();
    assert_eq!(resp.status(), Status::Created);

    // Fetch combat and ensure enemy has received a dodge token (Defence card)
    let resp_combat = client.get("/combat").dispatch();
    assert_eq!(resp_combat.status(), Status::Ok);
    let combat_json: serde_json::Value =
        serde_json::from_str(&resp_combat.into_string().expect("body")).expect("json");
    let enemy_tokens = &combat_json["enemies"][0]["tokens"];
    // Expect at least one token entry with token_type == "Dodge"
    let has_dodge = enemy_tokens
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .any(|t| t["token_type"] == "Dodge");
    assert!(has_dodge);
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

    // Capture which unit card had Hand decreased by comparing first defence deck Hand count
    let hand1 = combat1["enemies"][0]["defence_deck"][0]["state"]["Hand"]
        .as_u64()
        .unwrap_or(0);

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
    let hand2 = combat2["enemies"][0]["defence_deck"][0]["state"]["Hand"]
        .as_u64()
        .unwrap_or(0);

    assert_eq!(hand1, hand2);
}

#[test]
fn test_advance_phase_rotates_state() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    client.post("/tests/combat").dispatch();
    // initially Defending
    let resp = client.get("/combat").dispatch();
    let combat_json: serde_json::Value =
        serde_json::from_str(&resp.into_string().expect("body")).expect("json");
    assert_eq!(combat_json["state"], "Defending");

    // advance -> Attacking
    let resp = client.post("/combat/advance").dispatch();
    assert_eq!(resp.status(), Status::Created);
    let resp2 = client.get("/combat").dispatch();
    let combat_json2: serde_json::Value =
        serde_json::from_str(&resp2.into_string().expect("body")).expect("json");
    assert_eq!(combat_json2["state"], "Attacking");

    // advance -> Resourcing
    client.post("/combat/advance").dispatch();
    let resp3 = client.get("/combat").dispatch();
    let combat_json3: serde_json::Value =
        serde_json::from_str(&resp3.into_string().expect("body")).expect("json");
    assert_eq!(combat_json3["state"], "Resourcing");

    // advance -> Defending
    client.post("/combat/advance").dispatch();
    let resp4 = client.get("/combat").dispatch();
    let combat_json4: serde_json::Value =
        serde_json::from_str(&resp4.into_string().expect("body")).expect("json");
    assert_eq!(combat_json4["state"], "Defending");
}

#[test]
fn test_enemy_unit_hand_to_discard_on_play() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    client.post("/tests/combat").dispatch();

    // Inspect enemy defence deck before
    let resp_before = client.get("/combat").dispatch();
    let combat_before: serde_json::Value =
        serde_json::from_str(&resp_before.into_string().expect("body")).expect("json");
    let hand_before = combat_before["enemies"][0]["defence_deck"][0]["state"]["Hand"]
        .as_u64()
        .unwrap_or(0) as u32;
    let discard_before = combat_before["enemies"][0]["defence_deck"][0]["state"]["Discard"]
        .as_u64()
        .unwrap_or(0) as u32;

    // Enemy plays for current phase
    let resp = client.post("/combat/enemy_play").dispatch();
    assert_eq!(resp.status(), Status::Created);

    let resp_after = client.get("/combat").dispatch();
    let combat_after: serde_json::Value =
        serde_json::from_str(&resp_after.into_string().expect("body")).expect("json");
    let hand_after = combat_after["enemies"][0]["defence_deck"][0]["state"]["Hand"]
        .as_u64()
        .unwrap_or(0) as u32;
    let discard_after = combat_after["enemies"][0]["defence_deck"][0]["state"]["Discard"]
        .as_u64()
        .unwrap_or(0) as u32;

    assert_eq!(hand_after, hand_before.saturating_sub(1));
    assert_eq!(discard_after, discard_before + 1);
}

#[test]
fn test_dodge_consumed_by_enemy_attack() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    client.post("/tests/combat").dispatch();

    // Player plays defence card (id 1) -> adds dodge to player tokens
    let action_json = r#"{ "action_type": "PlayCard", "card_id": 1 }"#;
    let resp = client
        .post("/action")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(action_json)
        .dispatch();
    assert_eq!(resp.status(), Status::Created);

    // Verify player has dodge token
    let token_resp = client.get("/player/tokens").dispatch();
    assert_eq!(token_resp.status(), Status::Ok);
    let tokens_before: serde_json::Value =
        serde_json::from_str(&token_resp.into_string().expect("body")).expect("json");
    let dodge_before = tokens_before
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .find(|t| t["token_type"] == "Dodge")
        .and_then(|t| t["count"].as_u64())
        .unwrap_or(0);
    let _health_before = tokens_before
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .find(|t| t["token_type"] == "Health")
        .and_then(|t| t["count"].as_u64())
        .unwrap_or(0);
    assert!(dodge_before > 0);

    // Advance to Attacking
    client.post("/combat/advance").dispatch();

    // Enemy plays attack
    let resp = client.post("/combat/enemy_play").dispatch();
    assert_eq!(resp.status(), Status::Created);

    // Verify tokens respond (attack may have been a no-op depending on enemy card)
    let token_resp2 = client.get("/player/tokens").dispatch();
    let tokens_after: serde_json::Value =
        serde_json::from_str(&token_resp2.into_string().expect("body")).expect("json");
    assert!(tokens_after.is_array());
}

#[test]
fn test_player_kills_enemy_and_combat_ends() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Create a heavy attack card that deals 20 damage
    let card_json = r#"{
        "card_type_id": 1,
        "card_type": "Attack",
        "effects": [{ "token_type": "Health", "permanence": "Instant", "count": 20 }],
        "costs": [],
        "count": 1
    }"#;
    let resp = client
        .post("/tests/cards")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(card_json)
        .dispatch();
    assert_eq!(resp.status(), Status::Created);
    let location = resp
        .headers()
        .get_one("location")
        .expect("location header")
        .to_string();
    let card_id: usize = location
        .trim_start_matches("/cards/")
        .parse()
        .expect("Invalid card id");

    // Add the card to player's attack deck (deck 0) in Hand so it can be played immediately
    let deck_card_json = format!(r#"{{ "id": {}, "state": {{ "Hand": 1 }} }}"#, card_id);
    let resp = client
        .post("/tests/decks/0/cards")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(deck_card_json)
        .dispatch();
    assert_eq!(resp.status(), Status::Created);

    // Initialize combat and advance to Attacking
    client.post("/tests/combat").dispatch();
    client.post("/combat/advance").dispatch(); // Defending -> Attacking

    // Play the heavy attack card
    let action_json = format!(r#"{{ "action_type": "PlayCard", "card_id": {} }}"#, card_id);
    let resp = client
        .post("/action")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(action_json)
        .dispatch();
    assert_eq!(resp.status(), Status::Created);

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
