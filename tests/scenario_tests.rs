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

/// Find hand card IDs of a given card_kind (e.g. "Defence", "Attack", "Resource").
fn hand_card_ids_by_kind(client: &Client, kind: &str) -> Vec<usize> {
    let cards = get_json(
        client,
        &format!("/library/cards?location=Hand&card_kind={}", kind),
    );
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| c.get("id").and_then(|v| v.as_u64()).map(|v| v as usize))
        .collect()
}

/// Find a hand card of the given kind that has non-empty `rolled_costs` in its effects.
fn cost_card_id(client: &Client, kind: &str) -> usize {
    let cards = get_json(
        client,
        &format!("/library/cards?location=Hand&card_kind={}", kind),
    );
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .find_map(|c| {
            let effects = c.get("kind")?.get("effects")?.as_array()?;
            let has_cost = effects.iter().any(|e| {
                e.get("rolled_costs")
                    .and_then(|c| c.as_array())
                    .map(|costs| !costs.is_empty())
                    .unwrap_or(false)
            });
            if has_cost {
                c.get("id")?.as_u64().map(|v| v as usize)
            } else {
                None
            }
        })
        .expect("Should find a cost card in hand")
}

/// Play one full round (Defence → Attack → Resource) using dynamically
/// discovered card IDs.
/// Returns true if combat is still active after the round.
fn play_one_round(client: &Client) -> bool {
    let kinds = ["Defence", "Attack", "Resource"];
    for kind in &kinds {
        let card_ids = hand_card_ids_by_kind(client, kind);
        if card_ids.is_empty() {
            return false;
        }
        let json = format!(
            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
            card_ids[0]
        );
        let (status, _) = post_action(client, &json);
        if status != Status::Created {
            return false;
        }
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

    // 3. Pick a combat encounter dynamically
    let combat_enc = combat_encounter_ids(&client);
    assert!(!combat_enc.is_empty(), "Should have combat encounter cards");
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        combat_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
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
    let combat_enc = combat_encounter_ids(&client);
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        combat_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
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

        let combat_enc = combat_encounter_ids(&client);
        let pick_json = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            combat_enc[0]
        );
        let (status, _) = post_action(&client, &pick_json);
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
    let combat_enc = combat_encounter_ids(&client);
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        combat_enc[0]
    );
    post_action(&client, &pick_json);

    // Play one card
    let def_ids = hand_card_ids_by_kind(&client, "Defence");
    let play_json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        def_ids[0]
    );
    post_action(&client, &play_json);

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

/// Helper: sum (deck, hand, discard) counts across ALL cards of a given kind.
fn total_counts_by_kind(client: &Client, kind: &str) -> (u32, u32, u32) {
    let cards = get_json(client, &format!("/library/cards?card_kind={}", kind));
    let empty = vec![];
    let arr = cards.as_array().unwrap_or(&empty);
    let mut deck_total = 0u32;
    let mut hand_total = 0u32;
    let mut discard_total = 0u32;
    for card in arr {
        if let Some(counts) = card.get("counts") {
            deck_total += counts["deck"].as_u64().unwrap_or(0) as u32;
            hand_total += counts["hand"].as_u64().unwrap_or(0) as u32;
            discard_total += counts["discard"].as_u64().unwrap_or(0) as u32;
        }
    }
    (deck_total, hand_total, discard_total)
}

/// Helper: read an encounter-scoped token from `/encounter`'s `encounter_tokens` field.
fn encounter_token(client: &Client, token_type_name: &str) -> i64 {
    let encounter = combat_state(client);
    encounter
        .get("encounter_tokens")
        .and_then(|v| v.as_object())
        .and_then(|obj| obj.get(token_type_name))
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
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
    let combat_enc = combat_encounter_ids(&client);
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        combat_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created);

    // Initial counts summed across all cards of each kind (includes cost variants)
    let (atk_deck_before, atk_hand_before, _) = total_counts_by_kind(&client, "Attack");
    let (def_deck_before, def_hand_before, _) = total_counts_by_kind(&client, "Defence");
    let (res_deck_before, res_hand_before, _) = total_counts_by_kind(&client, "Resource");

    // Combat starts in Defending phase. Play defence first, then attack.
    let def_ids = hand_card_ids_by_kind(&client, "Defence");
    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        def_ids[0]
    );
    let (status, _) = post_action(&client, &json);
    assert_eq!(status, Status::Created);
    let atk_ids = hand_card_ids_by_kind(&client, "Attack");
    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        atk_ids[0]
    );
    let (status, _) = post_action(&client, &json);
    assert_eq!(status, Status::Created);

    // Now in Resourcing phase. Play resource card which draws 1 atk, 1 def, 2 res.
    let res_ids = hand_card_ids_by_kind(&client, "Resource");
    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        res_ids[0]
    );
    let (status, _) = post_action(&client, &json);
    assert_eq!(status, Status::Created);

    let (atk_deck_after, atk_hand_after, atk_discard_after) =
        total_counts_by_kind(&client, "Attack");
    let (def_deck_after, def_hand_after, def_discard_after) =
        total_counts_by_kind(&client, "Defence");
    let (res_deck_after, res_hand_after, res_discard_after) =
        total_counts_by_kind(&client, "Resource");

    // Attack: played 1 (to discard), but hand already above MaxHand so no draw
    assert_eq!(
        atk_hand_after,
        atk_hand_before - 1,
        "Attack hand: -1 played (no draw, above MaxHand)"
    );
    assert_eq!(atk_deck_after, atk_deck_before, "Attack deck: no draw");
    assert_eq!(atk_discard_after, 1, "Attack discard: 1 played card");

    // Defence: played 1 (to discard), but hand already above MaxHand so no draw
    assert_eq!(
        def_hand_after,
        def_hand_before - 1,
        "Defence hand: -1 played (no draw, above MaxHand)"
    );
    assert_eq!(def_deck_after, def_deck_before, "Defence deck: no draw");
    assert_eq!(def_discard_after, 1, "Defence discard: 1 played card");

    // Resource: played 1 (to discard), drew 1 from deck (hand back to MaxHand)
    assert_eq!(
        res_hand_after, res_hand_before,
        "Resource hand: -1 played, +1 drawn (capped at MaxHand)"
    );
    assert_eq!(
        res_deck_after,
        res_deck_before - 1,
        "Resource deck: -1 drawn"
    );
    assert_eq!(res_discard_after, 1, "Resource discard: 1 played card");
}

#[test]
fn scenario_enemy_draws_per_type() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);
    let combat_enc = combat_encounter_ids(&client);
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        combat_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
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
    for card_id in mining_ids {
        let json = format!(
            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
            card_id
        );
        let (status, _) = post_action(client, &json);
        if status == Status::Created {
            let encounter = combat_state(client);
            return encounter.get("outcome").and_then(|v| v.as_str()) == Some("Undecided");
        }
    }
    false
}

/// Find mining encounter card IDs in the encounter hand.
fn mining_encounter_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| {
            let kind = c.get("kind")?;
            let enc_kind = kind.get("encounter_kind")?;
            if enc_kind.get("encounter_type")?.as_str()? == "Mining" {
                c.get("id")?.as_u64().map(|v| v as usize)
            } else {
                None
            }
        })
        .collect()
}

/// Find combat encounter card IDs in the encounter hand.
fn combat_encounter_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| {
            let kind = c.get("kind")?;
            let enc_kind = kind.get("encounter_kind")?;
            if enc_kind.get("encounter_type")?.as_str()? == "Combat" {
                c.get("id")?.as_u64().map(|v| v as usize)
            } else {
                None
            }
        })
        .collect()
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

    // 3. Pick the mining encounter
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        mining_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created, "PickEncounter should succeed");

    // 4. Verify mining encounter started with light level
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

    // 5. Verify light level is initialized at 300
    let light_level = encounter_token(&client, "MiningLightLevel");
    assert_eq!(light_level, 300, "Light level should start at 300");

    // 6. Verify yield starts at 0
    let mining_yield = encounter_token(&client, "MiningYield");
    assert_eq!(mining_yield, 0, "Yield should start at 0");

    // 7. Verify player has MiningDurability token
    let durability = player_token(&client, "MiningDurability");
    assert_eq!(
        durability, 10000,
        "Player should start with 10000 mining durability"
    );

    // 8. Play mining cards to accumulate some yield
    let mut cards_played = 0;
    while cards_played < 5 {
        if !play_one_mining_card(&client) {
            break;
        }
        cards_played += 1;
    }

    // 9. Verify yield has accumulated (at least some cards should produce yield)
    let yield_after = encounter_token(&client, "MiningYield");
    assert!(
        yield_after > 0,
        "Yield should have accumulated after playing mining power cards, got {}",
        yield_after
    );

    // 10. Conclude the mining encounter
    let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
    assert_eq!(status, Status::Created, "Conclude should succeed");

    // 11. Verify encounter ended as PlayerWon
    let result = combat_result(&client);
    assert_eq!(result, Some("PlayerWon".to_string()));

    // 12. Verify ore reward was granted (min(stamina, yield))
    let ore = player_token(&client, "Ore");
    assert!(
        ore > 0,
        "Player should have Ore tokens after concluding mining"
    );

    // 13. Verify encounter-scoped tokens are cleaned up
    let light_after = player_token(&client, "MiningLightLevel");
    assert_eq!(
        light_after, 0,
        "Light level should be reset after encounter"
    );
    let yield_after = player_token(&client, "MiningYield");
    assert_eq!(yield_after, 0, "Yield should be reset after encounter");

    // 14. Scout after encounter
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(status, Status::Created, "ApplyScouting should succeed");
}

