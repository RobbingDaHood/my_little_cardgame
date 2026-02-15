use rocket::http::Status;
use rocket::local::blocking::Client;
use rocket::serde::json::Json;

#[test]
fn library_tokens_endpoint_returns_list() {
    let client =
        Client::tracked(my_little_cardgame::rocket_initialize()).expect("valid rocket instance");
    let response = client.get("/library/tokens").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().expect("response body");
    // Parse JSON and assert canonical tokens are present
    let tokens: Vec<String> = serde_json::from_str(&body).expect("valid json");
    assert!(tokens.contains(&"Insight".to_string()));
    assert!(tokens.contains(&"Renown".to_string()));
}
