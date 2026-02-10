use rocket::http::{Header, Status};
use rocket::local::blocking::Client;
use rocket::serde::json::serde_json;
use std::borrow::Cow;
use rocket::http::uncased::Uncased;
use my_little_cardgame::deck::{Card, Deck};
use my_little_cardgame::rocket_initialize;

#[test]
fn test_list_initial_cards() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    
    let response = client.get("/cards").dispatch();
    assert_eq!(response.status(), Status::Ok);
    
    let cards: Vec<Card> = serde_json::from_str(&response.into_string().unwrap()).unwrap();
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
    
    let response = client.post("/cards")
        .header(Header { name: Uncased::from("Content-Type"), value: Cow::from("application/json") })
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
    let response = client.post("/decks")
        .header(Header { name: Uncased::from("Content-Type"), value: Cow::from("application/json") })
        .body(deck_json)
        .dispatch();
    assert_eq!(response.status(), Status::Created);
}

#[test]
fn test_add_wrong_card_type_to_deck() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    
    let deck_json = r#"{ "contains_card_types": ["Attack"] }"#;
    let deck_response = client.post("/decks")
        .header(Header { name: Uncased::from("Content-Type"), value: Cow::from("application/json") })
        .body(deck_json)
        .dispatch();
    let deck_location = deck_response.headers().get_one("location").unwrap();
    
    let card_json = r#"{ "card_type_id": 1, "card_type": "Defence", "effects": [], "costs": [], "count": 10 }"#;
    let card_response = client.post("/cards")
        .header(Header { name: Uncased::from("Content-Type"), value: Cow::from("application/json") })
        .body(card_json)
        .dispatch();
    let card_location = card_response.headers().get_one("location").unwrap();
    let card_id: usize = card_location.trim_start_matches("/cards/").parse().unwrap();
    
    let deck_card_json = format!(r#"{{ "id": {}, "state": {{ "Deck": 10 }} }}"#, card_id);
    let add_response = client.post(format!("{}/cards", deck_location))
        .header(Header { name: Uncased::from("Content-Type"), value: Cow::from("application/json") })
        .body(deck_card_json)
        .dispatch();
    
    assert_eq!(add_response.status(), Status::BadRequest);
}