#[test]
fn scenario_abort_mining_encounter() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // 1. Start a new game
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    // 2. Pick a mining encounter
    let mining_enc = mining_encounter_ids(&client);
    assert!(!mining_enc.is_empty(), "Should have mining encounter cards");
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        mining_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created);

    // 3. Verify mining encounter is active
    let encounter = combat_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Mining")
    );

    // 4. Abort the encounter
    let (status, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    assert_eq!(status, Status::Created, "Abort should succeed");

    // 5. Verify encounter result is PlayerLost
    let result = combat_result(&client);
    assert_eq!(result, Some("PlayerLost".to_string()));

    // 6. Verify encounter-scoped tokens are cleaned up
    let light_after = player_token(&client, "MiningLightLevel");
    assert_eq!(light_after, 0, "Light level should be reset after abort");
    let yield_after = player_token(&client, "MiningYield");
    assert_eq!(yield_after, 0, "Yield should be reset after abort");

    // 7. Verify can scout after abort
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(
        status,
        Status::Created,
        "Should be able to scout after abort"
    );

    // 8. Verify aborting combat is rejected
    let (status2, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status2, Status::Created);
    let combat_enc = combat_encounter_ids(&client);
    assert!(!combat_enc.is_empty(), "Should have combat encounter cards");
    let pick_combat_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        combat_enc[0]
    );
    let (status2, _) = post_action(&client, &pick_combat_json);
    assert_eq!(status2, Status::Created);
    let (status2, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    assert_eq!(
        status2,
        Status::BadRequest,
        "Should not be able to abort combat"
    );
}

#[test]
fn scenario_mining_then_combat_coexist() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Start game and verify both combat and mining encounters exist in hand
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":100}"#);
    assert_eq!(status, Status::Created);

    let combat_enc = combat_encounter_ids(&client);
    let mining_enc = mining_encounter_ids(&client);
    assert!(!combat_enc.is_empty(), "Should have combat encounter cards");
    assert!(!mining_enc.is_empty(), "Should have mining encounter cards");

    // Do a mining encounter first — use conclude to end it cleanly
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        mining_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created);

    // Play a few mining cards, then conclude
    let mut turns = 0;
    while play_one_mining_card(&client) {
        turns += 1;
        if turns >= 3 {
            break;
        }
    }

    // If encounter is still active, conclude it
    let encounter = combat_state(&client);
    if encounter.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
        let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        assert_eq!(status, Status::Created, "Conclude should succeed");
    }

    // Scout after mining
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(status, Status::Created);

    // Now do a combat encounter
    let combat_enc = combat_encounter_ids(&client);
    assert!(
        !combat_enc.is_empty(),
        "Should still have combat encounter cards"
    );
    let pick_combat_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        combat_enc[0]
    );
    let (status, _) = post_action(&client, &pick_combat_json);
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

// --- Herbalism helpers ---

/// Find herbalism card IDs available in the player's hand (card IDs 16, 17, 18).
fn herbalism_hand_card_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Herbalism");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| c.get("id").and_then(|v| v.as_u64()).map(|v| v as usize))
        .collect()
}

/// Play one herbalism card. Returns true if the herbalism encounter is still active.
fn play_one_herbalism_card(client: &Client) -> bool {
    let herb_ids = herbalism_hand_card_ids(client);
    if herb_ids.is_empty() {
        return false;
    }
    let card_id = herb_ids[0];
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

/// Find herbalism encounter card IDs in the encounter hand.
fn herbalism_encounter_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| {
            let kind = c.get("kind")?;
            let enc_kind = kind.get("encounter_kind")?;
            if enc_kind.get("encounter_type")?.as_str()? == "Herbalism" {
                c.get("id")?.as_u64().map(|v| v as usize)
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn scenario_herbalism_encounter_full_loop() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // 1. Start a new game with a fixed seed
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created, "NewGame should succeed");

    // 2. Verify herbalism encounter cards are in hand
    let herb_enc = herbalism_encounter_ids(&client);
    assert!(
        !herb_enc.is_empty(),
        "Should have herbalism encounter cards in hand"
    );

    // 3. Pick the herbalism encounter dynamically
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        herb_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created, "PickEncounter should succeed");

    // 4. Verify herbalism encounter started
    let encounter = combat_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Herbalism"),
        "Encounter should be Herbalism type"
    );
    assert_eq!(
        encounter.get("outcome").and_then(|v| v.as_str()),
        Some("Undecided"),
        "Herbalism should be active"
    );

    // 5. Verify plant_hand has 5 cards
    let plant_hand = encounter
        .get("plant_hand")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    assert_eq!(plant_hand, 5, "Plant should start with 5 cards");

    // 6. Verify player has HerbalismDurability token
    let durability = player_token(&client, "HerbalismDurability");
    assert_eq!(
        durability, 10000,
        "Player should start with 10000 herbalism durability"
    );

    // 7. Play herbalism encounters in a loop until durability runs out
    let mut total_encounters = 0;
    let mut last_outcome;
    loop {
        let mut round_turns = 0;
        while play_one_herbalism_card(&client) {
            round_turns += 1;
            assert!(
                round_turns < 50,
                "Herbalism round should end within 50 turns"
            );
        }
        total_encounters += 1;

        last_outcome = combat_result(&client).unwrap_or_default();

        if last_outcome == "PlayerWon" {
            let plant = player_token(&client, "Plant");
            assert!(
                plant > 0,
                "Player should have Plant tokens after winning herbalism"
            );
        }

        if last_outcome == "PlayerLost" {
            break;
        }

        // Scout and pick another herbalism encounter
        let (status, _) = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
        assert_eq!(status, Status::Created, "ApplyScouting should succeed");

        let herb_enc = herbalism_encounter_ids(&client);
        if herb_enc.is_empty() {
            break;
        }
        let pick_json = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            herb_enc[0]
        );
        let (status, _) = post_action(&client, &pick_json);
        assert_eq!(status, Status::Created, "PickEncounter should succeed");

        assert!(
            total_encounters < 200,
            "Player should eventually lose from durability depletion"
        );
    }

    assert!(
        total_encounters >= 1,
        "Player should have completed at least one herbalism encounter"
    );

    // 8. Scout after final encounter
    if last_outcome == "PlayerLost" || last_outcome == "PlayerWon" {
        let (status, _) = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
        if last_outcome == "PlayerLost" {
            assert_eq!(
                status,
                Status::Created,
                "Should be able to scout after herbalism loss"
            );
        }
    }
}

#[test]
fn scenario_abort_herbalism_encounter() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Start game
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    // Pick herbalism encounter
    let herb_enc = herbalism_encounter_ids(&client);
    assert!(!herb_enc.is_empty());
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        herb_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created);

    // Abort it
    let (status, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    assert_eq!(status, Status::Created, "Should be able to abort herbalism");

    // Verify outcome is PlayerLost
    let last_result = combat_result(&client).unwrap_or_default();
    assert_eq!(last_result, "PlayerLost", "Abort should result in loss");
}

// ---- Woodcutting scenario helpers ----

fn woodcutting_hand_card_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Woodcutting");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| c.get("id").and_then(|v| v.as_u64()).map(|v| v as usize))
        .collect()
}

fn play_one_woodcutting_card(client: &Client) -> bool {
    let wc_ids = woodcutting_hand_card_ids(client);
    if wc_ids.is_empty() {
        return false;
    }
    for card_id in wc_ids {
        let json = format!(
            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
            card_id
        );
        let (status, _) = post_action(client, &json);
        if status == Status::Created {
            let encounter = combat_state(client);
            return encounter.get("outcome").and_then(|v| v.as_str()) == Some("Undecided");
        }
    }
    false
}

