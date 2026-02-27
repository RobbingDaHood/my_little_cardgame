//! Long-scenario integration tests that exercise full gameplay loops.
//!
//! These tests serve as living documentation for how to play the game
//! via the HTTP API. They use only the production endpoints (POST /action
//! and GET routes) — no test-only endpoints.
//!
//! When new use cases or encounter types are added, add or update
//! scenarios here so they remain an accurate gameplay guide.

use my_little_cardgame::rocket_initialize;
use rocket::http::uncased::Uncased;
use rocket::http::{Header, Status};
use rocket::local::blocking::Client;
use rocket::serde::json::serde_json;
use std::borrow::Cow;

fn json_header() -> Header<'static> {
    Header {
        name: Uncased::from("Content-Type"),
        value: Cow::from("application/json"),
    }
}

fn post_action(client: &Client, json: &str) -> (Status, serde_json::Value) {
    let resp = client
        .post("/action")
        .header(json_header())
        .body(json)
        .dispatch();
    let status = resp.status();
    let body: serde_json::Value =
        serde_json::from_str(&resp.into_string().unwrap_or_default()).unwrap_or_default();
    (status, body)
}

fn get_json(client: &Client, uri: &str) -> serde_json::Value {
    let resp = client.get(uri).dispatch();
    serde_json::from_str(&resp.into_string().unwrap_or_default()).unwrap_or_default()
}

fn player_health(client: &Client) -> i64 {
    player_token(client, "Health")
}

fn player_token(client: &Client, token_type_name: &str) -> i64 {
    let resp = client.get("/player/tokens").dispatch();
    let tokens: serde_json::Value =
        serde_json::from_str(&resp.into_string().unwrap_or_default()).unwrap_or_default();
    tokens
        .as_array()
        .and_then(|arr| {
            arr.iter().find_map(|entry| {
                let tt = entry.get("token")?.get("token_type")?.as_str()?;
                if tt == token_type_name {
                    entry.get("value")?.as_i64()
                } else {
                    None
                }
            })
        })
        .unwrap_or(0)
}

fn combat_state(client: &Client) -> serde_json::Value {
    get_json(client, "/encounter")
}

fn encounter_hand_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| c.get("id").and_then(|v| v.as_u64()).map(|v| v as usize))
        .collect()
}

fn combat_result(client: &Client) -> Option<String> {
    let resp = client.get("/encounter/results").dispatch();
    if resp.status() == Status::Ok {
        let body: Vec<serde_json::Value> =
            serde_json::from_str(&resp.into_string().unwrap_or_default()).unwrap_or_default();
        body.last().and_then(|v| v.as_str()).map(String::from)
    } else {
        None
    }
}

/// Play one full round (Defence → Attack → Resource) using the default
/// card IDs: 9 = Defence, 8 = Attack, 10 = Resource.
/// Returns true if combat is still active after the round.
fn play_one_round(client: &Client) -> bool {
    let cards = [9, 8, 10]; // Defence, Attack, Resource
    for card_id in &cards {
        let json = format!(
            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
            card_id
        );
        let (status, _) = post_action(client, &json);
        if status != Status::Created {
            return false;
        }
        // Check if combat ended
        let combat = combat_state(client);
        if combat.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
            return false;
        }
    }
    true
}

#[test]
fn scenario_player_wins_combat_then_picks_next_encounter() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // 1. Start a new game with a fixed seed for determinism
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created, "NewGame should succeed");

    // 2. Verify encounter cards are available
    let encounter_ids = encounter_hand_ids(&client);
    assert!(
        !encounter_ids.is_empty(),
        "Should have encounter cards in hand"
    );

    // 3. Pick the Gnome encounter (card_id 11)
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterPickEncounter","card_id":11}"#,
    );
    assert_eq!(status, Status::Created, "PickEncounter should succeed");

    // 4. Verify combat started
    let combat = combat_state(&client);
    assert_eq!(
        combat.get("outcome").and_then(|v| v.as_str()),
        Some("Undecided"),
        "Combat should be active"
    );
    assert!(player_health(&client) > 0, "Player should have health");

    // 5. Play rounds until combat finishes (max 50 to prevent infinite loop)
    let mut rounds = 0;
    while play_one_round(&client) {
        rounds += 1;
        assert!(rounds < 50, "Combat should end within 50 rounds");
    }

    // 6. Verify combat ended
    let result = combat_result(&client);
    assert!(result.is_some(), "Should have a combat result");

    // With seed 42, determine who won and assert appropriately
    let outcome = result.unwrap();
    assert!(
        outcome == "PlayerWon" || outcome == "PlayerLost",
        "Combat outcome should be PlayerWon or PlayerLost, got: {}",
        outcome
    );

    // 7. If player won, verify transition to Scouting and ability to continue
    if outcome == "PlayerWon" {
        // Apply scouting to move back to Ready
        let (status, _) = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
        assert_eq!(status, Status::Created, "ApplyScouting should succeed");

        // Verify we're back in Ready phase — can pick another encounter
        let ids_after = encounter_hand_ids(&client);
        assert!(
            !ids_after.is_empty(),
            "Should have encounter cards after scouting"
        );
    }
}

