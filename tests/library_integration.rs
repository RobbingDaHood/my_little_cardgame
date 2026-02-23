use my_little_cardgame::library::types::TokenType;
use rocket::http::Status;
use rocket::local::blocking::Client;

#[test]
fn library_tokens_endpoint_returns_list() {
    let client =
        Client::tracked(my_little_cardgame::rocket_initialize()).expect("valid rocket instance");
    let response = client.get("/tokens").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().expect("response body");
    let tokens: Vec<TokenType> = serde_json::from_str(&body).expect("valid json");
    assert!(tokens.contains(&TokenType::Insight));
    assert!(tokens.contains(&TokenType::Renown));
}