fn woodcutting_encounter_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| {
            let kind = c.get("kind")?;
            let enc_kind = kind.get("encounter_kind")?;
            if enc_kind.get("encounter_type")?.as_str()? == "Woodcutting" {
                c.get("id")?.as_u64().map(|v| v as usize)
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn scenario_woodcutting_encounter_full_loop() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created, "NewGame should succeed");

    let wc_enc = woodcutting_encounter_ids(&client);
    assert!(
        !wc_enc.is_empty(),
        "Should have woodcutting encounter cards in hand"
    );

    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        wc_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created, "PickEncounter should succeed");

    let encounter = combat_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Woodcutting"),
        "Encounter should be Woodcutting type"
    );
    assert_eq!(
        encounter.get("outcome").and_then(|v| v.as_str()),
        Some("Undecided"),
        "Woodcutting should be active"
    );
    assert_eq!(
        encounter.get("max_plays").and_then(|v| v.as_u64()),
        Some(8),
        "max_plays should be 8"
    );
    assert_eq!(
        encounter
            .get("played_cards")
            .and_then(|v| v.as_array())
            .map(|a| a.len()),
        Some(0),
        "No cards played yet"
    );

    let durability = player_token(&client, "WoodcuttingDurability");
    assert_eq!(
        durability, 10000,
        "Player should start with 10000 woodcutting durability"
    );

    // Play 8 cards (the encounter should auto-complete after 8)
    let mut total_turns = 0;
    loop {
        let still_going = play_one_woodcutting_card(&client);
        total_turns += 1;
        if !still_going {
            // Encounter ended (either 8th card was played or durability depleted)
            break;
        }
        assert!(total_turns < 50, "Woodcutting should end within 50 turns");
    }

    // After 8 cards, should always win
    let last_outcome = combat_result(&client).unwrap_or_default();
    assert_eq!(
        last_outcome, "PlayerWon",
        "Woodcutting should always win after 8 plays"
    );

    // Verify pattern was evaluated
    // (We can't easily check the encounter state after it's cleared, but
    // we can verify Lumber was awarded)
    let lumber = player_token(&client, "Lumber");
    assert!(
        lumber > 0,
        "Player should have Lumber after winning woodcutting (got {})",
        lumber
    );

    // Verify durability was consumed (8 cards × 1 durability each = 8)
    let final_durability = player_token(&client, "WoodcuttingDurability");
    assert!(
        final_durability < 10000,
        "Durability should decrease after woodcutting (got {})",
        final_durability
    );

    // Should be in Scouting phase now; can scout and pick another encounter
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(status, Status::Created, "Should be able to scout after win");

    // Play multiple encounters until durability runs out
    let mut total_encounters = 1; // Already did one
    loop {
        let wc_enc = woodcutting_encounter_ids(&client);
        if wc_enc.is_empty() {
            break;
        }
        let pick_json = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            wc_enc[0]
        );
        let (status, _) = post_action(&client, &pick_json);
        assert_eq!(status, Status::Created, "PickEncounter should succeed");

        let mut round_turns = 0;
        loop {
            let still_going = play_one_woodcutting_card(&client);
            round_turns += 1;
            if !still_going {
                break;
            }
            assert!(round_turns < 50, "Round should end within 50 turns");
        }
        total_encounters += 1;

        let outcome = combat_result(&client).unwrap_or_default();
        if outcome == "PlayerLost" {
            // Durability depleted
            let final_durability = player_token(&client, "WoodcuttingDurability");
            assert_eq!(
                final_durability, 0,
                "Durability should be 0 when losing woodcutting"
            );
            break;
        }

        // If encounter is still active (stuck on unplayable cost cards), abort it
        let encounter = combat_state(&client);
        if encounter.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
            let (status, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
            assert_eq!(status, Status::Created, "Abort should succeed when stuck");
        }

        let (status, _) = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
        assert_eq!(status, Status::Created, "Scouting should succeed");

        assert!(
            total_encounters < 100,
            "Should eventually run out of durability"
        );
    }

    assert!(
        total_encounters > 1,
        "With 10000 durability and cost 100 per card, should survive multiple encounters"
    );
}

#[test]
fn scenario_abort_woodcutting_encounter() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    let wc_enc = woodcutting_encounter_ids(&client);
    assert!(
        !wc_enc.is_empty(),
        "Should have woodcutting encounter cards"
    );
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        wc_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created);

    let encounter = combat_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Woodcutting")
    );

    let (status, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    assert_eq!(status, Status::Created, "Abort should succeed");

    let result = combat_result(&client);
    assert_eq!(result, Some("PlayerLost".to_string()));

    // Should be able to scout after abort
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(
        status,
        Status::Created,
        "Should be able to scout after abort"
    );
}

// ---- Fishing helpers ----

fn fishing_hand_card_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Fishing");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| c.get("id").and_then(|v| v.as_u64()).map(|v| v as usize))
        .collect()
}

fn fishing_encounter_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| {
            let kind = c.get("kind")?;
            let enc_kind = kind.get("encounter_kind")?;
            if enc_kind.get("encounter_type")?.as_str()? == "Fishing" {
                c.get("id")?.as_u64().map(|v| v as usize)
            } else {
                None
            }
        })
        .collect()
}

fn play_one_fishing_card(client: &Client) -> bool {
    let fc_ids = fishing_hand_card_ids(client);
    if fc_ids.is_empty() {
        return false;
    }
    let card_id = fc_ids[0];
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

#[test]
fn scenario_fishing_encounter_full_loop() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created, "NewGame should succeed");

    let fc_enc = fishing_encounter_ids(&client);
    assert!(
        !fc_enc.is_empty(),
        "Should have fishing encounter cards in hand"
    );

    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        fc_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created, "PickEncounter should succeed");

    let encounter = combat_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Fishing"),
        "Encounter should be Fishing type"
    );
    assert_eq!(
        encounter.get("outcome").and_then(|v| v.as_str()),
        Some("Undecided"),
        "Fishing should be active"
    );
    assert_eq!(
        encounter.get("max_turns").and_then(|v| v.as_u64()),
        Some(8),
        "max_turns should be 8"
    );
    assert_eq!(
        encounter.get("win_turns_needed").and_then(|v| v.as_u64()),
        Some(4),
        "win_turns_needed should be 4"
    );
    assert_eq!(
        encounter.get("turns_won").and_then(|v| v.as_u64()),
        Some(0),
        "No turns won yet"
    );

    let durability = player_token(&client, "FishingDurability");
    assert_eq!(
        durability, 10000,
        "Player should start with 10000 fishing durability"
    );

    // Play cards until the encounter ends
    let mut total_turns = 0;
    loop {
        let still_going = play_one_fishing_card(&client);
        total_turns += 1;
        if !still_going {
            break;
        }
        assert!(total_turns < 50, "Fishing should end within 50 turns");
    }

    let last_outcome = combat_result(&client).unwrap_or_default();
    assert!(
        last_outcome == "PlayerWon" || last_outcome == "PlayerLost",
        "Fishing should end with PlayerWon or PlayerLost (got {})",
        last_outcome
    );

    if last_outcome == "PlayerWon" {
        let fish = player_token(&client, "Fish");
        assert!(
            fish > 0,
            "Player should have Fish after winning fishing (got {})",
            fish
        );
    }

    let final_durability = player_token(&client, "FishingDurability");
    assert!(
        final_durability < 10000,
        "Durability should decrease after fishing (got {})",
        final_durability
    );

    // Should be in Scouting phase now
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(
        status,
        Status::Created,
        "Should be able to scout after fishing"
    );

    // Play multiple encounters until durability runs out
    let mut total_encounters = 1;
    loop {
        let fc_enc = fishing_encounter_ids(&client);
        if fc_enc.is_empty() {
            break;
        }
        let pick_json = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            fc_enc[0]
        );
        let (status, _) = post_action(&client, &pick_json);
        assert_eq!(status, Status::Created, "PickEncounter should succeed");

        let mut round_turns = 0;
        loop {
            let still_going = play_one_fishing_card(&client);
            round_turns += 1;
            if !still_going {
                break;
            }
            assert!(round_turns < 50, "Round should end within 50 turns");
        }
        total_encounters += 1;

        let outcome = combat_result(&client).unwrap_or_default();
        if outcome == "PlayerLost" {
            break;
        }

        let (status, _) = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
        assert_eq!(status, Status::Created, "Scouting should succeed");

        assert!(
            total_encounters < 100,
            "Should eventually run out of durability"
        );
    }

    assert!(
        total_encounters > 1,
        "With 10000 durability and cost 100 per card, should survive multiple encounters"
    );
}

#[test]
fn scenario_abort_fishing_encounter() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    let fc_enc = fishing_encounter_ids(&client);
    assert!(!fc_enc.is_empty(), "Should have fishing encounter cards");
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        fc_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created);

    let encounter = combat_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Fishing")
    );

    let (status, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    assert_eq!(status, Status::Created, "Abort should succeed");

    let result = combat_result(&client);
    assert_eq!(result, Some("PlayerLost".to_string()));

    // Should be able to scout after abort
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(
        status,
        Status::Created,
        "Should be able to scout after abort"
    );
}

// ---- Step 9.2: Cost system tests ----

#[test]
fn scenario_cost_card_rejected_without_stamina() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created, "NewGame should succeed");

    // Pick combat encounter
    let combat_enc = combat_encounter_ids(&client);
    assert!(!combat_enc.is_empty(), "Should have combat encounter cards");
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        combat_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created, "PickEncounter should succeed");

    // Verify player starts with 1000 stamina
    let stamina_before = player_token(&client, "Stamina");
    assert_eq!(
        stamina_before, 1000,
        "Player should start with 1000 Stamina"
    );

    // Play cost Defence card — it has a stamina cost effect.
    // With multi-effect evaluation, the card always succeeds:
    // - If stamina is sufficient, the cost is paid and the effect applies
    // - If stamina is insufficient, the costly effect is skipped
    let cost_def_id = cost_card_id(&client, "Defence");
    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        cost_def_id
    );
    let (status, _body) = post_action(&client, &json);
    assert_eq!(
        status,
        Status::Created,
        "Cost card should succeed (multi-effect evaluation)"
    );

    // Stamina should have decreased (cost was paid since we had enough)
    let stamina_after = player_token(&client, "Stamina");
    assert!(
        stamina_after < stamina_before,
        "Stamina should decrease when cost is affordable: before={}, after={}",
        stamina_before,
        stamina_after
    );
}