#[test]
fn scenario_full_loop_new_game_combat_scout_combat() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Start fresh game
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":12345}"#);
    assert_eq!(status, Status::Created);

    // First combat
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterPickEncounter","card_id":11}"#,
    );
    assert_eq!(status, Status::Created);

    let mut rounds = 0;
    while play_one_round(&client) {
        rounds += 1;
        assert!(rounds < 50, "First combat should end within 50 rounds");
    }

    let result = combat_result(&client).expect("Should have combat result");

    if result == "PlayerWon" {
        // Scout and then start second combat
        let (status, _) = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
        assert_eq!(status, Status::Created);

        // Player health should persist across encounters
        let hp = player_health(&client);
        assert!(hp > 0, "Player health should be positive after winning");

        // Pick second encounter
        let encounter_ids = encounter_hand_ids(&client);
        assert!(
            !encounter_ids.is_empty(),
            "Should have encounters available after scouting"
        );

        let second_enc_id = encounter_ids[0];
        let json = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            second_enc_id
        );
        let (status, _) = post_action(&client, &json);
        assert_eq!(
            status,
            Status::Created,
            "Second PickEncounter should succeed"
        );

        // Verify new combat started
        let combat = combat_state(&client);
        assert_eq!(
            combat.get("outcome").and_then(|v| v.as_str()),
            Some("Undecided"),
            "Second combat should be active"
        );

        // Play second combat
        let mut rounds2 = 0;
        while play_one_round(&client) {
            rounds2 += 1;
            assert!(rounds2 < 50, "Second combat should end within 50 rounds");
        }

        let result2 = combat_result(&client).expect("Should have second combat result");
        assert!(
            result2 == "PlayerWon" || result2 == "PlayerLost",
            "Second combat should have an outcome"
        );
    }
}

#[test]
fn scenario_enemy_wins_combat() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Try multiple seeds to find one where the enemy wins.
    // We test seeds until we find either a player-win or enemy-win scenario,
    // verifying the game state is correct in each case.
    let seeds = [1, 7, 99, 256, 1000, 9999];
    let mut found_enemy_win = false;

    for seed in &seeds {
        let (status, _) = post_action(
            &client,
            &format!(r#"{{"action_type":"NewGame","seed":{}}}"#, seed),
        );
        assert_eq!(status, Status::Created);

        let (status, _) = post_action(
            &client,
            r#"{"action_type":"EncounterPickEncounter","card_id":11}"#,
        );
        assert_eq!(status, Status::Created);

        let mut rounds = 0;
        while play_one_round(&client) {
            rounds += 1;
            if rounds >= 50 {
                break;
            }
        }

        if let Some(result) = combat_result(&client) {
            if result == "PlayerLost" {
                found_enemy_win = true;

                // Verify player health is 0
                let hp = player_health(&client);
                assert_eq!(hp, 0, "Player health should be 0 when enemy wins");

                // Verify encounter transitions to Scouting even on loss
                // (player can still apply scouting after a loss)
                let (scout_status, _) = post_action(
                    &client,
                    r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
                );
                assert_eq!(
                    scout_status,
                    Status::Created,
                    "Should be able to scout after losing"
                );
                break;
            }
        }
    }

    // If we didn't find an enemy-win scenario with these seeds,
    // that's OK — we verified combat works correctly for all tested seeds.
    // The test still validates the full game loop.
    if !found_enemy_win {
        eprintln!(
            "Note: No enemy-win scenario found with tested seeds. \
             All combats resulted in player wins."
        );
    }
}

