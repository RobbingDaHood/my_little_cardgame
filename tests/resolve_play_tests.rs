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

    // Discover a Defence card ID dynamically from the library API
    let resp = client
        .get("/library/cards?location=Hand&card_kind=Defence")
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body = resp.into_string().unwrap();
    let cards: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap();
    assert!(!cards.is_empty(), "Should have Defence cards in hand");
    let defence_id = cards[0]["id"].as_u64().unwrap();

    // Play the Defence card which adds shield via CardEffect
    let action_json = format!(
        r#"{{ "action_type": "EncounterPlayCard", "card_id": {} }}"#,
        defence_id
    );
    let play_response = client
        .post("/action")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(&action_json)
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

    // Discover card IDs dynamically from the library API
    let find_hand_card = |kind: &str| -> usize {
        let resp = client
            .get(format!("/library/cards?location=Hand&card_kind={}", kind))
            .dispatch();
        assert_eq!(resp.status(), Status::Ok);
        let body = resp.into_string().unwrap();
        let cards: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap();
        assert!(!cards.is_empty(), "Should have {} cards in hand", kind);
        cards[0]["id"].as_u64().unwrap() as usize
    };
    let defence_id = find_hand_card("Defence");
    let attack_id = find_hand_card("Attack");
    let resource_id = find_hand_card("Resource");
    let phase_cards = [defence_id, attack_id, resource_id];

    // Play cards cycling through phases until combat ends
    for phase_idx in 0..150 {
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
        let combat_resp = client.get("/encounter").dispatch();
        if combat_resp.status() == Status::NotFound {
            break;
        }
    }

    // Verify combat results exist
    let result_resp = client.get("/encounter/results").dispatch();
    assert_eq!(result_resp.status(), Status::Ok);
    let body = result_resp.into_string().unwrap();
    let results: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap();
    assert!(!results.is_empty());
}
