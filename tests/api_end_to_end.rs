use log::debug;
use my_little_cardgame::action::PlayerActions::PlayCard;
use my_little_cardgame::combat::units::get_gnome;
use my_little_cardgame::combat::Combat;
use my_little_cardgame::combat::States::Defending;
use my_little_cardgame::deck::card::CardType;
use my_little_cardgame::deck::{Card, CardState, CreateDeck};
use rocket::http::uncased::Uncased;
use rocket::http::{Header, Status};
use rocket::local::blocking::Client;
use rocket::serde::json::serde_json;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;

use my_little_cardgame::rocket_initialize;
use my_little_cardgame::status_messages::new_status;
use my_little_cardgame::status_messages::Status as MyStatus;

#[derive(Debug, Deserialize)]
struct DeckCardJson {
    id: usize,
    state: HashMap<CardState, u32>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct DeckJson {
    cards: Vec<DeckCardJson>,
    id: usize,
    contains_card_types: Vec<CardType>,
}

#[derive(Serialize)]
struct DeckCardPayload {
    id: usize,
    state: HashMap<CardState, u32>,
}

#[test]
fn hello_world() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let list_of_cards = get_cards(&client);
    assert_eq!(3, list_of_cards.len());

    let new_card_attack = get_attack_card();
    let location_header_card_attack = post_card(&client, &new_card_attack);
    assert_eq!("/cards/3", location_header_card_attack);

    let new_card_ressource = get_ressource_card();
    let location_header_card_ressource = post_card(&client, &new_card_ressource);
    assert_eq!("/cards/4", location_header_card_ressource);

    let card_id_attack = get_card(&client, location_header_card_attack);
    let card_id_ressource = get_card(&client, location_header_card_ressource);

    get_decks(&client, 3);

    check_deck_card_states(&client, "/tests/decks/0", &CardState::Deck, 35);
    check_deck_card_states(&client, "/tests/decks/0", &CardState::Hand, 5);
    check_deck_card_states(&client, "/tests/decks/1", &CardState::Deck, 35);
    check_deck_card_states(&client, "/tests/decks/1", &CardState::Hand, 5);
    check_deck_card_states(&client, "/tests/decks/2", &CardState::Deck, 35);
    check_deck_card_states(&client, "/tests/decks/2", &CardState::Hand, 5);

    let location_header_deck = post_deck(&client);
    assert_eq!("/tests/decks/3", location_header_deck);

    get_decks(&client, 4);

    let created_deck = get_deck(&client, location_header_deck.clone());
    assert_eq!(0, created_deck.cards.len());

    let deck_card = DeckCardPayload {
        id: card_id_attack,
        state: HashMap::from([(CardState::Deck, 20)]),
    };
    let location_header_card_in_deck = post_card_to_deck(&client, created_deck.id, &deck_card);
    assert_eq!("/tests/decks/3/cards/3", location_header_card_in_deck);

    let deck_card = DeckCardPayload {
        id: card_id_ressource,
        state: HashMap::from([(CardState::Deck, 20)]),
    };
    post_card_to_deck_fail_on_type(&client, created_deck.id, &deck_card,
                                   "Card with id 4 is of type Resource and that is not part of the types '[Attack]' allowed in deck with id 3");

    let card_in_deck = get_card_in_deck(&client, location_header_card_in_deck.clone());
    assert_eq!(card_in_deck.id, card_id_attack);
    assert_eq!(
        *card_in_deck
            .state
            .get(&CardState::Deck)
            .expect("Test assertion failed"),
        20
    );

    let created_deck = get_deck(&client, location_header_deck.clone());
    assert_eq!(1, created_deck.cards.len());
    assert_eq!(
        0,
        created_deck
            .cards
            .iter()
            .filter(|card| card.state.contains_key(&CardState::Deleted))
            .count()
    );

    delete_card_in_deck(&client, location_header_card_in_deck);

    let created_deck = get_deck(&client, location_header_deck);
    assert_eq!(1, created_deck.cards.len());
    assert_eq!(
        1,
        created_deck
            .cards
            .iter()
            .filter(|card| card.state.contains_key(&CardState::Deleted))
            .count()
    );

    assert_eq!(get_combat(&client), None);

    initialize_combat(&client);

