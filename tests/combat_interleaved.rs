use my_little_cardgame::rocket_initialize;
use rocket::http::uncased::Uncased;
use rocket::http::{Header, Status};
use rocket::local::blocking::Client;
use std::borrow::Cow;

// Interleaved player and enemy actions exercised sequentially to validate lock ordering and lack of panics.
#[test]
fn interleaved_player_and_enemy_actions() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Initialize combat (starts in Defending)
    let init_response = client.post("/tests/combat").dispatch();
    assert_eq!(init_response.status(), Status::Created);

    // Use existing Defence card (Library ID 1, starts with 5 hand copies)
    let card_id = 1;

    // Interleave plays and enemy plays sequentially to exercise locking
    for _ in 0..5 {
        let action_json = format!(
            r#"{{ "action_type": "EncounterPlayCard", "card_id": {} }}"#,
            card_id
        );
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

        let enemy_response = client.post("/tests/combat/enemy_play").dispatch();
        assert_eq!(enemy_response.status(), Status::Created);
    }
}
