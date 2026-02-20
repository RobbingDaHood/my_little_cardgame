use my_little_cardgame::rocket_initialize;
use rocket::http::uncased::Uncased;
use rocket::http::{Header, Status};
use rocket::local::blocking::Client;
use std::borrow::Cow;

// Interleaved player and enemy actions exercised sequentially to validate lock ordering and lack of panics.
#[test]
fn interleaved_player_and_enemy_actions() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Create a Defence card that grants a Dodge token so it can be played in Defending phase
    let card_json = r#"{
        "card_type_id": 1,
        "card_type": "Defence",
        "effects": [{"token_type":"Dodge","permanence":"UsedOnUnit","count":1}],
        "costs": [],
        "count": 1
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
    let card_location = response
        .headers()
        .get_one("location")
        .expect("Missing location header");
    let card_id: usize = card_location
        .trim_start_matches("/cards/")
        .parse()
        .expect("Invalid card ID");

    // Add the card to the default defence deck with several Hand copies so repeated plays succeed
    let deck_card_json = format!(r#"{{ "id": {}, "state": {{ "Hand": 20 }} }}"#, card_id);
    let add_response = client
        .post("/decks/1/cards")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(deck_card_json)
        .dispatch();
    assert_eq!(add_response.status(), Status::Created);

    // Initialize combat (starts in Defending)
    let init_response = client.post("/combat").dispatch();
    assert_eq!(init_response.status(), Status::Created);

    // Interleave plays and enemy plays sequentially to exercise locking
    for _ in 0..50 {
        let action_json = format!(r#"{{ "PlayCard": {} }}"#, card_id);
        let play_response = client
            .post("/action")
            .header(Header {
                name: Uncased::from("Content-Type"),
                value: Cow::from("application/json"),
            })
            .body(action_json)
            .dispatch();
        // allow BadRequest/NotFound if a play is invalid or already consumed; ensure server did not panic
        let play_status = play_response.status();
        if play_status != Status::Created
            && play_status != Status::BadRequest
            && play_status != Status::NotFound
        {
            let body = play_response.into_string().unwrap_or_default();
            panic!(
                "POST /action returned unexpected status {:?} body={}",
                play_status, body
            );
        }

        let enemy_response = client.post("/combat/enemy_play").dispatch();
        assert_eq!(enemy_response.status(), Status::Created);
    }
}