    let actual = get_combat(&client);
    assert!(actual.is_some());
    let actual_combat = actual.unwrap();
    assert_eq!(actual_combat.state, Defending);
    assert_eq!(actual_combat.allies.len(), 0);
    assert_eq!(actual_combat.enemies.len(), 1);
    let expected_enemy = get_gnome();
    let actual_enemy = &actual_combat.enemies[0];
    assert_eq!(
        actual_enemy.attack_deck.len(),
        expected_enemy.attack_deck.len()
    );
    assert_eq!(
        actual_enemy.defence_deck.len(),
        expected_enemy.defence_deck.len()
    );
    assert_eq!(
        actual_enemy.resource_deck.len(),
        expected_enemy.resource_deck.len()
    );
    assert_eq!(actual_enemy.tokens.len(), expected_enemy.tokens.len());
}

fn check_deck_card_states(client: &Client, location: &str, card_state: &CardState, count: u32) {
    let first_deck = get_deck(client, location.to_string());
    assert_eq!(1, first_deck.cards.len());
    debug!("{:?}", first_deck);
    let decked_cards_count = first_deck
        .cards
        .first()
        .expect("Test assertion failed")
        .state
        .get(card_state)
        .expect("Test assertion failed");
    assert_eq!(count, *decked_cards_count);
}

fn get_ressource_card() -> serde_json::Value {
    serde_json::json!({
        "card_type_id": 1,
        "card_type": "Resource",
        "effects": [
            {
                "token_type": "Health",
                "count": 1,
                "permanence": { "Permanent": { "max_count": 20 } }
            }
        ],
        "costs": [
            {
                "token_type": "Mana",
                "count": 1,
                "permanence": { "Permanent": { "max_count": 20 } }
            }
        ],
        "count": 22
    })
}

fn get_attack_card() -> serde_json::Value {
    serde_json::json!({
        "card_type_id": 1,
        "card_type": "Attack",
        "effects": [
            {
                "token_type": "Health",
                "count": 1,
                "permanence": { "Permanent": { "max_count": 20 } }
            }
        ],
        "costs": [
            {
                "token_type": "Mana",
                "count": 1,
                "permanence": { "Permanent": { "max_count": 20 } }
            }
        ],
        "count": 22
    })
}

fn get_decks(client: &Client, expected_number_of_decks: usize) {
    let response = client.get("/tests/decks").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let string_body = response.into_string().expect("Test assertion failed");
    let list_of_decks: Vec<DeckJson> =
        serde_json::from_str(string_body.as_str()).expect("Test assertion failed");
    assert_eq!(expected_number_of_decks, list_of_decks.len());
}

fn get_card(client: &Client, location_header: String) -> usize {
    let response = client.get(location_header).dispatch();
    assert_eq!(response.status(), Status::Ok);
    let string_body = response.into_string().expect("Test assertion failed");
    let created_card: Card =
        serde_json::from_str(string_body.as_str()).expect("Test assertion failed");
    created_card.id
}

fn get_deck(client: &Client, location_header: String) -> DeckJson {
    let response = client.get(location_header).dispatch();
    assert_eq!(response.status(), Status::Ok);
    let string_body = response.into_string().expect("Test assertion failed");
    let created_deck: DeckJson =
        serde_json::from_str(string_body.as_str()).expect("Test assertion failed");
    created_deck
}

fn delete_card_in_deck(client: &Client, location_header: String) {
    let response = client.delete(location_header).dispatch();
    assert_eq!(response.status(), Status::Ok);
}

fn get_card_in_deck(client: &Client, location_header: String) -> DeckCardJson {
    let response = client.get(location_header).dispatch();
    assert_eq!(response.status(), Status::Ok);
    let string_body = response.into_string().expect("Test assertion failed");
    let created_deck: DeckCardJson =
        serde_json::from_str(string_body.as_str()).expect("Test assertion failed");
    created_deck
}

fn post_card(client: &Client, new_card: &serde_json::Value) -> String {
    let body_json = serde_json::to_string(&new_card).expect("Test assertion failed");
    let response = client
        .post("/tests/cards")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(body_json)
        .dispatch();
    assert_eq!(response.status(), Status::Created);
    let response_headers = response.headers();
    let location_header_list: Vec<_> = response_headers.get("location").collect();
    assert_eq!(1, location_header_list.len());
    let location_header = location_header_list.first().expect("Test assertion failed");
    (*location_header).to_string()
}

