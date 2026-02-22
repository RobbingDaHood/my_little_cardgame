use my_little_cardgame::library::types::CombatSnapshot;
use rocket::http::Status;
use rocket::local::blocking::Client;
use rocket::serde::json::serde_json;
use serde::Deserialize;

use my_little_cardgame::rocket_initialize;

#[derive(Debug, Deserialize)]
struct LibraryCardJson {
    kind: serde_json::Value,
    counts: CountsJson,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct CountsJson {
    library: u32,
    deck: u32,
    hand: u32,
    discard: u32,
}

#[test]
fn hello_world() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Verify Library cards are initialized (Attack, Defence, Resource, CombatEncounter)
    let library_cards = get_library_cards(&client);
    assert_eq!(4, library_cards.len());

    // Verify card counts: each player card has deck:35, hand:5
    for card in &library_cards[0..3] {
        assert_eq!(card.counts.deck, 35);
        assert_eq!(card.counts.hand, 5);
        assert_eq!(card.counts.discard, 0);
    }

    // Verify card kinds
    assert_eq!(library_cards[0].kind["kind"], "Attack");
    assert_eq!(library_cards[1].kind["kind"], "Defence");
    assert_eq!(library_cards[2].kind["kind"], "Resource");
    assert_eq!(library_cards[3].kind["kind"], "CombatEncounter");

    // Verify combat not initialized yet
    assert_eq!(get_combat(&client), None);

    // Initialize combat
    initialize_combat(&client);

    let actual = get_combat(&client);
    assert!(actual.is_some());
    let actual_combat = actual.expect("combat exists");
    assert_eq!(
        actual_combat.phase,
        my_little_cardgame::library::types::CombatPhase::Defending
    );
    assert_eq!(actual_combat.round, 1);
    assert!(actual_combat.player_turn);
    // Enemy should have initial tokens (health=20, max_health=20 from gnome combatant_def)
    assert_eq!(
        actual_combat
            .enemy
            .active_tokens
            .get("health")
            .copied()
            .unwrap_or(0),
        20
    );
    // Player should have health token from token_balances (initialized to 20)
    assert_eq!(
        actual_combat
            .player_tokens
            .get("health")
            .copied()
            .unwrap_or(0),
        20
    );
}

fn get_library_cards(client: &Client) -> Vec<LibraryCardJson> {
    let response = client.get("/library/cards").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let string_body = response.into_string().expect("Test assertion failed");
    serde_json::from_str(string_body.as_str()).expect("Test assertion failed")
}

fn get_combat(client: &Client) -> Option<CombatSnapshot> {
    let response = client.get("/combat").dispatch();
    if response.status().code == 404 {
        None
    } else if response.status().code == 200 {
        let string_body = response.into_string().expect("Test assertion failed");
        let combat: CombatSnapshot =
            serde_json::from_str(string_body.as_str()).expect("Test assertion failed");
        Some(combat)
    } else {
        panic!("Unexpected status: {}", response.status());
    }
}

fn initialize_combat(client: &Client) {
    let response = client.post("/tests/combat").dispatch();
    assert_eq!(response.status(), Status::Created);
    let response_headers = response.headers();
    let location_header_list: Vec<_> = response_headers.get("location").collect();
    assert_eq!(1, location_header_list.len());
    let location_header = location_header_list.first().expect("Test assertion failed");
    assert_eq!("/tests/combat", (*location_header).to_string());
}