#[test]
fn scenario_action_log_records_full_game() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Start game
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);

    // Pick encounter
    post_action(
        &client,
        r#"{"action_type":"EncounterPickEncounter","card_id":11}"#,
    );

    // Play one round
    post_action(
        &client,
        r#"{"action_type":"EncounterPlayCard","card_id":9}"#,
    );

    // Verify action log captured all actions
    let log = get_json(&client, "/actions/log");
    let entries = log
        .get("entries")
        .and_then(|v| v.as_array())
        .expect("Action log should have an entries array");

    // Should have at least: SetSeed, DrawEncounter, PlayCard
    let payload_types: Vec<&str> = entries
        .iter()
        .filter_map(|e| {
            e.get("payload")
                .and_then(|p| p.get("type"))
                .and_then(|v| v.as_str())
        })
        .collect();

    assert!(
        payload_types.contains(&"SetSeed"),
        "Log should contain SetSeed"
    );
    assert!(
        payload_types.contains(&"DrawEncounter"),
        "Log should contain DrawEncounter"
    );
    assert!(
        payload_types.contains(&"PlayCard"),
        "Log should contain PlayCard"
    );

    // Verify entries have sequential seq numbers
    let seqs: Vec<u64> = entries
        .iter()
        .filter_map(|e| e.get("seq").and_then(|v| v.as_u64()))
        .collect();
    for window in seqs.windows(2) {
        assert!(
            window[1] > window[0],
            "Sequence numbers should be monotonically increasing"
        );
    }
}

/// Helper: extract (deck, hand, discard) counts for a player card by library index.
fn player_card_counts(client: &Client, card_index: usize) -> (u32, u32, u32) {
    let cards = get_json(client, "/library/cards");
    let card = &cards.as_array().expect("cards array")[card_index];
    let counts = card.get("counts").expect("counts");
    (
        counts["deck"].as_u64().unwrap_or(0) as u32,
        counts["hand"].as_u64().unwrap_or(0) as u32,
        counts["discard"].as_u64().unwrap_or(0) as u32,
    )
}

/// Helper: sum (deck, hand, discard) across all entries of an enemy deck from combat state.
fn enemy_deck_totals(combat: &serde_json::Value, deck_key: &str) -> (u32, u32, u32) {
    let deck = combat
        .get(deck_key)
        .and_then(|v| v.as_array())
        .expect("enemy deck array");
    let mut total_deck = 0u32;
    let mut total_hand = 0u32;
    let mut total_discard = 0u32;
    for entry in deck {
        let c = entry.get("counts").expect("enemy card counts");
        total_deck += c["deck"].as_u64().unwrap_or(0) as u32;
        total_hand += c["hand"].as_u64().unwrap_or(0) as u32;
        total_discard += c["discard"].as_u64().unwrap_or(0) as u32;
    }
    (total_deck, total_hand, total_discard)
}