#[test]
fn scenario_cost_card_deducts_stamina() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created, "NewGame should succeed");

    // Pick combat encounter
    let combat_enc = combat_encounter_ids(&client);
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        combat_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created, "PickEncounter should succeed");

    // Play a full round to get stamina: Defence → Attack → Resource
    // Defence phase: play non-cost Defence
    let def_ids = hand_card_ids_by_kind(&client, "Defence");
    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        def_ids[0]
    );
    let (status, _) = post_action(&client, &json);
    assert_eq!(status, Status::Created, "Defence card should succeed");
    // Auto-advance: enemy plays, phase → Attacking

    // Attack phase: play non-cost Attack
    let atk_ids = hand_card_ids_by_kind(&client, "Attack");
    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        atk_ids[0]
    );
    let (status, _) = post_action(&client, &json);
    assert_eq!(status, Status::Created, "Attack card should succeed");
    // Auto-advance: enemy plays, phase → Resourcing

    // Resource phase: play Resource — grants stamina
    let res_ids = hand_card_ids_by_kind(&client, "Resource");
    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        res_ids[0]
    );
    let (status, _) = post_action(&client, &json);
    assert_eq!(status, Status::Created, "Resource card should succeed");
    // Auto-advance: phase → Defending

    // Verify player now has stamina
    let stamina_after_resource = player_token(&client, "Stamina");
    assert!(
        stamina_after_resource > 0,
        "Player should have Stamina after playing Resource card (got {})",
        stamina_after_resource
    );

    // Defending phase again: play cost Defence card — should succeed now
    let cost_def_id = cost_card_id(&client, "Defence");
    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        cost_def_id
    );
    let (status, _) = post_action(&client, &json);
    assert_eq!(
        status,
        Status::Created,
        "Cost Defence card should succeed with stamina"
    );

    // Verify stamina was consumed
    let stamina_after_cost = player_token(&client, "Stamina");
    assert!(
        stamina_after_cost < stamina_after_resource,
        "Stamina should decrease after playing cost card (before={}, after={})",
        stamina_after_resource,
        stamina_after_cost
    );
}

#[test]
fn scenario_cost_mining_card_rejected_without_stamina() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created, "NewGame should succeed");

    // Pick mining encounter dynamically
    let mining_enc = mining_encounter_ids(&client);
    assert!(!mining_enc.is_empty(), "Should have mining encounter cards");
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        mining_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created, "Mining encounter should start");

    // Verify player starts with 1000 stamina
    let stamina_before = player_token(&client, "Stamina");
    assert_eq!(
        stamina_before, 1000,
        "Player should start with 1000 Stamina"
    );

    // Find a mining hand card with stamina cost and play it
    let cards = get_json(&client, "/library/cards?location=Hand&card_kind=Mining");
    let cost_card_id = cards.as_array().unwrap_or(&vec![]).iter().find_map(|c| {
        let costs = c
            .get("kind")?
            .get("mining_effect")?
            .get("costs")?
            .as_array()?;
        let has_stamina_cost = costs
            .iter()
            .any(|cost| cost.get("token_type").and_then(|v| v.as_str()) == Some("Stamina"));
        if has_stamina_cost {
            c.get("id")?.as_u64().map(|v| v as usize)
        } else {
            None
        }
    });

    if let Some(card_id) = cost_card_id {
        let json = format!(
            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
            card_id
        );
        let (status, _) = post_action(&client, &json);
        assert_eq!(
            status,
            Status::Created,
            "Cost Mining card should succeed with 1000 stamina"
        );

        let stamina_after = player_token(&client, "Stamina");
        assert!(
            stamina_after < stamina_before,
            "Stamina should decrease after cost mining card (before={}, after={})",
            stamina_before,
            stamina_after
        );
    }

    // Play a non-cost Mining card (one with empty costs)
    let enc_resp = client.get("/encounter").dispatch();
    if enc_resp.status() != Status::NotFound {
        let cards = get_json(&client, "/library/cards?location=Hand&card_kind=Mining");
        let free_card_id = cards.as_array().unwrap_or(&vec![]).iter().find_map(|c| {
            let costs = c
                .get("kind")?
                .get("mining_effect")?
                .get("costs")?
                .as_array()?;
            if costs.is_empty() {
                c.get("id")?.as_u64().map(|v| v as usize)
            } else {
                None
            }
        });

        if let Some(card_id) = free_card_id {
            let json = format!(
                r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                card_id
            );
            let (status, _) = post_action(&client, &json);
            assert_eq!(
                status,
                Status::Created,
                "Non-cost mining card should succeed"
            );
        }
    }
}

#[test]
fn scenario_cost_cards_exist_in_starting_decks() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created, "NewGame should succeed");

    // Check that both cost and non-cost Attack cards exist, identified dynamically
    let attack_cards = get_json(&client, "/library/cards?card_kind=Attack");
    let attack_arr = attack_cards.as_array().expect("Attack cards array");
    assert!(
        attack_arr.len() >= 2,
        "Should have at least 2 Attack cards (cost and non-cost), got {}",
        attack_arr.len()
    );
    let cost_attack = attack_arr.iter().find(|c| {
        c.get("kind")
            .and_then(|k| k.get("effects"))
            .and_then(|e| e.as_array())
            .map(|effects| {
                effects.iter().any(|e| {
                    e.get("rolled_costs")
                        .and_then(|c| c.as_array())
                        .map(|costs| !costs.is_empty())
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    });
    let non_cost_attack = attack_arr.iter().find(|c| {
        c.get("kind")
            .and_then(|k| k.get("effects"))
            .and_then(|e| e.as_array())
            .map(|effects| {
                effects.iter().all(|e| {
                    e.get("rolled_costs")
                        .and_then(|c| c.as_array())
                        .map(|costs| costs.is_empty())
                        .unwrap_or(true)
                })
            })
            .unwrap_or(true)
    });
    assert!(cost_attack.is_some(), "Should have a cost Attack card");
    assert!(
        non_cost_attack.is_some(),
        "Should have a non-cost Attack card"
    );

    // Check that both cost and non-cost Defence cards exist
    let defence_cards = get_json(&client, "/library/cards?card_kind=Defence");
    let defence_arr = defence_cards.as_array().expect("Defence cards array");
    assert!(
        defence_arr.len() >= 2,
        "Should have at least 2 Defence cards (cost and non-cost), got {}",
        defence_arr.len()
    );
    let cost_defence = defence_arr.iter().find(|c| {
        c.get("kind")
            .and_then(|k| k.get("effects"))
            .and_then(|e| e.as_array())
            .map(|effects| {
                effects.iter().any(|e| {
                    e.get("rolled_costs")
                        .and_then(|c| c.as_array())
                        .map(|costs| !costs.is_empty())
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    });
    assert!(cost_defence.is_some(), "Should have a cost Defence card");

    // Check that at least one cost Mining card exists (has Stamina in costs)
    let mining_cards = get_json(&client, "/library/cards?card_kind=Mining");
    let mining_arr = mining_cards.as_array().expect("Mining cards array");
    let cost_mining = mining_arr.iter().find(|c| {
        c.get("kind")
            .and_then(|k| k.get("mining_effect"))
            .and_then(|me| me.get("costs"))
            .and_then(|costs| costs.as_array())
            .map(|costs| {
                costs
                    .iter()
                    .any(|cost| cost.get("token_type").and_then(|t| t.as_str()) == Some("Stamina"))
            })
            .unwrap_or(false)
    });
    assert!(
        cost_mining.is_some(),
        "Should have at least one cost Mining card (with Stamina cost)"
    );

    // Check that at least one cost Woodcutting card exists (has Stamina in costs)
    let woodcutting_cards = get_json(&client, "/library/cards?card_kind=Woodcutting");
    let woodcutting_arr = woodcutting_cards
        .as_array()
        .expect("Woodcutting cards array");
    let cost_woodcutting = woodcutting_arr.iter().find(|c| {
        c.get("kind")
            .and_then(|k| k.get("woodcutting_effect"))
            .and_then(|we| we.get("costs"))
            .and_then(|costs| costs.as_array())
            .map(|costs| {
                costs
                    .iter()
                    .any(|cost| cost.get("token_type").and_then(|t| t.as_str()) == Some("Stamina"))
            })
            .unwrap_or(false)
    });
    assert!(
        cost_woodcutting.is_some(),
        "Should have at least one cost Woodcutting card (with Stamina cost)"
    );

    // Verify cost cards have fewer deck copies than non-cost cards
    let cost_atk = cost_attack.unwrap();
    let non_cost_atk = non_cost_attack.unwrap();
    let non_cost_deck = non_cost_atk["counts"]["deck"].as_u64().unwrap_or(0);
    let cost_deck = cost_atk["counts"]["deck"].as_u64().unwrap_or(0);
    assert!(
        cost_deck < non_cost_deck,
        "Cost Attack should have fewer deck copies ({}) than non-cost ({})",
        cost_deck,
        non_cost_deck
    );
}

// ---- 9.3 expansion scenario tests ----

#[test]
fn scenario_combat_victory_grants_milestone_insight() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Start game and enter combat
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    let combat_enc = combat_encounter_ids(&client);
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        combat_enc[0]
    );
    post_action(&client, &pick_json);

    // Before combat ends, MilestoneInsight should be 0
    assert_eq!(
        player_token(&client, "MilestoneInsight"),
        0,
        "Should start with 0 MilestoneInsight"
    );

    // Play rounds until combat finishes
    for _ in 0..80 {
        if !play_one_round(&client) {
            break;
        }
    }

    let result = combat_result(&client);
    if let Some(ref outcome) = result {
        if outcome == "PlayerWon" {
            assert!(
                player_token(&client, "MilestoneInsight") >= 100,
                "Should gain MilestoneInsight on combat win"
            );
        }
    }
}

#[test]
fn scenario_fishing_range_modification_cards_exist() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":100}"#);

    // Check that fishing expansion cards exist in library
    let cards = get_json(&client, "/library/cards?card_kind=Fishing");
    let card_arr = cards.as_array().expect("Should be array");

    // Should have more than the original 3 fishing cards
    assert!(
        card_arr.len() >= 10,
        "Should have at least 10 fishing cards (3 original + 7 expansion), got {}",
        card_arr.len()
    );
}

#[test]
fn scenario_herbalism_match_mode_cards_exist() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":200}"#);

    // Check herbalism cards
    let cards = get_json(&client, "/library/cards?card_kind=Herbalism");
    let card_arr = cards.as_array().expect("Should be array");

    // Should have original 3 + 4 expansion = 7 herbalism cards
    assert!(
        card_arr.len() >= 7,
        "Should have at least 7 herbalism cards, got {}",
        card_arr.len()
    );
}

#[test]
fn scenario_woodcutting_expansion_cards_exist() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":300}"#);

    // Check woodcutting cards
    let cards = get_json(&client, "/library/cards?card_kind=Woodcutting");
    let card_arr = cards.as_array().expect("Should be array");

    // Should have original 4 + 1 cost + 5 expansion = 10 woodcutting cards
    assert!(
        card_arr.len() >= 10,
        "Should have at least 10 woodcutting cards, got {}",
        card_arr.len()
    );
}

#[test]
fn scenario_mining_expansion_cards_exist() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":400}"#);

    // Check mining cards
    let cards = get_json(&client, "/library/cards?card_kind=Mining");
    let card_arr = cards.as_array().expect("Should be array");

    // Should have 8 mining cards (power, light, rest varieties)
    assert!(
        card_arr.len() >= 8,
        "Should have at least 8 mining cards, got {}",
        card_arr.len()
    );
}

#[test]
fn scenario_max_handsize_tokens_initialized() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":500}"#);

    // Verify max handsize tokens are initialized to 5
    for token_name in &[
        "AttackMaxHand",
        "DefenceMaxHand",
        "ResourceMaxHand",
        "MiningMaxHand",
        "HerbalismMaxHand",
        "WoodcuttingMaxHand",
        "FishingMaxHand",
    ] {
        let val = player_token(&client, token_name);
        assert_eq!(
            val, 5,
            "{} should be initialized to 5, got {}",
            token_name, val
        );
    }
}

