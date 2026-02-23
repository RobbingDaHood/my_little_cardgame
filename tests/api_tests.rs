use my_little_cardgame::rocket_initialize;
use rocket::http::uncased::Uncased;
use rocket::http::{Header, Status};
use rocket::local::blocking::Client;

use std::borrow::Cow;

#[test]
fn test_play_card_without_active_combat() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Try to play a card without initializing combat first
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
fn test_play_nonexistent_card_in_combat() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Initialize combat
    client.post("/tests/combat").dispatch();

    // Try to play a card that doesn't exist
    let action_json = r#"{ "action_type": "EncounterPlayCard", "card_id": 99999 }"#;

    let response = client
        .post("/action")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(action_json)
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn test_get_combat_before_initialization() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let response = client.get("/combat").dispatch();
    assert_eq!(response.status(), Status::NotFound);
}