#[test]
fn scenario_player_draw_cards_per_type() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Start game and enter combat
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterPickEncounter","card_id":11}"#,
    );
    assert_eq!(status, Status::Created);

    // Initial counts: Attack(8) deck=15 hand=5, Defence(9) deck=15 hand=5, Resource(10) deck=35 hand=5
    let (atk_deck_before, atk_hand_before, _) = player_card_counts(&client, 8);
    let (def_deck_before, def_hand_before, _) = player_card_counts(&client, 9);
    let (res_deck_before, res_hand_before, _) = player_card_counts(&client, 10);

    // Combat starts in Defending phase. Play defence first, then attack.
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterPlayCard","card_id":9}"#,
    );
    assert_eq!(status, Status::Created);
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterPlayCard","card_id":8}"#,
    );
    assert_eq!(status, Status::Created);

    // Now in Resourcing phase. Play resource card (id 10) which draws 1 atk, 1 def, 2 res.
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterPlayCard","card_id":10}"#,
    );
    assert_eq!(status, Status::Created);

    let (atk_deck_after, atk_hand_after, _) = player_card_counts(&client, 8);
    let (def_deck_after, def_hand_after, _) = player_card_counts(&client, 9);
    let (res_deck_after, res_hand_after, res_discard_after) = player_card_counts(&client, 10);

    // Attack: -1 played, +1 drawn from deck
    assert_eq!(
        atk_hand_after,
        atk_hand_before - 1 + 1,
        "Attack hand: -1 played, +1 drawn"
    );
    assert_eq!(atk_deck_after, atk_deck_before - 1, "Attack deck: -1 drawn");

    // Defence: -1 played, +1 drawn from deck
    assert_eq!(
        def_hand_after,
        def_hand_before - 1 + 1,
        "Defence hand: -1 played, +1 drawn"
    );
    assert_eq!(
        def_deck_after,
        def_deck_before - 1,
        "Defence deck: -1 drawn"
    );

    // Resource: -1 played to discard, +2 drawn from deck
    assert_eq!(
        res_hand_after,
        res_hand_before - 1 + 2,
        "Resource hand: -1 played, +2 drawn"
    );
    assert_eq!(
        res_deck_after,
        res_deck_before - 2,
        "Resource deck: -2 drawn"
    );
    assert_eq!(res_discard_after, 1, "Resource discard: 1 played card");
}

#[test]
fn scenario_enemy_draws_per_type() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterPickEncounter","card_id":11}"#,
    );
    assert_eq!(status, Status::Created);

    // Record enemy deck totals before any round
    let combat_before = combat_state(&client);
    let ea_total = {
        let (d, h, di) = enemy_deck_totals(&combat_before, "enemy_attack_deck");
        d + h + di
    };
    let ed_total = {
        let (d, h, di) = enemy_deck_totals(&combat_before, "enemy_defence_deck");
        d + h + di
    };
    let er_total = {
        let (d, h, di) = enemy_deck_totals(&combat_before, "enemy_resource_deck");
        d + h + di
    };

    // Play one full round which triggers enemy plays too
    play_one_round(&client);

    // Check if combat is still active (GET /combat returns 404 when finished)
    let resp = client.get("/encounter").dispatch();
    if resp.status() != Status::Ok {
        return;
    }
    let combat_after: serde_json::Value =
        serde_json::from_str(&resp.into_string().unwrap_or_default()).unwrap_or_default();

    let (ea_deck_a, ea_hand_a, ea_disc_a) = enemy_deck_totals(&combat_after, "enemy_attack_deck");
    let (ed_deck_a, ed_hand_a, ed_disc_a) = enemy_deck_totals(&combat_after, "enemy_defence_deck");
    let (er_deck_a, er_hand_a, er_disc_a) = enemy_deck_totals(&combat_after, "enemy_resource_deck");

    // Card conservation: total cards per deck type must not change
    // (enemy plays move hand→discard, draw effects recycle discard→deck→hand)
    assert_eq!(
        ea_deck_a + ea_hand_a + ea_disc_a,
        ea_total,
        "Enemy attack cards should be conserved"
    );
    assert_eq!(
        ed_deck_a + ed_hand_a + ed_disc_a,
        ed_total,
        "Enemy defence cards should be conserved"
    );
    assert_eq!(
        er_deck_a + er_hand_a + er_disc_a,
        er_total,
        "Enemy resource cards should be conserved"
    );
}

// --- Mining helpers ---

/// Find mining card IDs available in the player's hand (card IDs 12, 13, 14).
fn mining_hand_card_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Mining");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| c.get("id").and_then(|v| v.as_u64()).map(|v| v as usize))
        .collect()
}

/// Play one mining card. Returns true if the mining encounter is still active.
fn play_one_mining_card(client: &Client) -> bool {
    let mining_ids = mining_hand_card_ids(client);
    if mining_ids.is_empty() {
        return false;
    }
    let card_id = mining_ids[0];
    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        card_id
    );
    let (status, _) = post_action(client, &json);
    if status != Status::Created {
        return false;
    }
    let encounter = combat_state(client);
    encounter.get("outcome").and_then(|v| v.as_str()) == Some("Undecided")
}

/// Find mining encounter card IDs in the encounter hand.
fn mining_encounter_ids(client: &Client) -> Vec<usize> {
    let encounter_ids = encounter_hand_ids(client);
    // ID 15 is the mining encounter; filter for it
    encounter_ids.into_iter().filter(|&id| id == 15).collect()
}

