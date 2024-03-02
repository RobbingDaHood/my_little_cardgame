#[macro_use]
extern crate rocket;

use std::sync::Arc;

use rocket::futures::lock::Mutex;

use crate::deck::{add_card_to_deck, get_deck, list_all_decks, new_card};
use crate::deck::card::{create_card, get_card, list_all_cards};
use crate::deck::card_state::CardState::Deck;
use crate::deck::new as new_deck;
use crate::player_data::new as new_player;

mod deck;
mod player_data;
mod status_messages;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/decks", routes![list_all_decks, get_deck, add_card_to_deck])
        .mount("/cards", routes![list_all_cards, get_card, create_card])
        .manage(new_player(
            vec![
                Arc::new(
                    Mutex::new(
                        new_deck(vec![
                            new_card(22, Deck, 1)
                        ])
                    )
                )
            ],
            Arc::new(
                Mutex::new(
                    vec![],
                )
            ),
        ))
}

