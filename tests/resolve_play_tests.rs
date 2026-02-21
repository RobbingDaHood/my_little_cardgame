use my_little_cardgame::rocket_initialize;
use rocket::http::uncased::Uncased;
use rocket::http::Header;
use rocket::http::Status;
use rocket::local::blocking::Client;
use std::borrow::Cow;

#[test]
fn test_play_defence_card_adds_tokens() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Create a Defence card that grants Dodge and Stamina
    let card_json = r#"{
        "card_type_id": 1,
        "card_type": "Defence",
        "effects": [
            {"token_type":"Dodge","permanence":"UsedOnUnit","count":2},
            {"token_type":"Stamina","permanence":"Instant","count":3}
        ],
        "costs": [],
        "count": 1
    }"#;

    let response = client
        .post("/tests/cards")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(card_json)
        .dispatch();
    assert_eq!(response.status(), Status::Created);
    let card_location = response
        .headers()
        .get_one("location")
        .expect("Missing location header");
    let card_id: usize = card_location
        .trim_start_matches("/cards/")
        .parse()
        .expect("Invalid card ID");

    // Use the default defence deck
    let deck_location = "/tests/decks/1".to_string();

    // Add the card to the deck with one card in Hand so it can be played
    let deck_card_json = format!(r#"{{ "id": {}, "state": {{ "Hand": 1 }} }}"#, card_id);
    let add_response = client
        .post(format!("{}/cards", deck_location))
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(deck_card_json)
        .dispatch();
    assert_eq!(add_response.status(), Status::Created);

    // Initialize combat (starts in Defending)
    let init_response = client.post("/tests/combat").dispatch();
    assert_eq!(init_response.status(), Status::Created);

    // Play the card
    let action_json = format!(r#"{{ "PlayCard": {} }}"#, card_id);
    let play_response = client
        .post("/action")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(action_json)
        .dispatch();
    let status = play_response.status();
    if status != Status::Created {
        let b = play_response.into_string().unwrap_or_default();
        println!("POST /action returned status {:?} body={} ", status, b);
        panic!("action failed");
    }
    assert_eq!(status, Status::Created);

    // Verify player tokens include Dodge and Stamina
    let tokens_resp = client.get("/player/tokens").dispatch();
    assert_eq!(tokens_resp.status(), Status::Ok);
    let body = tokens_resp.into_string().expect("read tokens");
    assert!(body.contains("\"token_type\":\"Dodge\""));
    assert!(body.contains("\"token_type\":\"Stamina\""));
}

#[test]
fn test_play_defence_card_health_kills_enemy() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Create a Defence card that deals heavy Health damage
    let card_json = r#"{
        "card_type_id": 1,
        "card_type": "Defence",
        "effects": [
            {"token_type":"Health","permanence":"Instant","count":100}
        ],
        "costs": [],
        "count": 1
    }"#;

    let response = client
        .post("/tests/cards")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(card_json)
        .dispatch();
    assert_eq!(response.status(), Status::Created);
    let card_location = response
        .headers()
        .get_one("location")
        .expect("Missing location header");
    let card_id: usize = card_location
        .trim_start_matches("/cards/")
        .parse()
        .expect("Invalid card ID");

    // Use the default defence deck
    let deck_location = "/tests/decks/1".to_string();

    // Add the card to the deck with one card in Hand so it can be played
    let deck_card_json = format!(r#"{{ "id": {}, "state": {{ "Hand": 1 }} }}"#, card_id);
    let add_response = client
        .post(format!("{}/cards", deck_location))
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(deck_card_json)
        .dispatch();
    assert_eq!(add_response.status(), Status::Created);

    // Initialize combat (starts in Defending)
    let init_response = client.post("/tests/combat").dispatch();
    assert_eq!(init_response.status(), Status::Created);

    // Play the card
    let action_json = format!(r#"{{ "PlayCard": {} }}"#, card_id);
    let play_response = client
        .post("/action")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(action_json)
        .dispatch();
    let status = play_response.status();
    if status != Status::Created {
        let b = play_response.into_string().unwrap_or_default();
        println!("POST /action returned status {:?} body={} ", status, b);
        panic!("action failed");
    }
    assert_eq!(status, Status::Created);

    // Verify combat result is Player (enemy killed)
    let result_resp = client.get("/combat/result").dispatch();
    assert_eq!(result_resp.status(), Status::Ok);
    let body = result_resp.into_string().expect("read result");
    assert!(body.contains("Player"));
}