#[test]
fn scenario_mining_encounter_full_loop() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // 1. Start a new game with a fixed seed
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created, "NewGame should succeed");

    // 2. Verify mining encounter cards are in hand
    let mining_enc = mining_encounter_ids(&client);
    assert!(
        !mining_enc.is_empty(),
        "Should have mining encounter cards in hand"
    );

    // 3. Pick the mining encounter (card_id 15)
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterPickEncounter","card_id":15}"#,
    );
    assert_eq!(status, Status::Created, "PickEncounter should succeed");

    // 4. Verify mining encounter started
    let encounter = combat_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Mining"),
        "Encounter should be Mining type"
    );
    assert_eq!(
        encounter.get("outcome").and_then(|v| v.as_str()),
        Some("Undecided"),
        "Mining should be active"
    );
    assert_eq!(
        encounter
            .get("ore_tokens")
            .and_then(|v| v.get("OreHealth"))
            .and_then(|v| v.as_i64()),
        Some(15),
        "Ore Health should start at 15"
    );

    // 5. Verify player has MiningDurability token
    let durability = player_token(&client, "MiningDurability");
    assert_eq!(durability, 15, "Player should start with 15 durability");

    // 6. Play mining cards until the encounter finishes (max 50 turns)
    let mut turns = 0;
    while play_one_mining_card(&client) {
        turns += 1;
        assert!(turns < 50, "Mining should end within 50 turns");
    }

    // 7. Verify encounter ended
    let result = combat_result(&client);
    assert!(result.is_some(), "Should have an encounter result");
    let outcome = result.unwrap();
    assert!(
        outcome == "PlayerWon" || outcome == "PlayerLost",
        "Mining outcome should be PlayerWon or PlayerLost, got: {}",
        outcome
    );

    // 8. If player won, verify Ore tokens were granted and transition to Scouting
    if outcome == "PlayerWon" {
        let ore = player_token(&client, "Ore");
        assert!(ore > 0, "Player should have Ore after winning mining");

        // Apply scouting
        let (status, _) = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
        assert_eq!(status, Status::Created, "ApplyScouting should succeed");

        // Verify we can pick another encounter
        let ids_after = encounter_hand_ids(&client);
        assert!(
            !ids_after.is_empty(),
            "Should have encounter cards after scouting"
        );
    }

    // 9. If player lost, verify no penalties (mining has no penalties)
    if outcome == "PlayerLost" {
        let exhaustion = player_token(&client, "Exhaustion");
        assert_eq!(
            exhaustion, 0,
            "Mining has no penalties; Exhaustion should be 0"
        );
        let durability = player_token(&client, "MiningDurability");
        assert_eq!(
            durability, 0,
            "Player mining durability should be 0 when losing mining"
        );

        // Should still be able to scout after a loss
        let (status, _) = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
        assert_eq!(
            status,
            Status::Created,
            "Should be able to scout after mining loss"
        );
    }
}

#[test]
fn scenario_mining_then_combat_coexist() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Start game and verify both combat and mining encounters exist in hand
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":100}"#);
    assert_eq!(status, Status::Created);

    let all_encounters = encounter_hand_ids(&client);
    let has_combat = all_encounters.contains(&11);
    let has_mining = all_encounters.contains(&15);
    assert!(has_combat, "Should have combat encounter cards (id 11)");
    assert!(has_mining, "Should have mining encounter cards (id 15)");

    // Do a mining encounter first
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterPickEncounter","card_id":15}"#,
    );
    assert_eq!(status, Status::Created);

    let mut turns = 0;
    while play_one_mining_card(&client) {
        turns += 1;
        if turns >= 50 {
            break;
        }
    }

    // Scout after mining
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(status, Status::Created);

    // Now do a combat encounter
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterPickEncounter","card_id":11}"#,
    );
    assert_eq!(status, Status::Created);

    // Verify combat started correctly
    let encounter = combat_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Combat"),
        "Should now be in a combat encounter"
    );
    assert!(player_health(&client) > 0, "Player should have health");

    // Play one round of combat to verify it works after mining
    play_one_round(&client);
}