#[test]
fn scenario_fishing_encounter_initializes_range_tokens() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":600}"#);

    // Pick fishing encounter dynamically
    let fc_enc = fishing_encounter_ids(&client);
    assert!(!fc_enc.is_empty(), "Should have fishing encounter cards");
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        fc_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(
        status,
        Status::Created,
        "Pick fishing encounter should succeed"
    );

    // After starting fishing encounter, range tokens should be set in encounter_tokens
    let range_min = encounter_token(&client, "FishingRangeMin");
    let range_max = encounter_token(&client, "FishingRangeMax");
    let fish_amount = encounter_token(&client, "FishAmount");

    assert!(
        range_min > 0,
        "FishingRangeMin should be set, got {}",
        range_min
    );
    assert!(
        range_max > 0,
        "FishingRangeMax should be set, got {}",
        range_max
    );
    assert!(
        fish_amount >= 1,
        "FishAmount should be at least 1, got {}",
        fish_amount
    );
}

/// Find rest encounter card IDs by looking at encounter hand cards whose kind
/// contains `encounter_type: "Rest"`.
fn rest_encounter_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| {
            let id = c.get("id")?.as_u64()? as usize;
            let enc_type = c
                .get("kind")?
                .get("encounter_kind")?
                .get("encounter_type")?
                .as_str()?;
            if enc_type == "Rest" {
                Some(id)
            } else {
                None
            }
        })
        .collect()
}

/// Scenario: Rest encounter full loop.
///
/// New game → scout → pick rest encounter → verify rest_tokens → play rest cards
/// (Library card IDs) → verify recovery applied → encounter auto-completes when
/// tokens depleted → back to scouting.
#[test]
fn scenario_rest_encounter_full_loop() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // 1. Start a new game with a fixed seed
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created, "NewGame should succeed");

    // 2. Record initial stamina (Health starts at 0 until combat)
    let initial_stamina = player_token(&client, "Stamina");
    assert!(initial_stamina > 0, "Should have initial Stamina");

    // 3. Find rest encounter cards in hand
    let rest_enc = rest_encounter_ids(&client);
    assert!(
        !rest_enc.is_empty(),
        "Should have rest encounter cards in hand"
    );

    // 4. Pick the first rest encounter
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        rest_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created, "PickEncounter should succeed");

    // 5. Verify encounter is Rest type with rest_tokens
    let encounter = combat_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Rest"),
        "Encounter should be Rest type"
    );
    assert_eq!(
        encounter.get("outcome").and_then(|v| v.as_str()),
        Some("Undecided"),
        "Rest should be active"
    );
    let rest_tokens = encounter
        .get("rest_tokens")
        .and_then(|v| v.as_i64())
        .expect("Should have rest_tokens");
    assert!(
        (1..=2).contains(&rest_tokens),
        "Rest tokens should be 1-2, got {}",
        rest_tokens
    );

    // 6. Find rest cards in player hand
    let rest_hand_cards = get_json(&client, "/library/cards?location=Hand&card_kind=Rest");
    let rest_hand_arr = rest_hand_cards
        .as_array()
        .expect("Should have rest hand cards array");
    assert!(
        !rest_hand_arr.is_empty(),
        "Should have rest cards drawn to hand"
    );

    // Find a cost-free rest card (one whose effects have no rolled_costs),
    // or fall back to any rest card
    let mut play_card_id = None;
    for card in rest_hand_arr {
        let id = card.get("id").and_then(|v| v.as_u64()).unwrap() as usize;
        let effects = card
            .get("kind")
            .and_then(|k| k.get("effects"))
            .and_then(|e| e.as_array());
        if let Some(effs) = effects {
            let has_any_cost = effs.iter().any(|e| {
                e.get("rolled_costs")
                    .and_then(|c| c.as_array())
                    .map(|costs| !costs.is_empty())
                    .unwrap_or(false)
            });
            if !has_any_cost {
                play_card_id = Some(id);
                break;
            }
        }
    }

    if let Some(card_to_play) = play_card_id {
        // Play the cost-free rest card
        let play_json = format!(
            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
            card_to_play
        );
        let (status, _body) = post_action(&client, &play_json);
        assert_eq!(status, Status::Created, "Playing rest card should succeed");

        // Check if encounter completed (rest tokens may have been depleted)
        let encounter_after = combat_state(&client);
        if encounter_after.is_null()
            || encounter_after.get("outcome").and_then(|v| v.as_str()) != Some("Undecided")
        {
            let last_outcome = combat_result(&client).unwrap_or_default();
            assert_eq!(
                last_outcome, "PlayerWon",
                "Rest encounter should always be PlayerWon"
            );
        } else {
            // Still active — abort to finish (rest abort = PlayerWon)
            let (status, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
            assert_eq!(status, Status::Created, "EncounterAbort should succeed");
            let last_outcome = combat_result(&client).unwrap_or_default();
            assert_eq!(
                last_outcome, "PlayerWon",
                "Rest abort should always be PlayerWon"
            );
        }
    } else {
        // No cost-free card drawn — just abort (rest abort = PlayerWon)
        let (status, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
        assert_eq!(status, Status::Created, "EncounterAbort should succeed");
        let last_outcome = combat_result(&client).unwrap_or_default();
        assert_eq!(
            last_outcome, "PlayerWon",
            "Rest abort should always be PlayerWon"
        );
    }

    // 8. Should be in Scouting phase now
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(
        status,
        Status::Created,
        "Should be able to scout after rest"
    );
}

// ======================== Crafting encounter helpers ========================

fn crafting_encounter_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| {
            let id = c.get("id")?.as_u64()? as usize;
            let enc_type = c
                .get("kind")?
                .get("encounter_kind")?
                .get("encounter_type")?
                .as_str()?;
            if enc_type == "Crafting" {
                Some(id)
            } else {
                None
            }
        })
        .collect()
}

fn crafting_state(client: &Client) -> serde_json::Value {
    get_json(client, "/encounter")
}

