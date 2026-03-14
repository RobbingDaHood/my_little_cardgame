//! Long-scenario integration tests that exercise full gameplay loops.
//!
//! These tests serve as living documentation for how to play the game
//! via the HTTP API. They use only the production endpoints (POST /action
//! and GET routes) — no test-only endpoints.
//!
//! When new use cases or encounter types are added, add or update
//! scenarios here so they remain an accurate gameplay guide.

mod api;
mod combat;
mod costs;
mod crafting;
mod fishing;
mod helpers;
mod herbalism;
mod milestone;
mod mining;
mod research;
mod rest;
mod woodcutting;
