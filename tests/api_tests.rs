use my_little_cardgame::deck::{Card, Deck};
use my_little_cardgame::rocket_initialize;
use rocket::http::uncased::Uncased;
use rocket::http::{Header, Status};
use rocket::local::blocking::Client;
use rocket::serde::json::serde_json;
use std::borrow::Cow;

#[test]
fn test_list_initial_cards() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let response = client.get("/cards").dispatch();
    assert_eq!(response.status(), Status::Ok);

    let cards: Vec<Card> = serde_json::from_str(&response.into_string().expect("Failed to read response body"))
        .expect("Failed to parse cards JSON");
    assert_eq!(cards.len(), 3);
}

#[test]
fn test_create_attack_card() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let card_json = r#"{
        "card_type_id": 1,
        "card_type": "Attack",
        "effects": [],
        "costs": [],
        "count": 10
    }"#;

    let response = client
        .post("/cards")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(card_json)
        .dispatch();

    assert_eq!(response.status(), Status::Created);
}

#[test]
fn test_get_nonexistent_card() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let response = client.get("/cards/99999").dispatch();
    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn test_create_deck() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let deck_json = r#"{ "contains_card_types": ["Attack"] }"#;
    let response = client
        .post("/decks")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(deck_json)
        .dispatch();
    assert_eq!(response.status(), Status::Created);
}

#[test]
fn test_add_wrong_card_type_to_deck() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let deck_json = r#"{ "contains_card_types": ["Attack"] }"#;
    let deck_response = client
        .post("/decks")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(deck_json)
        .dispatch();
    let deck_location = deck_response.headers().get_one("location").expect("Missing location header");

    let card_json =
        r#"{ "card_type_id": 1, "card_type": "Defence", "effects": [], "costs": [], "count": 10 }"#;
    let card_response = client
        .post("/cards")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(card_json)
        .dispatch();
    let card_location = card_response.headers().get_one("location").expect("Missing location header");
    let card_id: usize = card_location.trim_start_matches("/cards/").parse().expect("Invalid card ID");

    let deck_card_json = format!(r#"{{ "id": {}, "state": {{ "Deck": 10 }} }}"#, card_id);
    let add_response = client
        .post(format!("{}/cards", deck_location))
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(deck_card_json)
        .dispatch();

    assert_eq!(add_response.status(), Status::BadRequest);
}

// ============================================================================
// CRITICAL EDGE CASES - Combat State & Error Handling
// ============================================================================

#[test]
fn test_play_card_without_active_combat() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Try to play a card without initializing combat first
    let action_json = r#"{ "PlayCard": 0 }"#;

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
    client.post("/combat").dispatch();

    // Try to play a card that doesn't exist
    let action_json = r#"{ "PlayCard": 99999 }"#;

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
    assert_eq!(response.status(), Status::Ok);

    let combat_json = response.into_string().expect("Failed to read combat response");
    assert_eq!(combat_json, "null");
}

#[test]
fn test_initialize_combat_creates_attacking_state() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Initialize combat
    let init_response = client.post("/combat").dispatch();
    assert_eq!(init_response.status(), Status::Created);

    // Verify combat is in Attacking state
    let get_response = client.get("/combat").dispatch();
    let combat_str = get_response.into_string().expect("Failed to read combat response");

    assert!(combat_str.contains("\"state\":\"Attacking\""));
    assert!(combat_str.contains("\"enemies\""));
}

#[test]
fn test_add_duplicate_card_to_deck() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Create a deck
    let deck_json = r#"{ "contains_card_types": ["Attack"] }"#;
    let deck_response = client
        .post("/decks")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(deck_json)
        .dispatch();
    let deck_location = deck_response.headers().get_one("location").expect("Missing location header");

    // Create a card
    let card_json = r#"{
        "card_type_id": 1,
        "card_type": "Attack",
        "effects": [],
        "costs": [],
        "count": 10
    }"#;
    let card_response = client
        .post("/cards")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(card_json)
        .dispatch();
    let card_location = card_response.headers().get_one("location").expect("Missing location header");
    let card_id: usize = card_location.trim_start_matches("/cards/").parse().expect("Invalid card ID");

    // Add card to deck
    let deck_card_json = format!(r#"{{ "id": {}, "state": {{ "Deck": 10 }} }}"#, card_id);
    let add_response = client
        .post(format!("{}/cards", deck_location))
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(&deck_card_json)
        .dispatch();
    assert_eq!(add_response.status(), Status::Created);

    // Try to add the same card again
    let duplicate_response = client
        .post(format!("{}/cards", deck_location))
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(&deck_card_json)
        .dispatch();

    assert_eq!(duplicate_response.status(), Status::BadRequest);
}

#[test]
fn test_add_card_to_nonexistent_deck() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let deck_card_json = r#"{ "id": 0, "state": { "Deck": 10 } }"#;

    let response = client
        .post("/decks/99999/cards")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(deck_card_json)
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn test_create_card_with_zero_count_rejected() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    
    let invalid_card_json = r#"{
        "card_type_id": 1,
        "card_type": "Attack",
        "effects": [],
        "costs": [],
        "count": 0
    }"#;
    
    let response = client
        .post("/cards")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(invalid_card_json)
        .dispatch();
    
    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn test_create_deck_with_empty_card_types_rejected() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    
    let invalid_deck_json = r#"{ "contains_card_types": [] }"#;
    
    let response = client
        .post("/decks")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(invalid_deck_json)
        .dispatch();
    
    assert_eq!(response.status(), Status::BadRequest);
}
