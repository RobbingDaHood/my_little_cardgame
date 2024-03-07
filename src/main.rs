#[macro_use]
extern crate rocket;

use std::sync::Arc;

use rocket::futures::lock::Mutex;
use rocket_okapi::openapi_get_routes;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};

use crate::deck::{add_card_to_deck, create_deck, delete_card_in_deck, get_card_in_deck, get_deck, list_all_decks};
use crate::deck::card::{create_card, get_card_json, list_all_cards};
use crate::deck::card::okapi_add_operation_for_create_card_;
use crate::deck::card::okapi_add_operation_for_get_card_json_;
use crate::deck::card::okapi_add_operation_for_list_all_cards_;
use crate::deck::okapi_add_operation_for_add_card_to_deck_;
use crate::deck::okapi_add_operation_for_create_deck_;
use crate::deck::okapi_add_operation_for_get_card_in_deck_;
use crate::deck::okapi_add_operation_for_get_deck_;
use crate::deck::okapi_add_operation_for_list_all_decks_;
use crate::deck::okapi_add_operation_for_delete_card_in_deck_;
use crate::player_data::new as new_player;

mod deck;
mod player_data;
mod status_messages;
mod tests;

#[launch]
fn rocket_initialize() -> _ {
    rocket::build()
        .mount("/", openapi_get_routes![list_all_decks, get_deck, add_card_to_deck, create_deck,
            list_all_cards, get_card_json, create_card, get_card_in_deck, delete_card_in_deck])
        .mount("/swagger", make_swagger_ui(&get_docs()))
        .manage(new_player(
            Arc::new(
                Mutex::new(
                    vec![]
                )
            ),
            Arc::new(
                Mutex::new(
                    vec![],
                )
            ),
        ))
}

fn get_docs() -> SwaggerUIConfig {
    SwaggerUIConfig {
        url: "/openapi.json".to_string(),
        ..Default::default()
    }
}