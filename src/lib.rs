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
pub mod area_deck;
pub mod combat;
pub mod library;
pub mod player_data;
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
    use crate::area_deck::endpoints::okapi_add_operation_for_get_area_;
    use crate::area_deck::endpoints::okapi_add_operation_for_get_area_encounters_;
    use crate::area_deck::endpoints::{get_area, get_area_encounters};
    use crate::combat::okapi_add_operation_for_get_combat_;
    use crate::combat::okapi_add_operation_for_get_combat_result_;
    use crate::combat::okapi_add_operation_for_initialize_combat_;
    use crate::combat::okapi_add_operation_for_simulate_combat_endpoint_;
    use crate::combat::{
        get_combat, get_combat_result, initialize_combat, simulate_combat_endpoint,
    };
    use crate::library::add_test_library_card;
    use crate::library::list_library_cards;
    use crate::library::list_library_tokens;
    use crate::library::okapi_add_operation_for_list_library_tokens_;
    use crate::player_tokens::get_player_tokens;
    use crate::player_tokens::okapi_add_operation_for_get_player_tokens_;

    #[allow(clippy::no_effect_underscore_binding)]
    let _ = env_logger::try_init();

    use rocket::fairing::AdHoc;

    let gs = std::sync::Arc::new(rocket::futures::lock::Mutex::new(library::GameState::new()));

    let rocket = rocket::build()
        .mount(
            "/",
            openapi_get_routes![
                get_combat,
                initialize_combat,
                simulate_combat_endpoint,
                play,
                get_player_tokens,
                get_combat_result,
                list_library_tokens,
                list_actions_log,
                get_area,
                get_area_encounters
            ],
        )
        .mount("/swagger", make_swagger_ui(&get_docs()))
        .mount(
            "/",
            rocket::routes![
                list_library_cards,
                add_test_library_card,
                crate::combat::enemy_play,
                crate::combat::advance_phase,
            ],
        )
        .manage(player_data::new())
        .manage(gs.clone())
        .attach(AdHoc::on_liftoff("actionlog-shutdown", |rocket| {
            Box::pin(async move {
                // When the process receives SIGINT/SIGTERM (or ctrl-c), flush the action log writer
                if let Some(gs_state) = rocket
                    .state::<std::sync::Arc<rocket::futures::lock::Mutex<library::GameState>>>()
                    .cloned()
                {
                    rocket::tokio::spawn(async move {
                        #[cfg(unix)]
                        {
                            use rocket::tokio::signal::unix::{signal, SignalKind};
                            let mut sigterm = signal(SignalKind::terminate())
                                .expect("failed to set SIGTERM handler");
                            let mut sigint = signal(SignalKind::interrupt())
                                .expect("failed to set SIGINT handler");
                            rocket::tokio::select! {
                                _ = sigterm.recv() => {},
                                _ = sigint.recv() => {},
                            }
                        }
                        #[cfg(not(unix))]
                        {
                            let _ = rocket::tokio::signal::ctrl_c().await;
                        }

                        // call shutdown helper to flush file writer
                        let gs = gs_state.lock().await;
                        gs.shutdown();
                    });
                }
            })
        }));

    rocket
}

fn get_docs() -> SwaggerUIConfig {
    SwaggerUIConfig {
        url: "/openapi.json".to_string(),
        ..Default::default()
    }
}