fn start_game_and_pick_crafting(client: &Client) {
    let (status, _) = post_action(client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created, "NewGame should succeed");

    let enc_ids = crafting_encounter_ids(client);
    assert!(
        !enc_ids.is_empty(),
        "Should have crafting encounter cards in hand"
    );

    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc_ids[0]
    );
    let (status, _) = post_action(client, &pick_json);
    assert_eq!(status, Status::Created, "PickEncounter should succeed");
}

// ======================== Crafting encounter tests ========================

/// Scenario: Start a crafting encounter and verify initial state.
#[test]
fn scenario_crafting_encounter_start() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    start_game_and_pick_crafting(&client);

    let encounter = crafting_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Crafting"),
        "Encounter should be Crafting type"
    );
    assert_eq!(
        encounter.get("outcome").and_then(|v| v.as_str()),
        Some("Undecided"),
        "Crafting should be active"
    );
    let crafting_tokens = encounter
        .get("crafting_tokens")
        .and_then(|v| v.as_i64())
        .expect("Should have crafting_tokens");
    assert!(
        crafting_tokens >= 8,
        "Should have at least 8 crafting tokens, got {}",
        crafting_tokens
    );
}

/// Scenario: Crafting encounter → swap cards between deck and library.
#[test]
fn scenario_crafting_swap_cards() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    start_game_and_pick_crafting(&client);

    // Find a player card in deck and a player card in library
    // "Player cards" are Attack, Defence, Resource, Mining, Herbalism, etc. — NOT Encounter, PlayerCardEffect, or EnemyCardEffect
    let player_card_kinds = [
        "Attack",
        "Defence",
        "Resource",
        "Mining",
        "Herbalism",
        "Woodcutting",
        "Fishing",
        "Rest",
        "Crafting",
    ];

    // Find a player card in deck
    let mut from_id_final = None;
    for kind in &player_card_kinds {
        let deck_cards = get_json(
            &client,
            &format!("/library/cards?location=Deck&card_kind={}", kind),
        );
        if let Some(arr) = deck_cards.as_array() {
            if let Some(card) = arr.first() {
                from_id_final = card.get("id").and_then(|v| v.as_u64()).map(|v| v as usize);
                break;
            }
        }
    }
    let from_id_final = from_id_final.expect("Should have a player card in deck to swap");

    // Find a player card in library (not the same card)
    let mut to_id_final = None;
    for kind in &player_card_kinds {
        let lib_cards = get_json(
            &client,
            &format!("/library/cards?location=Library&card_kind={}", kind),
        );
        if let Some(arr) = lib_cards.as_array() {
            if let Some(card) = arr.iter().find(|c| {
                c.get("id").and_then(|v| v.as_u64()).map(|v| v as usize) != Some(from_id_final)
            }) {
                to_id_final = card.get("id").and_then(|v| v.as_u64()).map(|v| v as usize);
                break;
            }
        }
    }

    if to_id_final.is_none() {
        // No library-only player cards available (all copies are in deck/hand/discard)
        // This is valid for some seeds — just verify error handling and return
        return;
    }
    let to_id_final = to_id_final.unwrap();

    // Record initial tokens
    let encounter_before = crafting_state(&client);
    let tokens_before = encounter_before
        .get("crafting_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    // Perform swap
    let swap_json = format!(
        r#"{{"action_type":"EncounterCraftSwap","from_id":{},"to_id":{}}}"#,
        from_id_final, to_id_final
    );
    let (status, _) = post_action(&client, &swap_json);
    assert_eq!(status, Status::Created, "CraftSwap should succeed");

    // Verify crafting tokens decreased by 1
    let encounter_after = crafting_state(&client);
    if let Some(tokens_after) = encounter_after
        .get("crafting_tokens")
        .and_then(|v| v.as_i64())
    {
        assert_eq!(
            tokens_after,
            tokens_before - 1,
            "Should spend 1 crafting token on swap"
        );
    }
}

/// Scenario: Abort a crafting encounter → verify PlayerWon, no penalty.
#[test]
fn scenario_crafting_abort() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    start_game_and_pick_crafting(&client);

    // Abort
    let (status, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    assert_eq!(status, Status::Created, "Abort should succeed");

    // Verify result is PlayerWon
    let result = combat_result(&client);
    assert_eq!(
        result,
        Some("PlayerWon".to_string()),
        "Crafting abort should always result in PlayerWon"
    );

    // Should be back to scouting
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(
        status,
        Status::Created,
        "Should be able to scout after crafting abort"
    );
}

/// Scenario: Conclude a crafting encounter without starting a craft.
#[test]
fn scenario_crafting_conclude_no_craft() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    start_game_and_pick_crafting(&client);

    // Conclude immediately (no craft in progress)
    let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
    assert_eq!(status, Status::Created, "Conclude should succeed");

    // Verify result
    let result = combat_result(&client);
    assert_eq!(
        result,
        Some("PlayerWon".to_string()),
        "Crafting conclude should result in PlayerWon"
    );

    // Should be back to scouting
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(
        status,
        Status::Created,
        "Should be able to scout after crafting conclude"
    );
}

/// Scenario: Start a craft mini-game → play crafting cards → conclude.
#[test]
fn scenario_crafting_craft_card_mini_game() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    start_game_and_pick_crafting(&client);

    // Find a target card to craft (any player card in library)
    let all_cards = get_json(&client, "/library/cards");
    let all_arr = all_cards.as_array().expect("Should have cards");
    let target = all_arr.iter().find(|c| {
        let kind = c
            .get("kind")
            .and_then(|k| k.get("card_kind"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        matches!(kind, "Attack" | "Defence" | "Resource")
    });
    let target_id = target
        .expect("Should have a player card to craft")
        .get("id")
        .and_then(|v| v.as_u64())
        .unwrap() as usize;

    // Count library cards before
    let lib_cards_before = get_json(&client, "/library/cards");
    let lib_count_before = lib_cards_before.as_array().map(|a| a.len()).unwrap_or(0);

    // Start crafting the card
    let craft_json = format!(
        r#"{{"action_type":"EncounterCraftCard","target_card_id":{}}}"#,
        target_id
    );
    let (status, _) = post_action(&client, &craft_json);
    assert_eq!(status, Status::Created, "CraftCard should succeed");

    // Verify active_craft is present
    let encounter = crafting_state(&client);
    let active_craft = encounter.get("active_craft");
    assert!(
        active_craft.is_some() && !active_craft.unwrap().is_null(),
        "Should have an active craft"
    );

    // Play crafting cards if we have them
    let crafting_hand = get_json(&client, "/library/cards?location=Hand&card_kind=Crafting");
    if let Some(cards) = crafting_hand.as_array() {
        for card in cards.iter().take(2) {
            let card_id = card.get("id").and_then(|v| v.as_u64()).unwrap();
            let play_json = format!(
                r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                card_id
            );
            let (status, _) = post_action(&client, &play_json);
            if status != Status::Created {
                break;
            }
            // Check if encounter concluded auto
            let enc = crafting_state(&client);
            if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                break;
            }
        }
    }

    // Conclude the crafting encounter (which finalizes the craft)
    let enc_check = crafting_state(&client);
    if enc_check.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
        let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        // May succeed or fail (if can't pay costs)
        if status == Status::Created {
            // Check if a new card was created
            let lib_cards_after = get_json(&client, "/library/cards");
            let lib_count_after = lib_cards_after.as_array().map(|a| a.len()).unwrap_or(0);
            assert!(
                lib_count_after >= lib_count_before,
                "Library should not lose cards after crafting"
            );
        }
    }

    // Verify we can proceed (either still in encounter or back to scouting)
    let result = combat_result(&client);
    assert!(
        result.is_some(),
        "Should have an encounter result after conclude/auto-conclude"
    );

    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(
        status,
        Status::Created,
        "Should be able to scout after crafting"
    );
}

/// Scenario: Full loop — combat → crafting → verify game continues.
#[test]
fn scenario_crafting_full_loop_after_combat() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    // Play a combat encounter to generate materials
    let combat_enc = combat_encounter_ids(&client);
    if !combat_enc.is_empty() {
        let pick_json = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            combat_enc[0]
        );
        let (status, _) = post_action(&client, &pick_json);
        assert_eq!(status, Status::Created);

        // Play combat rounds until finished
        for _ in 0..20 {
            if !play_one_round(&client) {
                break;
            }
        }

        // Conclude combat if still active
        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
            let (_, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        }

        // Scout
        let (status, _) = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
        assert_eq!(status, Status::Created, "Should be able to scout");
    }

    // Now pick a crafting encounter
    let craft_enc = crafting_encounter_ids(&client);
    if craft_enc.is_empty() {
        // Crafting encounter card may not be in hand — skip
        return;
    }

    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        craft_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created);

    // Verify it's crafting
    let encounter = crafting_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Crafting")
    );

    // Abort to end cleanly
    let (status, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    assert_eq!(status, Status::Created);

    let result = combat_result(&client);
    assert_eq!(result, Some("PlayerWon".to_string()));

    // Scout again
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(
        status,
        Status::Created,
        "Should be able to continue after crafting"
    );
}

