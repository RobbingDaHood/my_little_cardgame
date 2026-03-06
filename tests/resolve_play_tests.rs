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

    // Start a proper game so combat has correct state
    let action_json = r#"{"action_type":"NewGame","seed":42}"#;
    let resp = client
        .post("/action")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(action_json)
        .dispatch();
    assert_eq!(resp.status(), Status::Created);

    // Pick a combat encounter dynamically
    let enc_resp = client
        .get("/library/cards?location=Hand&card_kind=Encounter")
        .dispatch();
    let enc_cards: Vec<serde_json::Value> =
        serde_json::from_str(&enc_resp.into_string().unwrap()).unwrap();
    let combat_card_id = enc_cards
        .iter()
        .find_map(|c| {
            let et = c
                .get("kind")?
                .get("encounter_kind")?
                .get("encounter_type")?
                .as_str()?;
            if et == "Combat" {
                c.get("id")?.as_u64()
            } else {
                None
            }
        })
        .expect("Should have a combat encounter card");
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        combat_card_id
    );
    let resp = client
        .post("/action")
        .header(Header {
            name: Uncased::from("Content-Type"),
            value: Cow::from("application/json"),
        })
        .body(&pick_json)
        .dispatch();
    assert_eq!(resp.status(), Status::Created);

    // Play rounds (rediscovering card IDs each phase) until combat ends
    let play_card = |kind: &str| -> Status {
        let resp = client
            .get(format!("/library/cards?location=Hand&card_kind={}", kind))
            .dispatch();
        if resp.status() != Status::Ok {
            return resp.status();
        }
        let cards: Vec<serde_json::Value> =
            serde_json::from_str(&resp.into_string().unwrap_or_default()).unwrap_or_default();
        if cards.is_empty() {
            return Status::NotFound;
        }
        let card_id = cards[0]["id"].as_u64().unwrap();
        let json = format!(
            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
            card_id
        );
        client
            .post("/action")
            .header(Header {
                name: Uncased::from("Content-Type"),
                value: Cow::from("application/json"),
            })
            .body(&json)
            .dispatch()
            .status()
    };

    for _ in 0..80 {
        for kind in &["Defence", "Attack", "Resource"] {
            let status = play_card(kind);
            if status != Status::Created {
                break;
            }
            let combat_resp = client.get("/encounter").dispatch();
            if combat_resp.status() == Status::NotFound {
                break;
            }
            let combat: serde_json::Value =
                serde_json::from_str(&combat_resp.into_string().unwrap_or_default())
                    .unwrap_or_default();
            if combat.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                break;
            }
        }
        let combat_resp = client.get("/encounter").dispatch();
        if combat_resp.status() == Status::NotFound {
            break;
        }
        let combat: serde_json::Value =
            serde_json::from_str(&combat_resp.into_string().unwrap_or_default())
                .unwrap_or_default();
        if combat.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
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
