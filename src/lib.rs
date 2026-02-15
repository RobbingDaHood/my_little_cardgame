//! # My Little Card Game
//!
//! A web-based card game API where everything is represented as decks.
//!
//! ## Overview
//!
//! This library provides a RESTful API for managing cards, decks, and combat
//! in a card game system. The game features three types of cards (Attack,
//! Defence, Resource) that can be organized into decks and used in combat.
//!
//! ## Architecture
//!
//! The API is built using the Rocket web framework with OpenAPI documentation
//! support. Game state is managed through thread-safe `Arc<Mutex<T>>` wrappers
//! to allow concurrent access from multiple HTTP requests.

// Rocket makes this a bit tricky to support
#![allow(clippy::module_name_repetitions)]
#[macro_use]
extern crate rocket;

use rocket_okapi::openapi_get_routes;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};

pub mod action;
pub mod actions_log;
pub mod combat;
pub mod deck;
pub mod library;
pub mod player_data;
pub mod player_seed;
pub mod player_tokens;
pub mod status_messages;

// Re-export for tests
pub use crate::player_data::new as player_data_new;

/// Initializes and configures the Rocket web server with all routes and OpenAPI documentation.
///
/// # Returns
///
/// A configured Rocket instance ready to be launched.
///
/// # Example
///
/// ```no_run
/// use my_little_cardgame::rocket_initialize;
///
/// #[rocket::main]
/// async fn main() {
///     rocket_initialize().launch().await.expect("Failed to launch rocket");
/// }
/// ```
pub fn rocket_initialize() -> rocket::Rocket<rocket::Build> {
    use crate::action::okapi_add_operation_for_play_;
    use crate::action::play;
    use crate::actions_log::list_actions_log;
    use crate::actions_log::okapi_add_operation_for_list_actions_log_;
    use crate::combat::okapi_add_operation_for_advance_phase_;
    use crate::combat::okapi_add_operation_for_enemy_play_;
    use crate::combat::okapi_add_operation_for_get_combat_;
    use crate::combat::okapi_add_operation_for_get_combat_result_;
    use crate::combat::okapi_add_operation_for_initialize_combat_;
    use crate::combat::{
        advance_phase, enemy_play, get_combat, get_combat_result, initialize_combat,
    };
    use crate::deck::card::okapi_add_operation_for_create_card_;
    use crate::deck::card::okapi_add_operation_for_get_card_json_;
    use crate::deck::card::okapi_add_operation_for_list_all_cards_;
    use crate::deck::card::{create_card, get_card_json, list_all_cards};
    use crate::deck::okapi_add_operation_for_add_card_to_deck_;
    use crate::deck::okapi_add_operation_for_create_deck_;
    use crate::deck::okapi_add_operation_for_delete_card_in_deck_;
    use crate::deck::okapi_add_operation_for_get_card_in_deck_;
    use crate::deck::okapi_add_operation_for_get_deck_;
    use crate::deck::okapi_add_operation_for_list_all_decks_;
    use crate::deck::{
        add_card_to_deck, create_deck, delete_card_in_deck, get_card_in_deck, get_deck,
        list_all_decks,
    };
    use crate::library::list_library_cards;
    use crate::library::list_library_tokens;
    use crate::library::okapi_add_operation_for_list_library_tokens_;
    use crate::player_seed::okapi_add_operation_for_set_seed_;
    use crate::player_seed::set_seed;
    use crate::player_tokens::get_player_tokens;
    use crate::player_tokens::okapi_add_operation_for_get_player_tokens_;

    #[allow(clippy::no_effect_underscore_binding)]
    let _ = env_logger::try_init();
    rocket::build()
        .mount(
            "/",
            openapi_get_routes![
                list_all_decks,
                get_deck,
                add_card_to_deck,
                create_deck,
                list_all_cards,
                get_card_json,
                create_card,
                get_card_in_deck,
                delete_card_in_deck,
                get_combat,
                initialize_combat,
                enemy_play,
                advance_phase,
                play,
                get_player_tokens,
                set_seed,
                get_combat_result,
                list_library_tokens,
                list_actions_log
            ],
        )
        .mount("/swagger", make_swagger_ui(&get_docs()))
        .mount("/", rocket::routes![list_library_cards])
        .manage(player_data::new())
        .manage(std::sync::Arc::new(rocket::futures::lock::Mutex::new(
            library::GameState::new(),
        )))
}

fn get_docs() -> SwaggerUIConfig {
    SwaggerUIConfig {
        url: "/openapi.json".to_string(),
        ..Default::default()
    }
}
