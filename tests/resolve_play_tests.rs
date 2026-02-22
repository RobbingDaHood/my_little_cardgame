use my_little_cardgame::rocket_initialize;
use rocket::http::uncased::Uncased;
use rocket::http::Header;
use rocket::http::Status;
use rocket::local::blocking::Client;
use rocket::serde::json::serde_json;
use std::borrow::Cow;

#[test]
fn test_play_defence_card_adds_tokens() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Initialize combat (starts in Defending)
    let init_response = client.post("/tests/combat").dispatch();
    assert_eq!(init_response.status(), Status::Created);

    // Play the existing Defence card (Library ID 1) which adds shield via CardEffect
    let action_json = r#"{ "action_type": "PlayCard", "card_id": 1 }"#;
    let play_response = client
        .post("/action")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(action_json)
        .dispatch();
    assert_eq!(play_response.status(), Status::Created);

    // Verify combat snapshot has player shield tokens
    let combat_resp = client.get("/combat").dispatch();
    assert_eq!(combat_resp.status(), Status::Ok);
    let body = combat_resp.into_string().expect("read combat");
    let combat_json: serde_json::Value = serde_json::from_str(&body).expect("parse json");
    let shield = combat_json["player_tokens"]["Shield"].as_i64().unwrap_or(0);
    assert!(
        shield > 0,
        "Player should have shield tokens after defence play"
    );
}

#[test]
fn test_play_attack_card_kills_enemy() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Initialize combat and advance to Attacking
    client.post("/tests/combat").dispatch();
    client.post("/combat/advance").dispatch();

    // Play the existing Attack card 4 times (Library ID 0, deals 5 damage each, gnome has 20 HP)
    let action_json = r#"{ "action_type": "PlayCard", "card_id": 0 }"#;
    for _ in 0..4 {
        let play_response = client
            .post("/action")
            .header(Header {
                name: Uncased::from("Content-Type"),
                value: Cow::from("application/json"),
            })
            .body(action_json)
            .dispatch();
        assert_eq!(play_response.status(), Status::Created);
    }

    // Verify combat result is Player (enemy killed)
    let result_resp = client.get("/combat/result").dispatch();
    assert_eq!(result_resp.status(), Status::Ok);
    let body = result_resp.into_string().expect("read result");
    assert!(body.contains("Player"));
}
