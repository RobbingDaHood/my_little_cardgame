#[macro_use]
extern crate rocket;

use std::sync::Arc;

use rocket::futures::lock::Mutex;

use crate::deck::{add_card_to_deck, create_deck, get_deck_json, list_all_decks};
use crate::deck::card::{create_card, get_card_json, list_all_cards};
use crate::player_data::new as new_player;

mod deck;
mod player_data;
mod status_messages;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/decks", routes![list_all_decks, get_deck_json, add_card_to_deck, create_deck])
        .mount("/cards", routes![list_all_cards, get_card_json, create_card])
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