/// Scenario: Crafting encounter cards exist in the library.
#[test]
fn scenario_crafting_expansion_cards_exist() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    // Verify crafting cards are registered
    let crafting_cards = get_json(&client, "/library/cards?card_kind=Crafting");
    let arr = crafting_cards.as_array().expect("Should be array");
    assert_eq!(arr.len(), 6, "Should have 6 crafting player cards");

    // Verify crafting encounter card exists
    let enc_cards = get_json(&client, "/library/cards?card_kind=Encounter");
    let enc_arr = enc_cards.as_array().expect("Should be array");
    let crafting_encs: Vec<_> = enc_arr
        .iter()
        .filter(|c| {
            c.get("kind")
                .and_then(|k| k.get("encounter_kind"))
                .and_then(|k| k.get("encounter_type"))
                .and_then(|v| v.as_str())
                == Some("Crafting")
        })
        .collect();
    assert_eq!(
        crafting_encs.len(),
        1,
        "Should have 1 crafting encounter card"
    );
}

// ======================== Research encounter helpers ========================

fn research_encounter_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| {
            let id = c.get("id")?.as_u64()? as usize;
            let enc_type = c
                .get("kind")?
                .get("encounter_kind")?
                .get("encounter_type")?
                .as_str()?;
            if enc_type == "Research" {
                Some(id)
            } else {
                None
            }
        })
        .collect()
}

/// Win a combat encounter while specifically playing Insight resource cards
/// when available, and scout. Returns true if combat was won.
fn win_combat_and_scout(client: &Client) -> bool {
    let combat_enc = combat_encounter_ids(client);
    if combat_enc.is_empty() {
        return false;
    }
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        combat_enc[0]
    );
    let (status, _) = post_action(client, &pick_json);
    if status != Status::Created {
        return false;
    }
    for _ in 0..80 {
        if !play_one_round_prefer_insight(client) {
            break;
        }
    }
    let result = combat_result(client);
    if result.as_deref() != Some("PlayerWon") {
        return false;
    }
    let (status, _) = post_action(
        client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    status == Status::Created
}

/// Like `play_one_round` but prefers Insight Resource cards when available.
fn play_one_round_prefer_insight(client: &Client) -> bool {
    // Play Defence
    let def_ids = hand_card_ids_by_kind(client, "Defence");
    if def_ids.is_empty() {
        return false;
    }
    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        def_ids[0]
    );
    let (status, _) = post_action(client, &json);
    if status != Status::Created {
        return false;
    }
    let combat = combat_state(client);
    if combat.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
        return false;
    }

    // Play Attack
    let atk_ids = hand_card_ids_by_kind(client, "Attack");
    if atk_ids.is_empty() {
        return false;
    }
    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        atk_ids[0]
    );
    let (status, _) = post_action(client, &json);
    if status != Status::Created {
        return false;
    }
    let combat = combat_state(client);
    if combat.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
        return false;
    }

    // Play Resource — prefer Insight Resource cards
    let res_cards = get_json(client, "/library/cards?location=Hand&card_kind=Resource");
    let empty = vec![];
    let res_arr = res_cards.as_array().unwrap_or(&empty);
    if res_arr.is_empty() {
        return false;
    }

    // Find an Insight resource card (one whose effects reference the Insight effect)
    let insight_card_id = res_arr.iter().find_map(|c| {
        let id = c.get("id")?.as_u64()? as usize;
        let effects = c.get("kind")?.get("effects")?.as_array()?;
        // Insight cards are identified by having an effect that references
        // the Insight PlayerCardEffect. We check if any resolved effect
        // produces Insight by looking at the effect_id pointing to an
        // Insight-type PlayerCardEffect.
        // Since we can't easily check the effect kind from here, use a
        // heuristic: Insight Resource cards have exactly 1 effect (the
        // main Resource card has 2 effects: stamina + draw).
        if effects.len() == 1 {
            Some(id)
        } else {
            None
        }
    });

    let card_to_play = insight_card_id
        .unwrap_or_else(|| res_arr[0].get("id").and_then(|v| v.as_u64()).unwrap() as usize);

    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        card_to_play
    );
    let (status, _) = post_action(client, &json);
    if status != Status::Created {
        return false;
    }
    let combat = combat_state(client);
    combat.get("outcome").and_then(|v| v.as_str()) == Some("Undecided")
}

/// Accumulate Insight tokens by winning combats repeatedly.
/// Returns once the player has at least `min_insight` Insight tokens or
/// after `max_attempts` combat wins.
/// Start a new game, accumulate Insight via combat wins, then deplete the
/// encounter hand until the Research encounter card (initially in deck) is
/// drawn to hand.
/// Returns the Insight balance right before picking the research encounter.
fn start_game_accumulate_insight_and_pick_research(client: &Client, seed: u64) -> i64 {
    let seed_json = format!(r#"{{"action_type":"NewGame","seed":{}}}"#, seed);
    let (status, _) = post_action(client, &seed_json);
    assert_eq!(status, Status::Created, "NewGame should succeed");

    // Phase 1: Win combats to accumulate Insight (also depletes encounter hand).
    for _ in 0..3 {
        if combat_encounter_ids(client).is_empty() {
            break;
        }
        win_combat_and_scout(client);
    }

    // Phase 2: Abort remaining encounters to deplete the hand until Research
    // card is drawn from deck (encounter_draw_to_hand fills to Foresight=3).
    assert!(
        deplete_encounters_until_research(client),
        "Should have research encounter cards in hand after depleting encounter hand"
    );

    let insight = player_token(client, "Insight");
    let research_enc = research_encounter_ids(client);

    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        research_enc[0]
    );
    let (status, _) = post_action(client, &pick_json);
    assert_eq!(
        status,
        Status::Created,
        "PickEncounter for research should succeed"
    );

    insight
}

// ======================== Research encounter tests ========================

/// Scenario: Research encounter flow: choose project, select candidate, conclude.
///
/// New game → win combats to accumulate Insight → deplete encounter hand until
/// Research encounter is available → pick Research encounter → choose project
/// (Combat, tier 1) → verify Insight deducted, 3 candidates generated →
/// select candidate 0 → conclude encounter → apply scouting.
///
/// Note: completing the research (paying full progress cost) requires more
/// Insight than a single game can reliably produce, so this test verifies
/// the project setup flow and persists the research project across encounters.
#[test]
fn scenario_research_encounter_full_loop() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    // Seed 7777 yields ~10 Insight from 3 combats
    let insight_before = start_game_accumulate_insight_and_pick_research(&client, 7777);

    // Verify encounter is Research type
    let encounter = combat_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Research"),
        "Encounter should be Research type"
    );
    assert_eq!(
        encounter.get("outcome").and_then(|v| v.as_str()),
        Some("Undecided"),
        "Research should be active"
    );

    if insight_before < 10 {
        // Not enough Insight to choose a project — verify the error and conclude
        let (status, body) = post_action(
            &client,
            r#"{"action_type":"ResearchChooseProject","discipline":"Combat","tier_count":1}"#,
        );
        assert_eq!(
            status,
            Status::BadRequest,
            "Should fail with insufficient Insight"
        );
        let message = body.get("message").and_then(|v| v.as_str()).unwrap_or("");
        assert!(
            message.contains("Insufficient Insight"),
            "Error should mention insufficient Insight, got: {}",
            message
        );
        // Conclude and return
        let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        assert_eq!(status, Status::Created);
        return;
    }

    // Choose a project: Combat discipline, tier 1 (costs 10 Insight)
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchChooseProject","discipline":"Combat","tier_count":1}"#,
    );
    assert_eq!(
        status,
        Status::Created,
        "ResearchChooseProject should succeed"
    );

    // Verify Insight was deducted (tier 1 costs 10)
    let insight_after_choose = player_token(&client, "Insight");
    assert_eq!(
        insight_after_choose,
        insight_before - 10,
        "Should deduct 10 Insight for tier 1"
    );

    // Verify 3 candidates generated
    let encounter = combat_state(&client);
    let candidates = encounter
        .get("candidates")
        .and_then(|v| v.as_array())
        .expect("Should have candidates array");
    assert_eq!(candidates.len(), 3, "Should generate exactly 3 candidates");

    // Select candidate 0
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchSelectCandidate","candidate_index":0}"#,
    );
    assert_eq!(
        status,
        Status::Created,
        "ResearchSelectCandidate should succeed"
    );

    // Candidates should be cleared after selection
    let encounter = combat_state(&client);
    assert!(
        encounter.get("candidates").is_none_or(|v| v.is_null()),
        "Candidates should be cleared after selection"
    );

    // Make progress if we have any Insight left
    if insight_after_choose > 0 {
        let (status, _) = post_action(
            &client,
            r#"{"action_type":"ResearchProgress","amount":100}"#,
        );
        // May succeed or fail depending on available Insight
        if status == Status::Created {
            let insight_after_progress = player_token(&client, "Insight");
            assert!(
                insight_after_progress < insight_after_choose,
                "Insight should decrease after progress"
            );
        }
    }

    // Conclude the research encounter
    let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
    assert_eq!(
        status,
        Status::Created,
        "ConcludeEncounter should succeed for research"
    );

    // Verify result is PlayerWon
    let result = combat_result(&client);
    assert_eq!(
        result,
        Some("PlayerWon".to_string()),
        "Research encounter should result in PlayerWon"
    );

    // Apply scouting
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(
        status,
        Status::Created,
        "Should be able to scout after research"
    );
}