fn post_card_to_deck(client: &Client, id: usize, new_card: &DeckCardPayload) -> String {
    let body_json = serde_json::to_string(&new_card).expect("Test assertion failed");
    let response = client
        .post(format!("/tests/decks/{}/cards", id))
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(body_json)
        .dispatch();
    assert_eq!(Status::Created, response.status());
    let response_headers = response.headers();
    let location_header_list: Vec<_> = response_headers.get("location").collect();
    assert_eq!(1, location_header_list.len());
    let location_header = location_header_list.first().expect("Test assertion failed");
    (*location_header).to_string()
}

fn post_card_to_deck_fail_on_type(
    client: &Client,
    id: usize,
    new_card: &DeckCardPayload,
    expected_error_message: &str,
) {
    let body_json = serde_json::to_string(&new_card).expect("Test assertion failed");
    let response = client
        .post(format!("/tests/decks/{}/cards", id))
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(body_json)
        .dispatch();
    assert_eq!(Status::BadRequest, response.status());
    let body_json = response.into_string().expect("Test assertion failed");
    let body: MyStatus = serde_json::from_str(body_json.as_str()).expect("Test assertion failed");
    let expected_status: MyStatus = new_status(expected_error_message.to_string()).0;
    assert_eq!(expected_status, body);
}

fn post_deck(client: &Client) -> String {
    let create_deck = CreateDeck {
        contains_card_types: vec![CardType::Attack],
    };
    let create_deck_json = serde_json::to_string(&create_deck).expect("Test assertion failed");
    let response = client
        .post("/tests/decks")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(create_deck_json)
        .dispatch();
    assert_eq!(Status::Created, response.status());
    let response_headers = response.headers();
    let location_header_list: Vec<_> = response_headers.get("location").collect();
    assert_eq!(1, location_header_list.len());
    let location_header = location_header_list.first().expect("Test assertion failed");
    (*location_header).to_string()
}

fn get_cards(client: &Client) -> Vec<Card> {
    let response = client.get("/cards").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let string_body = response.into_string().expect("Test assertion failed");
    let list_of_decks: Vec<Card> =
        serde_json::from_str(string_body.as_str()).expect("Test assertion failed");
    list_of_decks
}

fn get_combat(client: &Client) -> Option<Combat> {
    let response = client.get("/combat").dispatch();
    if response.status().code == 404 {
        None
    } else if response.status().code == 200 {
        let string_body = response.into_string().expect("Test assertion failed");
        let combat: Combat =
            serde_json::from_str(string_body.as_str()).expect("Test assertion failed");
        Some(combat)
    } else {
        panic!("Unexpected status: {}", response.status());
    }
}

fn initialize_combat(client: &Client) {
    let response = client.post("/combat").dispatch();
    assert_eq!(response.status(), Status::Created);
    let response_headers = response.headers();
    let location_header_list: Vec<_> = response_headers.get("location").collect();
    assert_eq!(1, location_header_list.len());
    let location_header = location_header_list.first().expect("Test assertion failed");
    assert_eq!("/combat", (*location_header).to_string());
}

#[allow(dead_code)]
fn action_play_cards(client: &Client, id: usize, expected_response: &str) {
    let action = PlayCard(id);
    let body_json = serde_json::to_string(&action).expect("Test assertion failed");
    let response = client
        .post("/action")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(body_json)
        .dispatch();
    assert_eq!(response.status(), Status::Created);
    let response_headers = response.headers();
    let location_header_list: Vec<_> = response_headers.get("location").collect();
    assert_eq!(1, location_header_list.len());
    let location_header = location_header_list.first().expect("Test assertion failed");
    assert_eq!(expected_response, (*location_header).to_string());
}

#[allow(dead_code)]
fn action_play_cards_not_found(client: &Client, id: usize, expected_error_message: &str) {
    let action = PlayCard(id);
    let body_json = serde_json::to_string(&action).expect("Test assertion failed");
    let response = client
        .post("/action")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(body_json)
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);
    let body_json = response.into_string().expect("Test assertion failed");
    let body: MyStatus = serde_json::from_str(body_json.as_str()).expect("Test assertion failed");
    let expected_status: MyStatus = new_status(expected_error_message.to_string()).0;
    assert_eq!(expected_status, body);
}
