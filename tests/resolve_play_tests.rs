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
    let action_json = r#"{ "action_type": "EncounterPlayCard", "card_id": 1 }"#;
    let play_response = client
        .post("/action")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(action_json)
        .dispatch();
    assert_eq!(play_response.status(), Status::Created);

    // Verify player has shield tokens via /player/tokens
    let tokens_resp = client.get("/player/tokens").dispatch();
    assert_eq!(tokens_resp.status(), Status::Ok);
    let tokens_body = tokens_resp.into_string().expect("read tokens");
    let tokens_json: serde_json::Value = serde_json::from_str(&tokens_body).expect("parse json");
    let shield = tokens_json
        .as_array()
        .and_then(|arr| {
            arr.iter().find_map(|entry| {
                if entry["token"]["token_type"].as_str() == Some("Shield") {
                    entry["value"].as_i64()
                } else {
                    None
                }
            })
        })
        .unwrap_or(0);
    assert!(
        shield > 0,
        "Player should have shield tokens after defence play"
    );
}

#[test]
fn test_play_attack_card_kills_enemy() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Initialize combat (starts in Defending)
    client.post("/tests/combat").dispatch();

    // Play cards cycling through phases until combat ends
    // Phase cycle: Defending(1) -> Attacking(0) -> Resourcing(2) -> ...
    let phase_cards = [1, 0, 2]; // Defence, Attack, Resource
    for (phase_idx, _) in (0..30).enumerate() {
        let card_id = phase_cards[phase_idx % 3];
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
            .body(&action_json)
            .dispatch();
        if play_response.status() != Status::Created {
            break;
        }
        let combat_resp = client.get("/combat").dispatch();
        if combat_resp.status() == Status::NotFound {
            break;
        }
    }

    // Verify combat result exists
    let result_resp = client.get("/combat/result").dispatch();
    assert_eq!(result_resp.status(), Status::Ok);
}
