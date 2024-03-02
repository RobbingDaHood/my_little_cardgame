#[macro_use]
extern crate rocket;

use crate::deck::{list_all_decks, new_card};
use crate::deck::card_state::CardState::Deck;
use crate::deck::new as new_deck;
use crate::player_data::new as new_player;

mod deck;
mod player_data;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/decks", routes![list_all_decks])
        .manage(new_player(vec![
            new_deck(vec![
                new_card(22, Deck, 1)
            ])
        ]))
}

