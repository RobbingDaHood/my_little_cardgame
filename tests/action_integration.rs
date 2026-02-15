use my_little_cardgame::library::types::{ActionEntry, ActionPayload};
use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;
use serde_json::json;

#[test]
fn grant_token_via_action() {
    let client =
        Client::tracked(my_little_cardgame::rocket_initialize()).expect("valid rocket instance");
    let body = json!({"GrantToken": {"token_id":"Insight","amount":10}}).to_string();
    let response = client
        .post("/action")
        .header(ContentType::JSON)
        .body(body)
        .dispatch();
    assert_eq!(response.status(), Status::Created);
    let response = client.get("/actions/log").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().expect("response body");
    let entries: Vec<ActionEntry> = serde_json::from_str(&body).expect("valid json");
    let found = entries.iter().any(|e| match &e.payload {
        ActionPayload::GrantToken { token_id, amount } => token_id == "Insight" && *amount == 10,
        _ => false,
    });
    assert!(found);
}

#[test]
fn set_seed_via_action_records_log() {
    let client =
        Client::tracked(my_little_cardgame::rocket_initialize()).expect("valid rocket instance");
    let body = json!({"SetSeed": {"seed": 42u64}}).to_string();
    let response = client
        .post("/action")
        .header(ContentType::JSON)
        .body(body)
        .dispatch();
    assert_eq!(response.status(), Status::Created);
    let response = client.get("/actions/log").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().expect("response body");
    let entries: Vec<ActionEntry> = serde_json::from_str(&body).expect("valid json");
    let found = entries.iter().any(|e| match &e.payload {
        ActionPayload::SetSeed { seed } => *seed == 42u64,
        _ => false,
    });
    assert!(found);
}
