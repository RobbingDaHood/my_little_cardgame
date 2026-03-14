//! Integration tests for multi-instance server support (port configuration).
//!
//! Verifies that the Rocket server respects ROCKET_PORT and other
//! figment-based configuration, enabling multiple instances on the same machine.

use my_little_cardgame::rocket_initialize;
use rocket::local::blocking::Client;

#[test]
fn server_responds_with_default_configuration() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let response = client.get("/docs/tutorial").dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
}

#[test]
fn server_accepts_figment_port_override() {
    use rocket::figment::{providers::Serialized, Figment};

    let figment = Figment::from(rocket::Config::default()).merge(Serialized::default("port", 9999));

    let rocket = rocket_initialize().configure(figment);
    let config = rocket
        .figment()
        .extract::<rocket::Config>()
        .expect("valid config");
    assert_eq!(config.port, 9999);

    let client = Client::tracked(rocket).expect("valid rocket instance with custom port");
    let response = client.get("/docs/tutorial").dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
}
