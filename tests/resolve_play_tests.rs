use my_little_cardgame::rocket_initialize;
use rocket::http::uncased::Uncased;
use rocket::http::Header;
use rocket::http::Status;
use rocket::local::blocking::Client;
use std::borrow::Cow;

#[test]
fn test_play_defence_card_adds_tokens() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Initialize combat (starts in Defending)
    let init_response = client.post("/tests/combat").dispatch();
    assert_eq!(init_response.status(), Status::Created);

    // Play the existing Defence card (Library ID 1) which adds Dodge via player_data effects
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

    // Verify player tokens include Dodge
    let tokens_resp = client.get("/player/tokens").dispatch();
    assert_eq!(tokens_resp.status(), Status::Ok);
    let body = tokens_resp.into_string().expect("read tokens");
    assert!(body.contains("\"token_type\":\"Dodge\""));
}

#[test]
fn test_play_attack_card_kills_enemy() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Initialize combat and advance to Attacking
    client.post("/tests/combat").dispatch();
    client.post("/combat/advance").dispatch();

    // Play the existing Attack card (Library ID 0) which deals 20 damage (gnome has 20 HP)
    let action_json = r#"{ "action_type": "PlayCard", "card_id": 0 }"#;
    let play_response = client
        .post("/action")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(action_json)
        .dispatch();
    assert_eq!(play_response.status(), Status::Created);

    // Verify combat result is Player (enemy killed)
    let result_resp = client.get("/combat/result").dispatch();
    assert_eq!(result_resp.status(), Status::Ok);
    let body = result_resp.into_string().expect("read result");
    assert!(body.contains("Player"));
}
