use rocket::local::blocking::Client;
use rocket::http::Status;
use rocket::serde::json::Json;

#[test]
fn library_tokens_endpoint_returns_list() {
    let client = Client::tracked(my_little_cardgame::rocket_initialize()).expect("valid rocket instance");
    let response = client.get("/library/tokens").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().expect("response body");
    // Expect at least the canonical tokens to be present
    assert!(body.contains("Insight"));
    assert!(body.contains("Renown"));
}
