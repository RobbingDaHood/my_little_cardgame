use super::helpers::*;
use my_little_cardgame::rocket_initialize;
use rocket::http::Status;
use rocket::local::blocking::Client;
use rocket::serde::json::serde_json;

const TOKENS: &str = include_str!("../configurations/tokens_default.json");
const TOKENS_LOW_DUR: &str = include_str!("../configurations/tokens_low_durability.json");
const CRAFTING_WIN: &str = include_str!("../configurations/crafting_win.json");
const CRAFTING_LOSS: &str = include_str!("../configurations/crafting_loss.json");

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

/// Find crafting card IDs available in the player's hand.
fn crafting_hand_card_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Crafting");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| c.get("id").and_then(|v| v.as_u64()).map(|v| v as usize))
        .collect()
}

/// Play one crafting card. Starts a craft if none is active.
/// Returns true if the crafting encounter is still active.
fn play_one_crafting_card(client: &Client) -> bool {
    let enc = combat_state(client);
    let has_active_craft = enc
        .get("active_craft")
        .map(|v| !v.is_null())
        .unwrap_or(false);

    if !has_active_craft {
        let cards = get_json(client, "/library/cards?card_kind=Attack");
        let target_id = cards
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|c| c.get("id"))
            .and_then(|v| v.as_u64());
        if let Some(id) = target_id {
            let craft_json = format!(
                r#"{{"action_type":"EncounterCraftCard","target_card_id":{}}}"#,
                id
            );
            let (status, _) = post_action(client, &craft_json);
            if status != Status::Created {
                return false;
            }
        } else {
            return false;
        }
        let enc = combat_state(client);
        if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
            return false;
        }
    }

    let crafting_ids = crafting_hand_card_ids(client);
    if crafting_ids.is_empty() {
        return false;
    }
    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        crafting_ids[0]
    );
    let (status, _) = post_action(client, &json);
    if status != Status::Created {
        return false;
    }
    let encounter = combat_state(client);
    encounter.get("outcome").and_then(|v| v.as_str()) == Some("Undecided")
}

#[test]
fn scenario_crafting_win_and_scout() {
    let client = create_test_client_from_json(42, TOKENS, &[("crafting", CRAFTING_WIN)]);

    let enc_ids = crafting_encounter_ids(&client);
    assert!(!enc_ids.is_empty(), "Should have crafting encounters");
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc_ids[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    let encounter = combat_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Crafting")
    );

    for _ in 0..50 {
        if !play_one_crafting_card(&client) {
            break;
        }
    }

    let enc = combat_state(&client);
    if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
        let _ = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
    }

    let result = combat_result(&client);
    assert_eq!(
        result.as_deref(),
        Some("PlayerWon"),
        "Crafting should win with enough tokens"
    );

    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(status, Status::Created);

    let enc_after = encounter_hand_ids(&client);
    assert!(!enc_after.is_empty());
    let pick2 = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc_after[0]
    );
    let (status, _) = post_action(&client, &pick2);
    assert_eq!(status, Status::Created);
}

#[test]
fn scenario_crafting_loss_and_scout() {
    let client = create_test_client_from_json(42, TOKENS_LOW_DUR, &[("crafting", CRAFTING_LOSS)]);

    let enc_ids = crafting_encounter_ids(&client);
    assert!(!enc_ids.is_empty());
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc_ids[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    for _ in 0..50 {
        if !play_one_crafting_card(&client) {
            break;
        }
    }

    let enc = combat_state(&client);
    if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
        let _ = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
    }

    let result = combat_result(&client);
    assert_eq!(
        result.as_deref(),
        Some("PlayerLost"),
        "Crafting should lose when tokens exhausted"
    );

    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(status, Status::Created);

    let enc_after = encounter_hand_ids(&client);
    assert!(!enc_after.is_empty());
    let pick2 = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc_after[0]
    );
    let (status, _) = post_action(&client, &pick2);
    assert_eq!(status, Status::Created);
}

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

/// Scenario: Crafting encounter -> swap cards between deck and library.
#[test]
fn scenario_crafting_swap_cards() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    start_game_and_pick_crafting(&client);

    // Find a player card in deck and a player card in library
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

/// Scenario: Abort a crafting encounter -> verify PlayerWon, no penalty.
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

/// Scenario: Start a craft mini-game -> play crafting cards -> conclude.
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

/// Scenario: Full loop -- combat -> crafting -> verify game continues.
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
    assert_eq!(arr.len(), 8, "Should have 8 crafting player cards");

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

/// Scenario: Abort is blocked during an active craft mini-game.
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