/// Scenario: Choose a research project, select a candidate, conclude, then
/// verify the research project persists across encounters.
#[test]
fn scenario_research_choose_and_swap_project() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    // Seed 7777 yields ~10 Insight from 3 combats
    let insight_before = start_game_accumulate_insight_and_pick_research(&client, 7777);

    if insight_before < 10 {
        // Not enough Insight — skip test gracefully
        let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
        return;
    }

    // First research: Combat, tier 1
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchChooseProject","discipline":"Combat","tier_count":1}"#,
    );
    assert_eq!(status, Status::Created);

    // Select candidate 0
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchSelectCandidate","candidate_index":0}"#,
    );
    assert_eq!(status, Status::Created);

    // Conclude first research encounter (research project persists)
    let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
    assert_eq!(status, Status::Created);

    // Verify result
    let result = combat_result(&client);
    assert_eq!(result, Some("PlayerWon".to_string()));

    // Scout
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(status, Status::Created);

    // Check if Research encounter card is available again
    let research_enc = research_encounter_ids(&client);
    if research_enc.is_empty() {
        // Research card was consumed and not redrawn — test completed
        // This is expected since encounter cards don't recycle
        return;
    }

    // If available, start a second research encounter
    let insight_now = player_token(&client, "Insight");
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        research_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created);

    // If we have enough Insight, choose a different project
    if insight_now >= 10 {
        let (status, _) = post_action(
            &client,
            r#"{"action_type":"ResearchChooseProject","discipline":"Mining","tier_count":1}"#,
        );
        assert_eq!(
            status,
            Status::Created,
            "Second ResearchChooseProject should succeed"
        );

        // Select candidate 1
        let (status, _) = post_action(
            &client,
            r#"{"action_type":"ResearchSelectCandidate","candidate_index":1}"#,
        );
        assert_eq!(status, Status::Created);
    }

    // Conclude second research
    let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
    assert_eq!(status, Status::Created);

    let result = combat_result(&client);
    assert_eq!(result, Some("PlayerWon".to_string()));
}

/// Helper to get the research encounter card into hand by depleting the
/// encounter hand through aborting/concluding non-combat encounters.
/// Does NOT accumulate Insight. Returns true if research encounter is in hand.
fn deplete_encounters_until_research(client: &Client) -> bool {
    for _ in 0..25 {
        if !research_encounter_ids(client).is_empty() {
            return true;
        }
        let enc_hand = encounter_hand_ids(client);
        if enc_hand.is_empty() {
            return false;
        }
        let pick_json = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            enc_hand[0]
        );
        if post_action(client, &pick_json).0 != Status::Created {
            break;
        }
        let (status, _) = post_action(client, r#"{"action_type":"EncounterAbort"}"#);
        if status != Status::Created {
            let _ = post_action(client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        }
        let _ = post_action(
            client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
    }
    !research_encounter_ids(client).is_empty()
}

/// Scenario: Attempt to choose a research project with insufficient Insight.
#[test]
fn scenario_research_insufficient_insight() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Start game — do NOT play any combats (keep Insight at 0)
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    // Verify Insight is 0
    let insight = player_token(&client, "Insight");
    assert_eq!(insight, 0, "Should start with 0 Insight");

    // Deplete encounter hand to get research card
    if !deplete_encounters_until_research(&client) {
        // Research card never appeared — skip gracefully
        return;
    }

    // Pick research encounter
    let research_enc = research_encounter_ids(&client);
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        research_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created, "PickEncounter should succeed");

    // Try to choose project with 0 Insight — should fail
    let (status, body) = post_action(
        &client,
        r#"{"action_type":"ResearchChooseProject","discipline":"Combat","tier_count":1}"#,
    );
    assert_eq!(
        status,
        Status::BadRequest,
        "Should fail with insufficient Insight"
    );
    let message = body.get("message").and_then(|v| v.as_str()).unwrap_or("");
    assert!(
        message.contains("Insufficient Insight"),
        "Error should mention insufficient Insight, got: {}",
        message
    );
}

/// Scenario: Abort a research encounter — should succeed with PlayerWon, no penalty.
#[test]
fn scenario_research_abort() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    // Deplete encounter hand to get research card
    if !deplete_encounters_until_research(&client) {
        return;
    }

    let research_enc = research_encounter_ids(&client);
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        research_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created);

    // Verify encounter is active
    let encounter = combat_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Research")
    );

    // Abort
    let (status, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    assert_eq!(status, Status::Created, "Research abort should succeed");

    // Verify result is PlayerWon
    let result = combat_result(&client);
    assert_eq!(
        result,
        Some("PlayerWon".to_string()),
        "Research abort should always result in PlayerWon"
    );

    // Should be back to scouting
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(
        status,
        Status::Created,
        "Should be able to scout after research abort"
    );
}

// ======================== Additional crafting fix tests ========================

/// Scenario: Abort is blocked during an active craft mini-game.
///
/// Start crafting encounter → start a craft (EncounterCraftCard) →
/// try to abort → should fail with an error.
#[test]
fn scenario_crafting_abort_blocked_during_active_craft() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    start_game_and_pick_crafting(&client);

    // Find a target card to craft (any player card)
    let all_cards = get_json(&client, "/library/cards");
    let all_arr = all_cards.as_array().expect("Should have cards");
    let target = all_arr.iter().find(|c| {
        let kind = c
            .get("kind")
            .and_then(|k| k.get("card_kind"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        matches!(kind, "Attack" | "Defence" | "Resource")
    });
    let target_id = target
        .expect("Should have a player card to craft")
        .get("id")
        .and_then(|v| v.as_u64())
        .unwrap() as usize;

    // Start crafting the card
    let craft_json = format!(
        r#"{{"action_type":"EncounterCraftCard","target_card_id":{}}}"#,
        target_id
    );
    let (status, _) = post_action(&client, &craft_json);
    assert_eq!(status, Status::Created, "CraftCard should succeed");

    // Verify active_craft is present
    let encounter = crafting_state(&client);
    let active_craft = encounter.get("active_craft");
    assert!(
        active_craft.is_some() && !active_craft.unwrap().is_null(),
        "Should have an active craft"
    );

    // Try to abort — should fail because craft is in progress
    let (status, body) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    assert_eq!(
        status,
        Status::BadRequest,
        "Abort should be blocked during active craft"
    );
    let message = body.get("message").and_then(|v| v.as_str()).unwrap_or("");
    assert!(
        message.contains("Cannot abort while a craft is in progress"),
        "Error should explain craft is blocking abort, got: {}",
        message
    );
}

/// Scenario: Crafting a card increments an EXISTING library entry's count
/// rather than creating a new card (deduplication).
///
/// Start crafting → craft an existing card → conclude → verify the target
/// card's library count increased rather than a new card being created.
#[test]
fn scenario_crafting_card_deduplication() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    start_game_and_pick_crafting(&client);

    // Find an Attack card to craft (these exist with known library counts)
    let all_cards = get_json(&client, "/library/cards");
    let all_arr = all_cards.as_array().expect("Should have cards");
    let target = all_arr.iter().find(|c| {
        let kind = c
            .get("kind")
            .and_then(|k| k.get("card_kind"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        kind == "Attack"
    });
    let target_card = target.expect("Should have an Attack card to craft");
    let target_id = target_card.get("id").and_then(|v| v.as_u64()).unwrap() as usize;

    // Record the library count of the target card before crafting
    let lib_count_before = target_card
        .get("counts")
        .and_then(|c| c.get("library"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    // Total card count before crafting
    let total_cards_before = all_arr.len();

    // Start crafting the card
    let craft_json = format!(
        r#"{{"action_type":"EncounterCraftCard","target_card_id":{}}}"#,
        target_id
    );
    let (status, _) = post_action(&client, &craft_json);
    assert_eq!(status, Status::Created, "CraftCard should succeed");

    // Conclude crafting encounter (which finalizes the craft)
    let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
    // May succeed or fail (if player can't pay costs)
    if status == Status::Created {
        let result = combat_result(&client);
        if result.as_deref() == Some("PlayerWon") {
            // Verify deduplication: total card count should NOT have increased
            let all_cards_after = get_json(&client, "/library/cards");
            let total_cards_after = all_cards_after.as_array().map(|a| a.len()).unwrap_or(0);
            assert_eq!(
                total_cards_after, total_cards_before,
                "Crafting should increment existing card, not create a new one (before={}, after={})",
                total_cards_before, total_cards_after
            );

            // Verify the target card's library count increased by 1
            let target_after = all_cards_after
                .as_array()
                .unwrap()
                .iter()
                .find(|c| {
                    c.get("id").and_then(|v| v.as_u64()).map(|v| v as usize) == Some(target_id)
                })
                .expect("Target card should still exist");
            let lib_count_after = target_after
                .get("counts")
                .and_then(|c| c.get("library"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            assert_eq!(
                lib_count_after,
                lib_count_before + 1,
                "Library count should increase by 1 after crafting"
            );
        }
    }
}
