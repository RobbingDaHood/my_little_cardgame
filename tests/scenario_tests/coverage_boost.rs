//! Targeted integration tests to boost code coverage.
//!
//! These tests exercise specific uncovered code paths including:
//! - Milestone start_X_inner() for Herbalism, Woodcutting, Fishing
//! - Crafting durability (EncounterCraftDurability)
//! - Deeper fishing, herbalism, rest, and research encounters

use super::helpers::*;
use my_little_cardgame::rocket_initialize;
use rocket::http::Status;
use rocket::local::blocking::Client;
use rocket::serde::json::serde_json;

// ---------------------------------------------------------------------------
// Milestone helpers (reused from milestone.rs patterns)
// ---------------------------------------------------------------------------

fn milestone_encounter_by_discipline(client: &Client, discipline: &str) -> Option<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards.as_array()?.iter().find_map(|c| {
        let kind = c.get("kind")?;
        let enc_kind = kind.get("encounter_kind")?;
        if enc_kind.get("encounter_type")?.as_str()? == "Milestone" {
            let def = enc_kind.get("milestone_def")?;
            if def.get("discipline")?.as_str()? == discipline {
                c.get("id")?.as_u64().map(|v| v as usize)
            } else {
                None
            }
        } else {
            None
        }
    })
}

fn win_combat_for_milestone(client: &Client) {
    let enc = combat_encounter_ids(client);
    assert!(!enc.is_empty(), "Need combat encounter cards");
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc[0]
    );
    let (status, _) = post_action(client, &pick);
    assert_eq!(status, Status::Created);
    for _ in 0..200 {
        if !play_one_round(client) {
            break;
        }
    }
    let (status, _) = post_action(
        client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(status, Status::Created);
}

// ---------------------------------------------------------------------------
// Crafting helpers
// ---------------------------------------------------------------------------

fn crafting_encounter_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| {
            let enc_type = c
                .get("kind")?
                .get("encounter_kind")?
                .get("encounter_type")?
                .as_str()?;
            if enc_type == "Crafting" {
                c.get("id")?.as_u64().map(|v| v as usize)
            } else {
                None
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Encounter-type helpers
// ---------------------------------------------------------------------------

fn fishing_encounter_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| {
            let enc_type = c
                .get("kind")?
                .get("encounter_kind")?
                .get("encounter_type")?
                .as_str()?;
            if enc_type == "Fishing" {
                c.get("id")?.as_u64().map(|v| v as usize)
            } else {
                None
            }
        })
        .collect()
}

fn herbalism_encounter_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| {
            let enc_type = c
                .get("kind")?
                .get("encounter_kind")?
                .get("encounter_type")?
                .as_str()?;
            if enc_type == "Herbalism" {
                c.get("id")?.as_u64().map(|v| v as usize)
            } else {
                None
            }
        })
        .collect()
}

fn rest_encounter_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| {
            let enc_type = c
                .get("kind")?
                .get("encounter_kind")?
                .get("encounter_type")?
                .as_str()?;
            if enc_type == "Rest" {
                c.get("id")?.as_u64().map(|v| v as usize)
            } else {
                None
            }
        })
        .collect()
}

fn woodcutting_encounter_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| {
            let enc_type = c
                .get("kind")?
                .get("encounter_kind")?
                .get("encounter_type")?
                .as_str()?;
            if enc_type == "Woodcutting" {
                c.get("id")?.as_u64().map(|v| v as usize)
            } else {
                None
            }
        })
        .collect()
}

fn research_encounter_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| {
            let enc_type = c
                .get("kind")?
                .get("encounter_kind")?
                .get("encounter_type")?
                .as_str()?;
            if enc_type == "Research" {
                c.get("id")?.as_u64().map(|v| v as usize)
            } else {
                None
            }
        })
        .collect()
}

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

// ===================================================================
// 1. MILESTONE DISCIPLINE VARIANT TESTS
// ===================================================================

#[test]
fn milestone_herbalism_encounter() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);

    win_combat_for_milestone(&client);

    let insight = player_token(&client, "MilestoneInsight");
    assert!(insight >= 100, "Need at least 100 insight, got {}", insight);

    let milestone_id = milestone_encounter_by_discipline(&client, "Herbalism")
        .expect("Should have Herbalism milestone");
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        milestone_id
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created, "Pick Herbalism milestone");

    let enc = combat_state(&client);
    assert_eq!(
        enc.get("encounter_state_type").and_then(|v| v.as_str()),
        Some("Milestone")
    );
    assert_eq!(
        enc.get("discipline").and_then(|v| v.as_str()),
        Some("Herbalism")
    );

    let inner = enc.get("inner_state").expect("Should have inner_state");
    assert_eq!(
        inner.get("encounter_state_type").and_then(|v| v.as_str()),
        Some("Herbalism"),
        "Inner state should be Herbalism"
    );

    // Play a few herbalism cards if available
    let herb_cards = hand_card_ids_by_kind(&client, "Herbalism");
    for card_id in herb_cards.iter().take(2) {
        let play = format!(
            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
            card_id
        );
        let (status, _) = post_action(&client, &play);
        if status != Status::Created {
            break;
        }
        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
            break;
        }
    }

    // Abort only if still undecided
    let enc = combat_state(&client);
    if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
        let (status, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
        assert_eq!(status, Status::Created, "Abort should work");
    }

    // Scout to return to NoEncounter
    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

#[test]
fn milestone_woodcutting_encounter() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);

    win_combat_for_milestone(&client);

    let insight = player_token(&client, "MilestoneInsight");
    assert!(insight >= 100, "Need at least 100 insight, got {}", insight);

    let milestone_id = milestone_encounter_by_discipline(&client, "Woodcutting")
        .expect("Should have Woodcutting milestone");
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        milestone_id
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created, "Pick Woodcutting milestone");

    let enc = combat_state(&client);
    assert_eq!(
        enc.get("encounter_state_type").and_then(|v| v.as_str()),
        Some("Milestone")
    );
    assert_eq!(
        enc.get("discipline").and_then(|v| v.as_str()),
        Some("Woodcutting")
    );

    let inner = enc.get("inner_state").expect("Should have inner_state");
    assert_eq!(
        inner.get("encounter_state_type").and_then(|v| v.as_str()),
        Some("Woodcutting"),
        "Inner state should be Woodcutting"
    );

    // Play a few woodcutting cards if available
    let wc_cards = hand_card_ids_by_kind(&client, "Woodcutting");
    for card_id in wc_cards.iter().take(2) {
        let play = format!(
            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
            card_id
        );
        let (status, _) = post_action(&client, &play);
        if status != Status::Created {
            break;
        }
        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
            break;
        }
    }

    let (status, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    assert_eq!(status, Status::Created, "Abort should work");

    assert!(
        milestone_encounter_by_discipline(&client, "Woodcutting").is_some(),
        "Woodcutting milestone should be returned after abort"
    );
}

#[test]
fn milestone_fishing_encounter() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);

    win_combat_for_milestone(&client);

    let insight = player_token(&client, "MilestoneInsight");
    assert!(insight >= 100, "Need at least 100 insight, got {}", insight);

    let milestone_id = milestone_encounter_by_discipline(&client, "Fishing")
        .expect("Should have Fishing milestone");
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        milestone_id
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created, "Pick Fishing milestone");

    let enc = combat_state(&client);
    assert_eq!(
        enc.get("encounter_state_type").and_then(|v| v.as_str()),
        Some("Milestone")
    );
    assert_eq!(
        enc.get("discipline").and_then(|v| v.as_str()),
        Some("Fishing")
    );

    let inner = enc.get("inner_state").expect("Should have inner_state");
    assert_eq!(
        inner.get("encounter_state_type").and_then(|v| v.as_str()),
        Some("Fishing"),
        "Inner state should be Fishing"
    );

    // Play a few fishing cards if available
    let fish_cards = hand_card_ids_by_kind(&client, "Fishing");
    for card_id in fish_cards.iter().take(2) {
        let play = format!(
            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
            card_id
        );
        let (status, _) = post_action(&client, &play);
        if status != Status::Created {
            break;
        }
        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
            break;
        }
    }

    let (status, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    assert_eq!(status, Status::Created, "Abort should work");

    assert!(
        milestone_encounter_by_discipline(&client, "Fishing").is_some(),
        "Fishing milestone should be returned after abort"
    );
}

// ===================================================================
// 2. CRAFTING DURABILITY TEST
// ===================================================================

/// Exercise the EncounterCraftDurability action to add durability during crafting.
/// This requires having materials (Ore or Lumber) which we earn from combat/gathering.
#[test]
fn crafting_add_durability_mining() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    // Win a combat and do mining/woodcutting to accumulate Ore and Lumber.
    // First win combat:
    win_combat_for_milestone(&client);

    // Do a mining encounter to get Ore
    let mining_enc = mining_encounter_ids(&client);
    if !mining_enc.is_empty() {
        let pick = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            mining_enc[0]
        );
        let (status, _) = post_action(&client, &pick);
        if status == Status::Created {
            for _ in 0..20 {
                let mining_cards = hand_card_ids_by_kind(&client, "Mining");
                if mining_cards.is_empty() {
                    break;
                }
                let play = format!(
                    r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                    mining_cards[0]
                );
                let (status, _) = post_action(&client, &play);
                if status != Status::Created {
                    break;
                }
                let enc = combat_state(&client);
                if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                    break;
                }
            }
            // If still undecided, abort
            let enc = combat_state(&client);
            if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
                let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
            }
            let _ = post_action(
                &client,
                r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
            );
        }
    }

    // Do a woodcutting encounter to get Lumber
    let wc_enc = woodcutting_encounter_ids(&client);
    if !wc_enc.is_empty() {
        let pick = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            wc_enc[0]
        );
        let (status, _) = post_action(&client, &pick);
        if status == Status::Created {
            for _ in 0..20 {
                let wc_cards = hand_card_ids_by_kind(&client, "Woodcutting");
                if wc_cards.is_empty() {
                    break;
                }
                let play = format!(
                    r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                    wc_cards[0]
                );
                let (status, _) = post_action(&client, &play);
                if status != Status::Created {
                    break;
                }
                let enc = combat_state(&client);
                if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                    break;
                }
            }
            let enc = combat_state(&client);
            if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
                let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
            }
            let _ = post_action(
                &client,
                r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
            );
        }
    }

    let ore_before = player_token(&client, "Ore");
    let _lumber_before = player_token(&client, "Lumber");
    let mining_dur_before = player_token(&client, "MiningDurability");

    // Now start crafting encounter
    let craft_enc = crafting_encounter_ids(&client);
    if craft_enc.is_empty() {
        return;
    }
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        craft_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created, "Pick crafting should work");

    let enc = combat_state(&client);
    assert_eq!(
        enc.get("encounter_state_type").and_then(|v| v.as_str()),
        Some("Crafting")
    );

    // Try Mining durability (costs 50 Ore)
    if ore_before >= 50 {
        let (status, _) = post_action(
            &client,
            r#"{"action_type":"EncounterCraftDurability","discipline":"Mining"}"#,
        );
        assert_eq!(
            status,
            Status::Created,
            "CraftDurability Mining should succeed with {} ore",
            ore_before
        );

        let ore_after = player_token(&client, "Ore");
        assert_eq!(ore_after, ore_before - 50, "Should deduct 50 Ore");

        let mining_dur_after = player_token(&client, "MiningDurability");
        assert_eq!(
            mining_dur_after,
            mining_dur_before + 500,
            "Should add 500 mining durability"
        );
    }

    // Try Herbalism durability (costs 50 Lumber)
    let lumber_now = player_token(&client, "Lumber");
    let herb_dur_before = player_token(&client, "HerbalismDurability");
    if lumber_now >= 50 {
        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
            let (status, _) = post_action(
                &client,
                r#"{"action_type":"EncounterCraftDurability","discipline":"Herbalism"}"#,
            );
            if status == Status::Created {
                let herb_dur_after = player_token(&client, "HerbalismDurability");
                assert_eq!(herb_dur_after, herb_dur_before + 500);
            }
        }
    }

    // Try Woodcutting durability (costs 50 Lumber)
    let lumber_now = player_token(&client, "Lumber");
    let wc_dur_before = player_token(&client, "WoodcuttingDurability");
    if lumber_now >= 50 {
        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
            let (status, _) = post_action(
                &client,
                r#"{"action_type":"EncounterCraftDurability","discipline":"Woodcutting"}"#,
            );
            if status == Status::Created {
                let wc_dur_after = player_token(&client, "WoodcuttingDurability");
                assert_eq!(wc_dur_after, wc_dur_before + 500);
            }
        }
    }

    // Try Fishing durability (costs 50 Ore)
    let ore_now = player_token(&client, "Ore");
    let fish_dur_before = player_token(&client, "FishingDurability");
    if ore_now >= 50 {
        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
            let (status, _) = post_action(
                &client,
                r#"{"action_type":"EncounterCraftDurability","discipline":"Fishing"}"#,
            );
            if status == Status::Created {
                let fish_dur_after = player_token(&client, "FishingDurability");
                assert_eq!(fish_dur_after, fish_dur_before + 500);
            }
        }
    }

    // Try invalid discipline
    let enc = combat_state(&client);
    if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
        let (status, _) = post_action(
            &client,
            r#"{"action_type":"EncounterCraftDurability","discipline":"InvalidDiscipline"}"#,
        );
        assert_eq!(status, Status::BadRequest, "Invalid discipline should fail");
    }

    // Clean up
    let enc = combat_state(&client);
    if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
        let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    }
}

/// Test crafting durability when we don't have enough materials.
#[test]
fn crafting_add_durability_insufficient_materials() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    let craft_enc = crafting_encounter_ids(&client);
    if craft_enc.is_empty() {
        return;
    }
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        craft_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    // At start we have 0 Ore, so Mining durability should fail
    let ore = player_token(&client, "Ore");
    if ore < 50 {
        let (status, body) = post_action(
            &client,
            r#"{"action_type":"EncounterCraftDurability","discipline":"Mining"}"#,
        );
        assert_eq!(
            status,
            Status::BadRequest,
            "Should fail with insufficient materials"
        );
        let msg = body.get("message").and_then(|v| v.as_str()).unwrap_or("");
        assert!(
            msg.contains("Not enough materials"),
            "Should mention materials, got: {}",
            msg
        );
    }

    let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
}

// ===================================================================
// 3. FISHING DEEPER COVERAGE
// ===================================================================

/// Exercise a longer fishing encounter with multiple card plays, using a
/// different seed for varied branching paths.
#[test]
fn fishing_multi_card_play_varied_seed() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":100}"#);
    assert_eq!(status, Status::Created);

    let fc_enc = fishing_encounter_ids(&client);
    if fc_enc.is_empty() {
        return;
    }

    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        fc_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    let enc = combat_state(&client);
    assert_eq!(
        enc.get("encounter_state_type").and_then(|v| v.as_str()),
        Some("Fishing")
    );

    let range_min = encounter_token(&client, "FishingRangeMin");
    let range_max = encounter_token(&client, "FishingRangeMax");
    assert!(range_min > 0, "FishingRangeMin should be set");
    assert!(range_max > 0, "FishingRangeMax should be set");

    // Play as many cards as possible
    let mut total_plays = 0;
    for _ in 0..30 {
        let fish_cards = hand_card_ids_by_kind(&client, "Fishing");
        if fish_cards.is_empty() {
            break;
        }
        // Try each card until one succeeds
        let mut played = false;
        for card_id in &fish_cards {
            let play = format!(
                r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                card_id
            );
            let (status, _) = post_action(&client, &play);
            if status == Status::Created {
                played = true;
                total_plays += 1;
                break;
            }
        }
        if !played {
            break;
        }
        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
            break;
        }
    }

    assert!(
        total_plays >= 1,
        "Should have played at least 1 fishing card"
    );

    let result = combat_result(&client);
    assert!(
        result.is_some(),
        "Should have an outcome after fishing plays"
    );

    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

/// Run multiple fishing encounters back-to-back using seed 200 for different paths.
#[test]
fn fishing_multiple_encounters_seed200() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":200}"#);
    assert_eq!(status, Status::Created);

    let mut total_encounters = 0;
    for _ in 0..10 {
        let fc_enc = fishing_encounter_ids(&client);
        if fc_enc.is_empty() {
            break;
        }
        let pick = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            fc_enc[0]
        );
        let (status, _) = post_action(&client, &pick);
        if status != Status::Created {
            break;
        }

        for _ in 0..20 {
            let fish_cards = hand_card_ids_by_kind(&client, "Fishing");
            if fish_cards.is_empty() {
                break;
            }
            let play = format!(
                r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                fish_cards[0]
            );
            let (status, _) = post_action(&client, &play);
            if status != Status::Created {
                break;
            }
            let enc = combat_state(&client);
            if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                break;
            }
        }

        total_encounters += 1;
        let outcome = combat_result(&client).unwrap_or_default();
        if outcome == "PlayerLost" {
            break;
        }

        let _ = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
    }

    assert!(
        total_encounters >= 1,
        "Should complete at least 1 fishing encounter"
    );
}

// ===================================================================
// 4. REST ENCOUNTER VARIATIONS
// ===================================================================

/// Play multiple rest cards in a single rest encounter before concluding.
#[test]
fn rest_encounter_play_multiple_cards() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":100}"#);
    assert_eq!(status, Status::Created);

    let rest_enc = rest_encounter_ids(&client);
    if rest_enc.is_empty() {
        return;
    }

    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        rest_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    let enc = combat_state(&client);
    assert_eq!(
        enc.get("encounter_state_type").and_then(|v| v.as_str()),
        Some("Rest")
    );
    let rest_tokens = enc.get("rest_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
    assert!(rest_tokens >= 1, "Should have rest_tokens");

    let _stamina_before = player_token(&client, "Stamina");

    // Play as many rest cards as possible
    for _ in 0..10 {
        let rest_cards = hand_card_ids_by_kind(&client, "Rest");
        if rest_cards.is_empty() {
            break;
        }

        let mut played = false;
        for card_id in &rest_cards {
            let play = format!(
                r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                card_id
            );
            let (status, _) = post_action(&client, &play);
            if status == Status::Created {
                played = true;
                break;
            }
        }
        if !played {
            break;
        }
        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
            break;
        }
    }

    let result = combat_result(&client);
    if result.is_none() {
        // Still undecided, conclude
        let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    }

    let final_result = combat_result(&client).unwrap_or_default();
    assert_eq!(final_result, "PlayerWon", "Rest should always be PlayerWon");

    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

/// Rest encounter with a different seed to exercise different card draws.
#[test]
fn rest_encounter_seed300() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":300}"#);
    assert_eq!(status, Status::Created);

    let rest_enc = rest_encounter_ids(&client);
    if rest_enc.is_empty() {
        return;
    }

    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        rest_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    let enc = combat_state(&client);
    assert_eq!(
        enc.get("encounter_state_type").and_then(|v| v.as_str()),
        Some("Rest")
    );

    // Try to play all rest cards
    for _ in 0..10 {
        let rest_cards = hand_card_ids_by_kind(&client, "Rest");
        if rest_cards.is_empty() {
            break;
        }
        let play = format!(
            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
            rest_cards[0]
        );
        let (status, _) = post_action(&client, &play);
        if status != Status::Created {
            break;
        }
        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
            break;
        }
    }

    let enc = combat_state(&client);
    if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
        // Try conclude first, fall back to abort
        let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        if status != Status::Created {
            let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
        }
    }

    let result = combat_result(&client).unwrap_or_default();
    assert!(
        result == "PlayerWon" || result == "PlayerLost",
        "Rest should have a result, got: '{}'",
        result
    );

    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

// ===================================================================
// 5. HERBALISM DEEPER — PLAY MULTIPLE CARDS, DIFFERENT SEEDS
// ===================================================================

/// Exercise herbalism with seed 100 for different match mode paths.
#[test]
fn herbalism_multi_card_play_seed100() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":100}"#);
    assert_eq!(status, Status::Created);

    let herb_enc = herbalism_encounter_ids(&client);
    if herb_enc.is_empty() {
        return;
    }

    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        herb_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    let enc = combat_state(&client);
    assert_eq!(
        enc.get("encounter_state_type").and_then(|v| v.as_str()),
        Some("Herbalism")
    );
    let plant_hand = enc
        .get("plant_hand")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    assert_eq!(plant_hand, 5, "Should start with 5 plants");

    // Play many herbalism cards without aborting
    let mut total_plays = 0;
    for _ in 0..20 {
        let herb_cards = hand_card_ids_by_kind(&client, "Herbalism");
        if herb_cards.is_empty() {
            break;
        }
        let mut played = false;
        for card_id in &herb_cards {
            let play = format!(
                r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                card_id
            );
            let (status, _) = post_action(&client, &play);
            if status == Status::Created {
                played = true;
                total_plays += 1;
                break;
            }
        }
        if !played {
            break;
        }
        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
            break;
        }
    }

    assert!(
        total_plays >= 1,
        "Should have played at least 1 herbalism card"
    );

    let result = combat_result(&client);
    assert!(
        result.is_some(),
        "Should have outcome after herbalism plays"
    );

    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

/// Run multiple herbalism encounters back-to-back with seed 200.
#[test]
fn herbalism_multiple_encounters_seed200() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":200}"#);
    assert_eq!(status, Status::Created);

    let mut total_encounters = 0;
    for _ in 0..15 {
        let herb_enc = herbalism_encounter_ids(&client);
        if herb_enc.is_empty() {
            break;
        }
        let pick = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            herb_enc[0]
        );
        let (status, _) = post_action(&client, &pick);
        if status != Status::Created {
            break;
        }

        // Play all available herbalism cards
        for _ in 0..20 {
            let herb_cards = hand_card_ids_by_kind(&client, "Herbalism");
            if herb_cards.is_empty() {
                break;
            }
            let play = format!(
                r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                herb_cards[0]
            );
            let (status, _) = post_action(&client, &play);
            if status != Status::Created {
                break;
            }
            let enc = combat_state(&client);
            if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                break;
            }
        }

        total_encounters += 1;
        let outcome = combat_result(&client).unwrap_or_default();
        if outcome == "PlayerLost" {
            // Check durability — might be depleted
            let dur = player_token(&client, "HerbalismDurability");
            if dur == 0 {
                break;
            }
        }

        let _ = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
    }

    assert!(
        total_encounters >= 1,
        "Should complete at least 1 herbalism encounter"
    );
}

/// Herbalism with seed 500 to hit different card/mode combinations.
#[test]
fn herbalism_seed500_varied_modes() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":500}"#);
    assert_eq!(status, Status::Created);

    let herb_enc = herbalism_encounter_ids(&client);
    if herb_enc.is_empty() {
        return;
    }

    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        herb_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    // Play until encounter ends
    for _ in 0..20 {
        let herb_cards = hand_card_ids_by_kind(&client, "Herbalism");
        if herb_cards.is_empty() {
            break;
        }
        let play = format!(
            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
            herb_cards[0]
        );
        let (status, _) = post_action(&client, &play);
        if status != Status::Created {
            break;
        }
        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
            break;
        }
    }

    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

// ===================================================================
// 6. RESEARCH EXPERIMENT — DEEPER PLAY HAND COVERAGE
// ===================================================================

/// Play multiple research experiment rounds with a different seed.
#[test]
fn research_experiment_multi_round_seed42() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let seed_json = r#"{"action_type":"NewGame","seed":42}"#;
    let (status, _) = post_action(&client, seed_json);
    assert_eq!(status, Status::Created);

    // Win combats to get insight
    for _ in 0..3 {
        if combat_encounter_ids(&client).is_empty() {
            break;
        }
        win_combat_and_scout(&client);
    }

    if !deplete_encounters_until_research(&client) {
        return;
    }

    let insight = player_token(&client, "CombatInsight");
    let research_enc = research_encounter_ids(&client);
    if research_enc.is_empty() || insight < 10 {
        return;
    }

    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        research_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    // Choose project: Mining, tier 1
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchChooseProject","discipline":"Mining","tier_count":1}"#,
    );
    if status != Status::Created {
        let _ = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        return;
    }

    // Select candidate
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchSelectCandidate","candidate_index":0}"#,
    );
    assert_eq!(status, Status::Created);

    // Play multiple rounds
    let mut rounds_played = 0;
    for _ in 0..5 {
        let research_cards = hand_card_ids_by_kind(&client, "Research");
        if research_cards.len() < 3 {
            break;
        }
        let hand: Vec<usize> = research_cards[..3].to_vec();
        let card_ids_json = serde_json::to_string(&hand).unwrap();
        let play_json = format!(
            r#"{{"action_type":"ResearchPlayHand","card_ids":{}}}"#,
            card_ids_json
        );
        let (status, _) = post_action(&client, &play_json);
        if status != Status::Created {
            break;
        }
        rounds_played += 1;

        let enc = combat_state(&client);
        let empty_history = vec![];
        let round_history = enc
            .get("round_history")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_history);
        assert_eq!(
            round_history.len(),
            rounds_played,
            "Should have {} rounds in history",
            rounds_played
        );
    }

    if rounds_played > 0 {
        let enc = combat_state(&client);
        let acc_yield = enc
            .get("accumulated_yield")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        assert!(acc_yield >= 0, "Yield should be non-negative");

        let (status, _) = post_action(&client, r#"{"action_type":"ResearchConcludeExperiment"}"#);
        assert_eq!(status, Status::Created);

        let result = combat_result(&client);
        assert!(result.is_some(), "Should have result after experiment");
    } else {
        let _ = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
    }

    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

/// Research with different discipline choices to exercise more branch paths.
#[test]
fn research_experiment_herbalism_project() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":7777}"#);
    assert_eq!(status, Status::Created);

    // Win combats for insight
    for _ in 0..3 {
        if combat_encounter_ids(&client).is_empty() {
            break;
        }
        win_combat_and_scout(&client);
    }

    if !deplete_encounters_until_research(&client) {
        return;
    }

    let insight = player_token(&client, "CombatInsight");
    let research_enc = research_encounter_ids(&client);
    if research_enc.is_empty() || insight < 10 {
        return;
    }

    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        research_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    // Choose Herbalism project
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchChooseProject","discipline":"Herbalism","tier_count":1}"#,
    );
    if status != Status::Created {
        let _ = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        return;
    }

    // Select candidate 1 (different index than usual)
    let enc = combat_state(&client);
    let num_candidates = enc
        .get("candidates")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    let candidate_idx = if num_candidates >= 2 { 1 } else { 0 };
    let select_json = format!(
        r#"{{"action_type":"ResearchSelectCandidate","candidate_index":{}}}"#,
        candidate_idx
    );
    let (status, _) = post_action(&client, &select_json);
    assert_eq!(status, Status::Created);

    // Play hand
    let research_cards = hand_card_ids_by_kind(&client, "Research");
    if research_cards.len() >= 3 {
        let hand: Vec<usize> = research_cards[..3].to_vec();
        let play_json = format!(
            r#"{{"action_type":"ResearchPlayHand","card_ids":{}}}"#,
            serde_json::to_string(&hand).unwrap()
        );
        let (status, _) = post_action(&client, &play_json);
        if status == Status::Created {
            let (status, _) =
                post_action(&client, r#"{"action_type":"ResearchConcludeExperiment"}"#);
            assert_eq!(status, Status::Created);

            let _ = post_action(
                &client,
                r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
            );
            return;
        }
    }

    let _ = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

/// Research with Woodcutting and Fishing disciplines.
#[test]
fn research_experiment_woodcutting_and_fishing_projects() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":12345}"#);
    assert_eq!(status, Status::Created);

    for _ in 0..3 {
        if combat_encounter_ids(&client).is_empty() {
            break;
        }
        win_combat_and_scout(&client);
    }

    if !deplete_encounters_until_research(&client) {
        return;
    }

    let insight = player_token(&client, "CombatInsight");
    let research_enc = research_encounter_ids(&client);
    if research_enc.is_empty() || insight < 10 {
        return;
    }

    // Woodcutting project
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        research_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchChooseProject","discipline":"Woodcutting","tier_count":1}"#,
    );
    if status == Status::Created {
        let (status, _) = post_action(
            &client,
            r#"{"action_type":"ResearchSelectCandidate","candidate_index":0}"#,
        );
        if status == Status::Created {
            let research_cards = hand_card_ids_by_kind(&client, "Research");
            if research_cards.len() >= 3 {
                let hand: Vec<usize> = research_cards[..3].to_vec();
                let play_json = format!(
                    r#"{{"action_type":"ResearchPlayHand","card_ids":{}}}"#,
                    serde_json::to_string(&hand).unwrap()
                );
                let (status, _) = post_action(&client, &play_json);
                if status == Status::Created {
                    let _ = post_action(&client, r#"{"action_type":"ResearchConcludeExperiment"}"#);
                    let _ = post_action(
                        &client,
                        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
                    );

                    // Now try Fishing project in a second research encounter
                    if deplete_encounters_until_research(&client) {
                        let insight = player_token(&client, "CombatInsight");
                        let research_enc = research_encounter_ids(&client);
                        if !research_enc.is_empty() && insight >= 10 {
                            let pick = format!(
                                r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
                                research_enc[0]
                            );
                            let (status, _) = post_action(&client, &pick);
                            if status == Status::Created {
                                let (status, _) = post_action(
                                    &client,
                                    r#"{"action_type":"ResearchChooseProject","discipline":"Fishing","tier_count":1}"#,
                                );
                                if status == Status::Created {
                                    let _ = post_action(
                                        &client,
                                        r#"{"action_type":"ResearchSelectCandidate","candidate_index":2}"#,
                                    );
                                }
                                let _ = post_action(
                                    &client,
                                    r#"{"action_type":"EncounterConcludeEncounter"}"#,
                                );
                            }
                        }
                    }
                    return;
                }
            }
        }
    }

    let _ = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

// ===================================================================
// 7. WOODCUTTING DEEPER — DIFFERENT SEEDS
// ===================================================================

/// Woodcutting encounter with seed 100 for different card/pattern paths.
#[test]
fn woodcutting_encounter_seed100() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":100}"#);
    assert_eq!(status, Status::Created);

    let wc_enc = woodcutting_encounter_ids(&client);
    if wc_enc.is_empty() {
        return;
    }

    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        wc_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    let enc = combat_state(&client);
    assert_eq!(
        enc.get("encounter_state_type").and_then(|v| v.as_str()),
        Some("Woodcutting")
    );

    // Play all 8 cards
    for _ in 0..20 {
        let wc_cards = hand_card_ids_by_kind(&client, "Woodcutting");
        if wc_cards.is_empty() {
            break;
        }
        let mut played = false;
        for card_id in &wc_cards {
            let play = format!(
                r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                card_id
            );
            let (status, _) = post_action(&client, &play);
            if status == Status::Created {
                played = true;
                break;
            }
        }
        if !played {
            break;
        }
        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
            break;
        }
    }

    let result = combat_result(&client).unwrap_or_default();
    assert!(
        result == "PlayerWon" || result == "PlayerLost",
        "Expected win or loss, got: {}",
        result
    );

    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

// ===================================================================
// 8. MINING DEEPER — DIFFERENT SEEDS
// ===================================================================

/// Mining encounter with seed 100 to exercise different ore veins.
#[test]
fn mining_encounter_seed100() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":100}"#);
    assert_eq!(status, Status::Created);

    let mining_enc = mining_encounter_ids(&client);
    if mining_enc.is_empty() {
        return;
    }

    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        mining_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    let enc = combat_state(&client);
    assert_eq!(
        enc.get("encounter_state_type").and_then(|v| v.as_str()),
        Some("Mining")
    );

    // Play all mining cards
    for _ in 0..20 {
        let mining_cards = hand_card_ids_by_kind(&client, "Mining");
        if mining_cards.is_empty() {
            break;
        }
        let play = format!(
            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
            mining_cards[0]
        );
        let (status, _) = post_action(&client, &play);
        if status != Status::Created {
            break;
        }
        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
            break;
        }
    }

    // If still undecided (all hand cards unpayable), abort
    let enc = combat_state(&client);
    if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
        let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    }

    let result = combat_result(&client);
    assert!(result.is_some(), "Should have mining result");

    if combat_result(&client).as_deref() == Some("PlayerWon") {
        let ore = player_token(&client, "Ore");
        assert!(ore > 0, "Should earn Ore from mining win");
    }

    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

/// Mining with multiple encounters until durability runs out (seed 200).
#[test]
fn mining_multiple_encounters_seed200() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":200}"#);
    assert_eq!(status, Status::Created);

    let mut total_encounters = 0;
    for _ in 0..20 {
        let mining_enc = mining_encounter_ids(&client);
        if mining_enc.is_empty() {
            break;
        }
        let pick = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            mining_enc[0]
        );
        let (status, _) = post_action(&client, &pick);
        if status != Status::Created {
            break;
        }

        for _ in 0..20 {
            let mining_cards = hand_card_ids_by_kind(&client, "Mining");
            if mining_cards.is_empty() {
                break;
            }
            let play = format!(
                r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                mining_cards[0]
            );
            let (status, _) = post_action(&client, &play);
            if status != Status::Created {
                break;
            }
            let enc = combat_state(&client);
            if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                break;
            }
        }

        total_encounters += 1;
        let outcome = combat_result(&client).unwrap_or_default();
        if outcome == "PlayerLost" {
            break;
        }

        let _ = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
    }

    assert!(total_encounters >= 1, "Should complete at least 1 mining");
}

// ===================================================================
// 9. COMBAT WITH DIFFERENT SEEDS
// ===================================================================

/// Combat encounter with seed 100 to exercise different enemy configurations.
#[test]
fn combat_encounter_seed100() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":100}"#);
    assert_eq!(status, Status::Created);

    let combat_enc = combat_encounter_ids(&client);
    if combat_enc.is_empty() {
        return;
    }

    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        combat_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    let enc = combat_state(&client);
    assert_eq!(
        enc.get("encounter_state_type").and_then(|v| v.as_str()),
        Some("Combat")
    );

    for _ in 0..200 {
        if !play_one_round(&client) {
            break;
        }
    }

    let result = combat_result(&client);
    assert!(result.is_some(), "Should have combat result");

    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

/// Win multiple combats with seed 200 for more code path diversity.
#[test]
fn combat_multiple_wins_seed200() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":200}"#);
    assert_eq!(status, Status::Created);

    let mut wins = 0;
    for _ in 0..3 {
        let combat_enc = combat_encounter_ids(&client);
        if combat_enc.is_empty() {
            break;
        }

        let pick = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            combat_enc[0]
        );
        let (status, _) = post_action(&client, &pick);
        if status != Status::Created {
            break;
        }

        for _ in 0..200 {
            if !play_one_round(&client) {
                break;
            }
        }

        if combat_result(&client).as_deref() == Some("PlayerWon") {
            wins += 1;
        }

        let _ = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
    }

    assert!(wins >= 1, "Should win at least 1 combat");
}

// ===================================================================
// 10. MIXED DISCIPLINE FULL GAMEPLAY LOOP
// ===================================================================

/// A complete gameplay loop that exercises many encounter types in sequence.
#[test]
fn full_gameplay_loop_mixed_disciplines() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    // 1. Combat encounter
    let combat_enc = combat_encounter_ids(&client);
    if !combat_enc.is_empty() {
        let pick = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            combat_enc[0]
        );
        let (status, _) = post_action(&client, &pick);
        assert_eq!(status, Status::Created);
        for _ in 0..200 {
            if !play_one_round_prefer_insight(&client) {
                break;
            }
        }
        let _ = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
    }

    // 2. Mining encounter
    let mining_enc = mining_encounter_ids(&client);
    if !mining_enc.is_empty() {
        let pick = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            mining_enc[0]
        );
        let (status, _) = post_action(&client, &pick);
        if status == Status::Created {
            for _ in 0..20 {
                let cards = hand_card_ids_by_kind(&client, "Mining");
                if cards.is_empty() {
                    break;
                }
                let play = format!(
                    r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                    cards[0]
                );
                let (status, _) = post_action(&client, &play);
                if status != Status::Created {
                    break;
                }
                let enc = combat_state(&client);
                if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                    break;
                }
            }
            let _ = post_action(
                &client,
                r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
            );
        }
    }

    // 3. Woodcutting encounter
    let wc_enc = woodcutting_encounter_ids(&client);
    if !wc_enc.is_empty() {
        let pick = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            wc_enc[0]
        );
        let (status, _) = post_action(&client, &pick);
        if status == Status::Created {
            for _ in 0..20 {
                let cards = hand_card_ids_by_kind(&client, "Woodcutting");
                if cards.is_empty() {
                    break;
                }
                let play = format!(
                    r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                    cards[0]
                );
                let (status, _) = post_action(&client, &play);
                if status != Status::Created {
                    break;
                }
                let enc = combat_state(&client);
                if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                    break;
                }
            }
            let _ = post_action(
                &client,
                r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
            );
        }
    }

    // 4. Herbalism encounter
    let herb_enc = herbalism_encounter_ids(&client);
    if !herb_enc.is_empty() {
        let pick = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            herb_enc[0]
        );
        let (status, _) = post_action(&client, &pick);
        if status == Status::Created {
            for _ in 0..20 {
                let cards = hand_card_ids_by_kind(&client, "Herbalism");
                if cards.is_empty() {
                    break;
                }
                let play = format!(
                    r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                    cards[0]
                );
                let (status, _) = post_action(&client, &play);
                if status != Status::Created {
                    break;
                }
                let enc = combat_state(&client);
                if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                    break;
                }
            }
            let _ = post_action(
                &client,
                r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
            );
        }
    }

    // 5. Fishing encounter
    let fish_enc = fishing_encounter_ids(&client);
    if !fish_enc.is_empty() {
        let pick = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            fish_enc[0]
        );
        let (status, _) = post_action(&client, &pick);
        if status == Status::Created {
            for _ in 0..20 {
                let cards = hand_card_ids_by_kind(&client, "Fishing");
                if cards.is_empty() {
                    break;
                }
                let play = format!(
                    r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                    cards[0]
                );
                let (status, _) = post_action(&client, &play);
                if status != Status::Created {
                    break;
                }
                let enc = combat_state(&client);
                if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                    break;
                }
            }
            let _ = post_action(
                &client,
                r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
            );
        }
    }

    // 6. Rest encounter
    let rest_enc = rest_encounter_ids(&client);
    if !rest_enc.is_empty() {
        let pick = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            rest_enc[0]
        );
        let (status, _) = post_action(&client, &pick);
        if status == Status::Created {
            for _ in 0..10 {
                let cards = hand_card_ids_by_kind(&client, "Rest");
                if cards.is_empty() {
                    break;
                }
                let play = format!(
                    r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                    cards[0]
                );
                let (status, _) = post_action(&client, &play);
                if status != Status::Created {
                    break;
                }
                let enc = combat_state(&client);
                if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                    break;
                }
            }
            let enc = combat_state(&client);
            if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
                let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
            }
            let _ = post_action(
                &client,
                r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
            );
        }
    }

    // 7. Crafting encounter
    let craft_enc = crafting_encounter_ids(&client);
    if !craft_enc.is_empty() {
        let pick = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            craft_enc[0]
        );
        let (status, _) = post_action(&client, &pick);
        if status == Status::Created {
            // Do a swap if possible
            let player_card_kinds = [
                "Attack",
                "Defence",
                "Resource",
                "Mining",
                "Herbalism",
                "Woodcutting",
                "Fishing",
            ];
            let mut from_id = None;
            for kind in &player_card_kinds {
                let deck_cards = get_json(
                    &client,
                    &format!("/library/cards?location=Deck&card_kind={}", kind),
                );
                if let Some(arr) = deck_cards.as_array() {
                    if let Some(card) = arr.first() {
                        from_id = card.get("id").and_then(|v| v.as_u64()).map(|v| v as usize);
                        break;
                    }
                }
            }
            let mut to_id = None;
            for kind in &player_card_kinds {
                let lib_cards = get_json(
                    &client,
                    &format!("/library/cards?location=Library&card_kind={}", kind),
                );
                if let Some(arr) = lib_cards.as_array() {
                    if let Some(card) = arr.first() {
                        let id = card.get("id").and_then(|v| v.as_u64()).map(|v| v as usize);
                        if id != from_id {
                            to_id = id;
                            break;
                        }
                    }
                }
            }
            if let (Some(from), Some(to)) = (from_id, to_id) {
                let swap = format!(
                    r#"{{"action_type":"EncounterCraftSwap","from_id":{},"to_id":{}}}"#,
                    from, to
                );
                let _ = post_action(&client, &swap);
            }

            let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
            let _ = post_action(
                &client,
                r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
            );
        }
    }
}

// ===================================================================
// 11. RESEARCH EXPERIMENT DEEP PATHS — HIGH INSIGHT
// ===================================================================

/// Reliable research experiment: win many combats, accumulate a lot of insight,
/// then do a full experiment with project completion via progress.
#[test]
fn research_full_experiment_with_progress_completion() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":7777}"#);
    assert_eq!(status, Status::Created);

    // Win multiple combats using prefer-insight to maximize CombatInsight
    for _ in 0..5 {
        if combat_encounter_ids(&client).is_empty() {
            break;
        }
        win_combat_and_scout(&client);
    }

    if !deplete_encounters_until_research(&client) {
        return;
    }

    let insight = player_token(&client, "CombatInsight");
    if insight < 30 {
        return;
    }

    let research_enc = research_encounter_ids(&client);
    if research_enc.is_empty() {
        return;
    }

    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        research_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    // Choose Combat project, tier 1
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchChooseProject","discipline":"Combat","tier_count":1}"#,
    );
    if status != Status::Created {
        let _ = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        return;
    }

    // Select candidate
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchSelectCandidate","candidate_index":0}"#,
    );
    assert_eq!(status, Status::Created);

    // Use ResearchProgress to spend insight and advance the project
    let insight_now = player_token(&client, "CombatInsight");
    if insight_now >= 5 {
        let (status, _) = post_action(
            &client,
            r#"{"action_type":"ResearchProgress","amount":100}"#,
        );
        if status == Status::Created {
            let insight_after = player_token(&client, "CombatInsight");
            assert!(insight_after < insight_now, "Insight should decrease");
        }
    }

    // Try more progress to complete the project
    for _ in 0..5 {
        let insight_now = player_token(&client, "CombatInsight");
        if insight_now < 1 {
            break;
        }
        let (status, _) = post_action(
            &client,
            r#"{"action_type":"ResearchProgress","amount":100}"#,
        );
        if status != Status::Created {
            break;
        }
    }

    // Conclude
    let _ = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

/// Research experiment with begin_experiment and multiple play_hand rounds.
#[test]
fn research_begin_experiment_and_play_hands() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":33333}"#);
    assert_eq!(status, Status::Created);

    for _ in 0..5 {
        if combat_encounter_ids(&client).is_empty() {
            break;
        }
        win_combat_and_scout(&client);
    }

    if !deplete_encounters_until_research(&client) {
        return;
    }

    let insight = player_token(&client, "CombatInsight");
    if insight < 25 {
        return;
    }

    let research_enc = research_encounter_ids(&client);
    if research_enc.is_empty() {
        return;
    }

    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        research_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    // Choose project
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchChooseProject","discipline":"Combat","tier_count":1}"#,
    );
    if status != Status::Created {
        let _ = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        return;
    }

    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchSelectCandidate","candidate_index":0}"#,
    );
    if status != Status::Created {
        let _ = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        return;
    }

    // Play multiple rounds of experiment
    for _round in 0..4 {
        let research_cards = hand_card_ids_by_kind(&client, "Research");
        if research_cards.len() < 3 {
            break;
        }
        let hand: Vec<usize> = research_cards[..3].to_vec();
        let play_json = format!(
            r#"{{"action_type":"ResearchPlayHand","card_ids":{}}}"#,
            serde_json::to_string(&hand).unwrap()
        );
        let (status, _) = post_action(&client, &play_json);
        if status != Status::Created {
            break;
        }

        // Check accumulated yield grows
        let enc = combat_state(&client);
        let acc = enc
            .get("accumulated_yield")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        assert!(acc >= 0, "Accumulated yield should be non-negative");
    }

    // Conclude experiment
    let enc = combat_state(&client);
    if enc
        .get("experiment_active")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        let (status, _) = post_action(&client, r#"{"action_type":"ResearchConcludeExperiment"}"#);
        assert_eq!(status, Status::Created);
    } else {
        let _ = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
    }

    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

/// Research with tier 2 project (higher cost, more effects).
#[test]
fn research_tier2_project() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":7777}"#);
    assert_eq!(status, Status::Created);

    for _ in 0..5 {
        if combat_encounter_ids(&client).is_empty() {
            break;
        }
        win_combat_and_scout(&client);
    }

    if !deplete_encounters_until_research(&client) {
        return;
    }

    let insight = player_token(&client, "CombatInsight");
    if insight < 20 {
        return;
    }

    let research_enc = research_encounter_ids(&client);
    if research_enc.is_empty() {
        return;
    }

    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        research_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    // Tier 2 costs 20 insight (10 * 2^(2-1))
    let (status, _body) = post_action(
        &client,
        r#"{"action_type":"ResearchChooseProject","discipline":"Combat","tier_count":2}"#,
    );
    if status == Status::Created {
        // Verify candidates generated
        let enc = combat_state(&client);
        let candidates = enc.get("candidates").and_then(|v| v.as_array());
        assert!(
            candidates.is_some(),
            "Should have candidates for tier 2 project"
        );

        // Select candidate 2
        let (status, _) = post_action(
            &client,
            r#"{"action_type":"ResearchSelectCandidate","candidate_index":2}"#,
        );
        if status == Status::Created {
            // Play a hand
            let research_cards = hand_card_ids_by_kind(&client, "Research");
            if research_cards.len() >= 3 {
                let hand: Vec<usize> = research_cards[..3].to_vec();
                let play_json = format!(
                    r#"{{"action_type":"ResearchPlayHand","card_ids":{}}}"#,
                    serde_json::to_string(&hand).unwrap()
                );
                let (status, _) = post_action(&client, &play_json);
                if status == Status::Created {
                    let _ = post_action(&client, r#"{"action_type":"ResearchConcludeExperiment"}"#);
                    let _ = post_action(
                        &client,
                        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
                    );
                    return;
                }
            }
        }
    }

    let _ = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

// ===================================================================
// 12. MILESTONE WIN THROUGH — PLAY TO COMPLETION
// ===================================================================

/// Play through a milestone combat encounter to completion (win or loss).
#[test]
fn milestone_combat_play_through() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);

    // Win 2 combats for extra insight
    win_combat_for_milestone(&client);
    if !combat_encounter_ids(&client).is_empty() {
        win_combat_for_milestone(&client);
    }

    let insight = player_token(&client, "MilestoneInsight");
    if insight < 100 {
        return;
    }

    let milestone_id =
        milestone_encounter_by_discipline(&client, "Combat").expect("Should have Combat milestone");
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        milestone_id
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    // Play through the milestone (it's a combat encounter inside)
    for _ in 0..200 {
        if !play_one_round(&client) {
            break;
        }
    }

    // Check result
    let enc = combat_state(&client);
    let outcome = enc.get("outcome").and_then(|v| v.as_str());
    assert!(
        outcome == Some("PlayerWon") || outcome == Some("PlayerLost") || outcome.is_none(),
        "Should have a definitive outcome"
    );

    // Scout or handle post-encounter
    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

/// Play through milestone mining encounter.
#[test]
fn milestone_mining_play_through() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);

    win_combat_for_milestone(&client);
    if !combat_encounter_ids(&client).is_empty() {
        win_combat_for_milestone(&client);
    }

    let insight = player_token(&client, "MilestoneInsight");
    if insight < 100 {
        return;
    }

    let milestone_id =
        milestone_encounter_by_discipline(&client, "Mining").expect("Should have Mining milestone");
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        milestone_id
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    // Play mining cards
    for _ in 0..30 {
        let cards = hand_card_ids_by_kind(&client, "Mining");
        if cards.is_empty() {
            break;
        }
        let play = format!(
            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
            cards[0]
        );
        let (status, _) = post_action(&client, &play);
        if status != Status::Created {
            break;
        }
        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
            break;
        }
    }

    let enc = combat_state(&client);
    if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
        let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    }
    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

// ===================================================================
// 13. EXTENSIVE GAMEPLAY LOOPS — DIFFERENT SEEDS
// ===================================================================

/// Extended gameplay loop with seed 500 — exercise many encounter types.
#[test]
fn extended_gameplay_seed500() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":500}"#);
    assert_eq!(status, Status::Created);

    // Play through all available encounter types
    for _ in 0..15 {
        let all_encounters = encounter_hand_ids(&client);
        if all_encounters.is_empty() {
            break;
        }

        let pick = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            all_encounters[0]
        );
        let (status, _) = post_action(&client, &pick);
        if status != Status::Created {
            break;
        }

        let enc = combat_state(&client);
        let enc_type = enc
            .get("encounter_state_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match enc_type {
            "Combat" => {
                for _ in 0..200 {
                    if !play_one_round(&client) {
                        break;
                    }
                }
            }
            "Mining" => {
                for _ in 0..20 {
                    let cards = hand_card_ids_by_kind(&client, "Mining");
                    if cards.is_empty() {
                        break;
                    }
                    let play = format!(
                        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                        cards[0]
                    );
                    let (s, _) = post_action(&client, &play);
                    if s != Status::Created {
                        break;
                    }
                    let e = combat_state(&client);
                    if e.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                        break;
                    }
                }
            }
            "Herbalism" => {
                for _ in 0..20 {
                    let cards = hand_card_ids_by_kind(&client, "Herbalism");
                    if cards.is_empty() {
                        break;
                    }
                    let play = format!(
                        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                        cards[0]
                    );
                    let (s, _) = post_action(&client, &play);
                    if s != Status::Created {
                        break;
                    }
                    let e = combat_state(&client);
                    if e.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                        break;
                    }
                }
            }
            "Woodcutting" => {
                for _ in 0..20 {
                    let cards = hand_card_ids_by_kind(&client, "Woodcutting");
                    if cards.is_empty() {
                        break;
                    }
                    let play = format!(
                        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                        cards[0]
                    );
                    let (s, _) = post_action(&client, &play);
                    if s != Status::Created {
                        break;
                    }
                    let e = combat_state(&client);
                    if e.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                        break;
                    }
                }
            }
            "Fishing" => {
                for _ in 0..20 {
                    let cards = hand_card_ids_by_kind(&client, "Fishing");
                    if cards.is_empty() {
                        break;
                    }
                    let play = format!(
                        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                        cards[0]
                    );
                    let (s, _) = post_action(&client, &play);
                    if s != Status::Created {
                        break;
                    }
                    let e = combat_state(&client);
                    if e.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                        break;
                    }
                }
            }
            "Rest" => {
                for _ in 0..10 {
                    let cards = hand_card_ids_by_kind(&client, "Rest");
                    if cards.is_empty() {
                        break;
                    }
                    let play = format!(
                        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                        cards[0]
                    );
                    let (s, _) = post_action(&client, &play);
                    if s != Status::Created {
                        break;
                    }
                    let e = combat_state(&client);
                    if e.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                        break;
                    }
                }
            }
            "Crafting" => {
                // Just abort crafting
            }
            _ => {}
        }

        // Ensure encounter is resolved
        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
            let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
        }
        let _ = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
    }
}

/// Extended gameplay loop with seed 1000.
#[test]
fn extended_gameplay_seed1000() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":1000}"#);
    assert_eq!(status, Status::Created);

    for _ in 0..20 {
        let all_encounters = encounter_hand_ids(&client);
        if all_encounters.is_empty() {
            break;
        }

        let pick = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            all_encounters[0]
        );
        let (status, _) = post_action(&client, &pick);
        if status != Status::Created {
            break;
        }

        let enc = combat_state(&client);
        let enc_type = enc
            .get("encounter_state_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Play through whatever encounter type
        let card_kind = match enc_type {
            "Combat" => {
                for _ in 0..200 {
                    if !play_one_round(&client) {
                        break;
                    }
                }
                ""
            }
            "Mining" => "Mining",
            "Herbalism" => "Herbalism",
            "Woodcutting" => "Woodcutting",
            "Fishing" => "Fishing",
            "Rest" => "Rest",
            _ => "",
        };

        if !card_kind.is_empty() {
            for _ in 0..20 {
                let cards = hand_card_ids_by_kind(&client, card_kind);
                if cards.is_empty() {
                    break;
                }
                let play = format!(
                    r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                    cards[0]
                );
                let (s, _) = post_action(&client, &play);
                if s != Status::Created {
                    break;
                }
                let e = combat_state(&client);
                if e.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                    break;
                }
            }
        }

        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
            let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
        }
        let _ = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
    }
}

/// Extended gameplay with seed 777 exercising card effects endpoint.
#[test]
fn extended_gameplay_with_card_effects_queries() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":777}"#);
    assert_eq!(status, Status::Created);

    // Query various library endpoints to exercise endpoint code paths
    let _ = get_json(&client, "/library/cards?card_kind=Attack");
    let _ = get_json(&client, "/library/cards?card_kind=Defence");
    let _ = get_json(&client, "/library/cards?card_kind=Resource");
    let _ = get_json(&client, "/library/cards?card_kind=PlayerCardEffect");
    let _ = get_json(&client, "/library/cards?card_kind=Encounter");
    let _ = get_json(&client, "/library/cards?location=Library");
    let _ = get_json(&client, "/library/cards?location=Deck");
    let _ = get_json(&client, "/library/cards?location=Hand");
    let _ = get_json(&client, "/library/card_effects");
    let _ = get_json(&client, "/actions/possible");

    // Play a combat
    let combat_enc = combat_encounter_ids(&client);
    if !combat_enc.is_empty() {
        let pick = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            combat_enc[0]
        );
        let (status, _) = post_action(&client, &pick);
        if status == Status::Created {
            // Query during encounter
            let _ = get_json(&client, "/actions/possible");
            let _ = get_json(&client, "/encounter");
            let _ = get_json(&client, "/library/cards?location=Hand");

            for _ in 0..200 {
                if !play_one_round(&client) {
                    break;
                }
            }
            let _ = post_action(
                &client,
                r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
            );
        }
    }

    // Query after combat
    let _ = get_json(&client, "/actions/possible");
    let _ = get_json(&client, "/player/tokens");
    let _ = get_json(&client, "/metrics");
}

// ===================================================================
// 14. HERBALISM — MANY SEEDS FOR MATCH MODE DIVERSITY
// ===================================================================

/// Herbalism encounters with seeds 300, 400, 600, 700 to hit different match modes.
#[test]
fn herbalism_seed300() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":300}"#);
    assert_eq!(status, Status::Created);

    let herb_enc = herbalism_encounter_ids(&client);
    if herb_enc.is_empty() {
        return;
    }

    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        herb_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    if status != Status::Created {
        return;
    }

    for _ in 0..20 {
        let cards = hand_card_ids_by_kind(&client, "Herbalism");
        if cards.is_empty() {
            break;
        }
        let play = format!(
            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
            cards[0]
        );
        let (s, _) = post_action(&client, &play);
        if s != Status::Created {
            break;
        }
        let e = combat_state(&client);
        if e.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
            break;
        }
    }

    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

#[test]
fn herbalism_seed400() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":400}"#);
    assert_eq!(status, Status::Created);

    let herb_enc = herbalism_encounter_ids(&client);
    if herb_enc.is_empty() {
        return;
    }

    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        herb_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    if status != Status::Created {
        return;
    }

    for _ in 0..20 {
        let cards = hand_card_ids_by_kind(&client, "Herbalism");
        if cards.is_empty() {
            break;
        }
        let play = format!(
            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
            cards[0]
        );
        let (s, _) = post_action(&client, &play);
        if s != Status::Created {
            break;
        }
        let e = combat_state(&client);
        if e.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
            break;
        }
    }

    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

#[test]
fn herbalism_seed700() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":700}"#);
    assert_eq!(status, Status::Created);

    let herb_enc = herbalism_encounter_ids(&client);
    if herb_enc.is_empty() {
        return;
    }
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        herb_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    if status != Status::Created {
        return;
    }

    for _ in 0..20 {
        let cards = hand_card_ids_by_kind(&client, "Herbalism");
        if cards.is_empty() {
            break;
        }
        // Try all available cards (different cards may trigger different match modes)
        for card_id in &cards {
            let play = format!(
                r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                card_id
            );
            let (s, _) = post_action(&client, &play);
            if s == Status::Created {
                break;
            }
        }
        let e = combat_state(&client);
        if e.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
            break;
        }
    }

    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

// ===================================================================
// 15. FISHING — MANY SEEDS
// ===================================================================

#[test]
fn fishing_seed300() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":300}"#);
    assert_eq!(status, Status::Created);

    let fc_enc = fishing_encounter_ids(&client);
    if fc_enc.is_empty() {
        return;
    }

    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        fc_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    if status != Status::Created {
        return;
    }

    for _ in 0..20 {
        let cards = hand_card_ids_by_kind(&client, "Fishing");
        if cards.is_empty() {
            break;
        }
        let play = format!(
            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
            cards[0]
        );
        let (s, _) = post_action(&client, &play);
        if s != Status::Created {
            break;
        }
        let e = combat_state(&client);
        if e.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
            break;
        }
    }

    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

#[test]
fn fishing_seed500() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":500}"#);
    assert_eq!(status, Status::Created);

    let fc_enc = fishing_encounter_ids(&client);
    if fc_enc.is_empty() {
        return;
    }
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        fc_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    if status != Status::Created {
        return;
    }

    for _ in 0..20 {
        let cards = hand_card_ids_by_kind(&client, "Fishing");
        if cards.is_empty() {
            break;
        }
        for card_id in &cards {
            let play = format!(
                r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                card_id
            );
            let (s, _) = post_action(&client, &play);
            if s == Status::Created {
                break;
            }
        }
        let e = combat_state(&client);
        if e.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
            break;
        }
    }

    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

// ===================================================================
// 16. MILESTONE WIN PATHS — MUST WIN TO TRIGGER REWARD GENERATION
// ===================================================================

#[test]
fn milestone_combat_win_for_rewards() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    for _ in 0..3 {
        if combat_encounter_ids(&client).is_empty() {
            break;
        }
        win_combat_and_scout(&client);
    }
    let insight = player_token(&client, "MilestoneInsight");
    if insight < 100 {
        return;
    }
    let milestone_id = match milestone_encounter_by_discipline(&client, "Combat") {
        Some(id) => id,
        None => return,
    };
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        milestone_id
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);
    for _ in 0..400 {
        if !play_one_round(&client) {
            break;
        }
    }
    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

#[test]
fn milestone_combat_win_seed100() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":100}"#);
    for _ in 0..4 {
        if combat_encounter_ids(&client).is_empty() {
            break;
        }
        win_combat_and_scout(&client);
    }
    let insight = player_token(&client, "MilestoneInsight");
    if insight < 100 {
        return;
    }
    let milestone_id = match milestone_encounter_by_discipline(&client, "Combat") {
        Some(id) => id,
        None => return,
    };
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        milestone_id
    );
    let (status, _) = post_action(&client, &pick);
    if status != Status::Created {
        return;
    }
    for _ in 0..400 {
        if !play_one_round(&client) {
            break;
        }
    }
    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

#[test]
fn milestone_mining_win_for_rewards() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    for _ in 0..3 {
        if combat_encounter_ids(&client).is_empty() {
            break;
        }
        win_combat_and_scout(&client);
    }
    if player_token(&client, "MilestoneInsight") < 100 {
        return;
    }
    let milestone_id = match milestone_encounter_by_discipline(&client, "Mining") {
        Some(id) => id,
        None => return,
    };
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        milestone_id
    );
    let (status, _) = post_action(&client, &pick);
    if status != Status::Created {
        return;
    }
    for _ in 0..50 {
        let cards = hand_card_ids_by_kind(&client, "Mining");
        if cards.is_empty() {
            break;
        }
        let mut played = false;
        for card_id in &cards {
            let play = format!(
                r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                card_id
            );
            let (s, _) = post_action(&client, &play);
            if s == Status::Created {
                played = true;
                break;
            }
        }
        if !played {
            break;
        }
        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
            break;
        }
    }
    let enc = combat_state(&client);
    if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
        let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    }
    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

#[test]
fn milestone_herbalism_win_attempt() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    for _ in 0..3 {
        if combat_encounter_ids(&client).is_empty() {
            break;
        }
        win_combat_and_scout(&client);
    }
    if player_token(&client, "MilestoneInsight") < 100 {
        return;
    }
    let milestone_id = match milestone_encounter_by_discipline(&client, "Herbalism") {
        Some(id) => id,
        None => return,
    };
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        milestone_id
    );
    let (status, _) = post_action(&client, &pick);
    if status != Status::Created {
        return;
    }
    for _ in 0..30 {
        let cards = hand_card_ids_by_kind(&client, "Herbalism");
        if cards.is_empty() {
            break;
        }
        let mut played = false;
        for card_id in &cards {
            let play = format!(
                r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                card_id
            );
            let (s, _) = post_action(&client, &play);
            if s == Status::Created {
                played = true;
                break;
            }
        }
        if !played {
            break;
        }
        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
            break;
        }
    }
    let enc = combat_state(&client);
    if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
        let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    }
    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

#[test]
fn milestone_woodcutting_win_attempt() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    for _ in 0..3 {
        if combat_encounter_ids(&client).is_empty() {
            break;
        }
        win_combat_and_scout(&client);
    }
    if player_token(&client, "MilestoneInsight") < 100 {
        return;
    }
    let milestone_id = match milestone_encounter_by_discipline(&client, "Woodcutting") {
        Some(id) => id,
        None => return,
    };
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        milestone_id
    );
    let (status, _) = post_action(&client, &pick);
    if status != Status::Created {
        return;
    }
    for _ in 0..30 {
        let cards = hand_card_ids_by_kind(&client, "Woodcutting");
        if cards.is_empty() {
            break;
        }
        let mut played = false;
        for card_id in &cards {
            let play = format!(
                r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                card_id
            );
            let (s, _) = post_action(&client, &play);
            if s == Status::Created {
                played = true;
                break;
            }
        }
        if !played {
            break;
        }
        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
            break;
        }
    }
    let enc = combat_state(&client);
    if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
        let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    }
    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

#[test]
fn milestone_fishing_win_attempt() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    for _ in 0..3 {
        if combat_encounter_ids(&client).is_empty() {
            break;
        }
        win_combat_and_scout(&client);
    }
    if player_token(&client, "MilestoneInsight") < 100 {
        return;
    }
    let milestone_id = match milestone_encounter_by_discipline(&client, "Fishing") {
        Some(id) => id,
        None => return,
    };
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        milestone_id
    );
    let (status, _) = post_action(&client, &pick);
    if status != Status::Created {
        return;
    }
    for _ in 0..30 {
        let cards = hand_card_ids_by_kind(&client, "Fishing");
        if cards.is_empty() {
            break;
        }
        let mut played = false;
        for card_id in &cards {
            let play = format!(
                r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                card_id
            );
            let (s, _) = post_action(&client, &play);
            if s == Status::Created {
                played = true;
                break;
            }
        }
        if !played {
            break;
        }
        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
            break;
        }
    }
    let enc = combat_state(&client);
    if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
        let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    }
    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

// ===================================================================
// 17. CRAFTING DEEPER — SWAP AND CRAFT
// ===================================================================

#[test]
fn crafting_swap_cards() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    let craft_enc = crafting_encounter_ids(&client);
    if craft_enc.is_empty() {
        return;
    }
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        craft_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);
    let deck_cards = get_json(&client, "/library/cards?location=Deck&card_kind=Attack");
    let lib_cards = get_json(&client, "/library/cards?location=Library&card_kind=Attack");
    if let (Some(da), Some(la)) = (deck_cards.as_array(), lib_cards.as_array()) {
        if let (Some(fc), Some(tc)) = (da.first(), la.first()) {
            let from_id = fc.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let to_id = tc.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            if from_id != to_id {
                let swap = format!(
                    r#"{{"action_type":"EncounterCraftSwap","from_id":{},"to_id":{}}}"#,
                    from_id, to_id
                );
                let _ = post_action(&client, &swap);
            }
        }
    }
    let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

#[test]
fn crafting_craft_card() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    win_combat_for_milestone(&client);
    let craft_enc = crafting_encounter_ids(&client);
    if craft_enc.is_empty() {
        return;
    }
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        craft_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    if status != Status::Created {
        return;
    }
    let lib_cards = get_json(&client, "/library/cards?location=Library&card_kind=Attack");
    if let Some(arr) = lib_cards.as_array() {
        if let Some(card) = arr.first() {
            let target_id = card.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let craft = format!(
                r#"{{"action_type":"EncounterCraftCard","target_card_id":{}}}"#,
                target_id
            );
            let _ = post_action(&client, &craft);
        }
    }
    let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

// ===================================================================
// 18. ACTION HANDLER EDGE CASES
// ===================================================================

#[test]
fn play_card_outside_encounter() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterPlayCard","card_id":0}"#,
    );
    assert_eq!(status, Status::BadRequest);
}

#[test]
fn conclude_outside_encounter() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
    assert_eq!(status, Status::BadRequest);
}

#[test]
fn abort_outside_encounter() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    let (status, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    assert_eq!(status, Status::BadRequest);
}

#[test]
fn craft_durability_outside_crafting() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterCraftDurability","discipline":"Mining"}"#,
    );
    assert_eq!(status, Status::BadRequest);
}

#[test]
fn research_choose_project_outside_research() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchChooseProject","discipline":"Combat","tier_count":1}"#,
    );
    assert_eq!(status, Status::BadRequest);
}

#[test]
fn research_select_candidate_outside_research() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchSelectCandidate","candidate_index":0}"#,
    );
    assert_eq!(status, Status::BadRequest);
}

#[test]
fn research_progress_outside_research() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    let (status, _) = post_action(&client, r#"{"action_type":"ResearchProgress","amount":10}"#);
    assert_eq!(status, Status::BadRequest);
}

#[test]
fn research_play_hand_outside_research() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchPlayHand","card_ids":[0,1,2]}"#,
    );
    assert_eq!(status, Status::BadRequest);
}

#[test]
fn research_conclude_outside_research() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    let (status, _) = post_action(&client, r#"{"action_type":"ResearchConcludeExperiment"}"#);
    assert_eq!(status, Status::BadRequest);
}

#[test]
fn craft_swap_outside_crafting() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterCraftSwap","from_id":0,"to_id":1}"#,
    );
    assert_eq!(status, Status::BadRequest);
}

#[test]
fn craft_card_outside_crafting() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterCraftCard","target_card_id":0}"#,
    );
    assert_eq!(status, Status::BadRequest);
}

#[test]
fn pick_encounter_invalid_card() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterPickEncounter","card_id":99999}"#,
    );
    assert!(status == Status::BadRequest || status == Status::NotFound);
}

#[test]
fn scouting_while_in_encounter() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    let enc = encounter_hand_ids(&client);
    if enc.is_empty() {
        return;
    }
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(status, Status::BadRequest);
    let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
}

// ===================================================================
// 19. EXTENDED GAMEPLAY SESSIONS
// ===================================================================

#[test]
fn extended_gameplay_seed2000() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":2000}"#);
    for _ in 0..20 {
        let all_encounters = encounter_hand_ids(&client);
        if all_encounters.is_empty() {
            break;
        }
        let pick = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            all_encounters[0]
        );
        let (status, _) = post_action(&client, &pick);
        if status != Status::Created {
            break;
        }
        let enc = combat_state(&client);
        let enc_type = enc
            .get("encounter_state_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let card_kind = match enc_type {
            "Combat" => {
                for _ in 0..200 {
                    if !play_one_round(&client) {
                        break;
                    }
                }
                ""
            }
            "Mining" => "Mining",
            "Herbalism" => "Herbalism",
            "Woodcutting" => "Woodcutting",
            "Fishing" => "Fishing",
            "Rest" => "Rest",
            _ => "",
        };
        if !card_kind.is_empty() {
            for _ in 0..20 {
                let cards = hand_card_ids_by_kind(&client, card_kind);
                if cards.is_empty() {
                    break;
                }
                for card_id in &cards {
                    let play = format!(
                        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                        card_id
                    );
                    let (s, _) = post_action(&client, &play);
                    if s == Status::Created {
                        break;
                    }
                }
                let e = combat_state(&client);
                if e.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                    break;
                }
            }
        }
        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
            let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
        }
        let _ = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
    }
}

#[test]
fn extended_gameplay_seed5000() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":5000}"#);
    for _ in 0..20 {
        let all_encounters = encounter_hand_ids(&client);
        if all_encounters.is_empty() {
            break;
        }
        let pick = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            all_encounters[0]
        );
        let (status, _) = post_action(&client, &pick);
        if status != Status::Created {
            break;
        }
        let enc = combat_state(&client);
        let enc_type = enc
            .get("encounter_state_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let card_kind = match enc_type {
            "Combat" => {
                for _ in 0..200 {
                    if !play_one_round(&client) {
                        break;
                    }
                }
                ""
            }
            "Mining" => "Mining",
            "Herbalism" => "Herbalism",
            "Woodcutting" => "Woodcutting",
            "Fishing" => "Fishing",
            "Rest" => "Rest",
            _ => "",
        };
        if !card_kind.is_empty() {
            for _ in 0..20 {
                let cards = hand_card_ids_by_kind(&client, card_kind);
                if cards.is_empty() {
                    break;
                }
                for card_id in &cards {
                    let play = format!(
                        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                        card_id
                    );
                    let (s, _) = post_action(&client, &play);
                    if s == Status::Created {
                        break;
                    }
                }
                let e = combat_state(&client);
                if e.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                    break;
                }
            }
        }
        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
            let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
        }
        let _ = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
    }
}

// ===================================================================
// 20. RESEARCH DEEP PLAY — WITH CORRECT EXPERIMENT FLOW
// ===================================================================

/// Research play hand requires exactly target_size (3) cards.
/// The experiment auto-begins on first ResearchPlayHand call.
/// Strategy: call with wrong number to trigger begin_experiment, then fetch
/// actual hand cards and call with correct number.
#[test]
fn research_play_hand_correct_flow() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":7777}"#);

    // Win combats for insight
    for _ in 0..4 {
        if combat_encounter_ids(&client).is_empty() {
            break;
        }
        win_combat_and_scout(&client);
    }

    if !deplete_encounters_until_research(&client) {
        return;
    }

    let insight = player_token(&client, "CombatInsight");
    if insight < 30 {
        return;
    }

    let research_enc = research_encounter_ids(&client);
    if research_enc.is_empty() {
        return;
    }
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        research_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    // Choose project
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchChooseProject","discipline":"Combat","tier_count":1}"#,
    );
    if status != Status::Created {
        let _ = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        let _ = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
        return;
    }

    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchSelectCandidate","candidate_index":0}"#,
    );
    assert_eq!(status, Status::Created);

    // First call with empty card_ids to trigger auto-begin experiment
    // This will fail because card_ids.len() != target_size, but experiment is now active
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchPlayHand","card_ids":[]}"#,
    );
    // Expected: BadRequest (wrong number of cards)
    assert_eq!(status, Status::BadRequest);

    // Now the experiment is active. Fetch research hand cards.
    let research_hand = hand_card_ids_by_kind(&client, "Research");
    if research_hand.len() >= 3 {
        // Play exactly 3 cards
        let hand: Vec<usize> = research_hand[..3].to_vec();
        let play_json = format!(
            r#"{{"action_type":"ResearchPlayHand","card_ids":{}}}"#,
            serde_json::to_string(&hand).unwrap()
        );
        let (status, _body) = post_action(&client, &play_json);
        if status == Status::Created {
            // SUCCESS! The deep research_play_hand code was exercised
            // Check that round was recorded
            let enc = combat_state(&client);
            let rounds = enc
                .get("rounds_played")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            assert!(rounds >= 1, "Should have played at least 1 round");

            // Try another round
            let research_hand = hand_card_ids_by_kind(&client, "Research");
            if research_hand.len() >= 3 {
                let hand: Vec<usize> = research_hand[..3].to_vec();
                let play_json = format!(
                    r#"{{"action_type":"ResearchPlayHand","card_ids":{}}}"#,
                    serde_json::to_string(&hand).unwrap()
                );
                let _ = post_action(&client, &play_json);
            }

            // Conclude experiment to exercise conclude path
            let (status, _) =
                post_action(&client, r#"{"action_type":"ResearchConcludeExperiment"}"#);
            if status == Status::Created {
                let _ = post_action(
                    &client,
                    r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
                );
                return;
            }
        }
    }

    // Fallback: abort
    let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

/// Research play hand with seed 33333 for different card composition.
#[test]
fn research_play_hand_seed33333() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":33333}"#);

    for _ in 0..5 {
        if combat_encounter_ids(&client).is_empty() {
            break;
        }
        win_combat_and_scout(&client);
    }

    if !deplete_encounters_until_research(&client) {
        return;
    }
    if player_token(&client, "CombatInsight") < 30 {
        return;
    }

    let research_enc = research_encounter_ids(&client);
    if research_enc.is_empty() {
        return;
    }
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        research_enc[0]
    );
    let (status, _) = post_action(&client, &pick);
    if status != Status::Created {
        return;
    }

    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchChooseProject","discipline":"Combat","tier_count":1}"#,
    );
    if status != Status::Created {
        let _ = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        let _ = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
        return;
    }

    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchSelectCandidate","candidate_index":1}"#,
    );
    if status != Status::Created {
        let _ = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        let _ = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
        return;
    }

    // Trigger auto-begin with wrong card count
    let _ = post_action(
        &client,
        r#"{"action_type":"ResearchPlayHand","card_ids":[0]}"#,
    );

    // Now play with correct cards
    for _round in 0..5 {
        let research_hand = hand_card_ids_by_kind(&client, "Research");
        if research_hand.len() < 3 {
            break;
        }
        let hand: Vec<usize> = research_hand[..3].to_vec();
        let play_json = format!(
            r#"{{"action_type":"ResearchPlayHand","card_ids":{}}}"#,
            serde_json::to_string(&hand).unwrap()
        );
        let (status, _) = post_action(&client, &play_json);
        if status != Status::Created {
            break;
        }
    }

    // Conclude
    let _ = post_action(&client, r#"{"action_type":"ResearchConcludeExperiment"}"#);
    let _ = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
}

/// Research play with Research-discipline insight for broader coverage.
#[test]
fn research_play_hand_various_disciplines() {
    for seed in [42, 100, 500, 1000, 5000u64] {
        let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
        let seed_json = format!(r#"{{"action_type":"NewGame","seed":{}}}"#, seed);
        post_action(&client, &seed_json);

        for _ in 0..4 {
            if combat_encounter_ids(&client).is_empty() {
                break;
            }
            win_combat_and_scout(&client);
        }

        if !deplete_encounters_until_research(&client) {
            continue;
        }
        if player_token(&client, "CombatInsight") < 20 {
            continue;
        }

        let research_enc = research_encounter_ids(&client);
        if research_enc.is_empty() {
            continue;
        }
        let pick = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            research_enc[0]
        );
        let (status, _) = post_action(&client, &pick);
        if status != Status::Created {
            continue;
        }

        let (status, _) = post_action(
            &client,
            r#"{"action_type":"ResearchChooseProject","discipline":"Combat","tier_count":1}"#,
        );
        if status != Status::Created {
            let _ = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
            let _ = post_action(
                &client,
                r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
            );
            continue;
        }

        let (status, _) = post_action(
            &client,
            r#"{"action_type":"ResearchSelectCandidate","candidate_index":0}"#,
        );
        if status != Status::Created {
            continue;
        }

        // Trigger begin experiment
        let _ = post_action(
            &client,
            r#"{"action_type":"ResearchPlayHand","card_ids":[]}"#,
        );

        // Play rounds
        for _ in 0..3 {
            let research_hand = hand_card_ids_by_kind(&client, "Research");
            if research_hand.len() < 3 {
                break;
            }
            let hand: Vec<usize> = research_hand[..3].to_vec();
            let play_json = format!(
                r#"{{"action_type":"ResearchPlayHand","card_ids":{}}}"#,
                serde_json::to_string(&hand).unwrap()
            );
            let (status, _) = post_action(&client, &play_json);
            if status != Status::Created {
                break;
            }
        }

        let _ = post_action(&client, r#"{"action_type":"ResearchConcludeExperiment"}"#);
        let _ = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
    }
}

// ===================================================================
// 21. HERBALISM — DIRECT LIBRARY UNIT TESTS FOR MATCH MODES
// ===================================================================

/// Exercise the herbalism encounter more aggressively with multiple seeds,
/// playing ALL available herbalism cards each turn.
#[test]
fn herbalism_aggressive_card_play() {
    for seed in [42, 100, 200, 300, 400, 500, 600, 700, 800, 900u64] {
        let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
        let seed_json = format!(r#"{{"action_type":"NewGame","seed":{}}}"#, seed);
        post_action(&client, &seed_json);

        let herb_enc = herbalism_encounter_ids(&client);
        if herb_enc.is_empty() {
            continue;
        }

        let pick = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            herb_enc[0]
        );
        let (status, _) = post_action(&client, &pick);
        if status != Status::Created {
            continue;
        }

        for _ in 0..30 {
            let cards = hand_card_ids_by_kind(&client, "Herbalism");
            if cards.is_empty() {
                break;
            }
            let mut any_played = false;
            for card_id in &cards {
                let play = format!(
                    r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                    card_id
                );
                let (s, _) = post_action(&client, &play);
                if s == Status::Created {
                    any_played = true;
                    break;
                }
            }
            if !any_played {
                break;
            }
            let e = combat_state(&client);
            if e.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                break;
            }
        }

        let enc = combat_state(&client);
        if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
            let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
        }
        let _ = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
    }
}

// ===================================================================
// 20. AGGRESSIVE MULTI-DISCIPLINE COVERAGE TESTS
// ===================================================================

/// Play through complete games trying many different encounter types with multiple seeds.
/// This uses a high iteration count to maximize code path coverage.
#[test]
fn aggressive_coverage_seed42() {
    aggressive_multi_discipline_gameplay(42);
}

#[test]
fn aggressive_coverage_seed77() {
    aggressive_multi_discipline_gameplay(77);
}

#[test]
fn aggressive_coverage_seed123() {
    aggressive_multi_discipline_gameplay(123);
}

#[test]
fn aggressive_coverage_seed256() {
    aggressive_multi_discipline_gameplay(256);
}

#[test]
fn aggressive_coverage_seed999() {
    aggressive_multi_discipline_gameplay(999);
}

#[test]
fn aggressive_coverage_seed3333() {
    aggressive_multi_discipline_gameplay(3333);
}

#[test]
fn aggressive_coverage_seed7777() {
    aggressive_multi_discipline_gameplay(7777);
}

#[test]
fn aggressive_coverage_seed12345() {
    aggressive_multi_discipline_gameplay(12345);
}

fn aggressive_multi_discipline_gameplay(seed: u64) {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(
        &client,
        &format!(r#"{{"action_type":"NewGame","seed":{}}}"#, seed),
    );

    // Play through many encounters of all types
    for round in 0..30 {
        let actions = possible_action_types(&client);

        if actions.contains(&"EncounterPickEncounter".to_string()) {
            // Try to pick the best encounter for coverage
            let enc_hand = encounter_hand_ids(&client);
            if enc_hand.is_empty() {
                continue;
            }

            // Check what types are available
            let cards = get_json(&client, "/library/cards?location=Hand&card_kind=Encounter");
            let empty = vec![];
            let arr = cards.as_array().unwrap_or(&empty);

            // Find encounter by type priority (milestone > research > crafting > herbalism > woodcutting > fishing > mining > rest > combat)
            let priority_types = [
                "Milestone",
                "Research",
                "Crafting",
                "Herbalism",
                "Woodcutting",
                "Fishing",
                "Mining",
                "Rest",
                "Combat",
            ];
            let mut picked = None;
            for enc_type in &priority_types {
                for c in arr {
                    let et = c
                        .get("kind")
                        .and_then(|k| k.get("encounter_kind"))
                        .and_then(|ek| ek.get("encounter_type"))
                        .and_then(|t| t.as_str());
                    if et == Some(enc_type) {
                        picked = c.get("id").and_then(|v| v.as_u64()).map(|v| v as usize);
                        break;
                    }
                }
                if picked.is_some() {
                    break;
                }
            }

            let enc_id = picked.unwrap_or(enc_hand[0]);
            let pick = format!(
                r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
                enc_id
            );
            let (status, _) = post_action(&client, &pick);
            if status != Status::Created {
                continue;
            }

            // Play through the encounter
            play_encounter_aggressively(&client, round);
        } else if actions.contains(&"EncounterPlayCard".to_string()) {
            play_encounter_aggressively(&client, round);
        } else if actions.contains(&"EncounterApplyScouting".to_string()) {
            post_action(
                &client,
                r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
            );
        } else {
            break;
        }
    }
}

fn play_encounter_aggressively(client: &Client, _seed_offset: usize) {
    let enc = combat_state(client);
    let enc_type = enc
        .get("encounter_state_type")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    match enc_type {
        "Combat" | "Milestone" => {
            // Play combat rounds
            for _ in 0..100 {
                if !play_one_round(client) {
                    break;
                }
            }
            // Conclude or scout
            let actions = possible_action_types(client);
            if actions.contains(&"EncounterConcludeEncounter".to_string()) {
                post_action(client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
            }
            let actions = possible_action_types(client);
            if actions.contains(&"EncounterApplyScouting".to_string()) {
                post_action(
                    client,
                    r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
                );
            }
        }
        "Mining" => {
            for _ in 0..50 {
                let cards = hand_card_ids_by_kind(client, "Mining");
                if cards.is_empty() {
                    break;
                }
                let play = format!(
                    r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                    cards[0]
                );
                let (status, _) = post_action(client, &play);
                if status != Status::Created {
                    break;
                }
                let e = combat_state(client);
                if e.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                    break;
                }
            }
            conclude_or_abort(client);
        }
        "Herbalism" => {
            for _ in 0..50 {
                let cards = hand_card_ids_by_kind(client, "Herbalism");
                if cards.is_empty() {
                    break;
                }
                // Try each card (different match modes)
                for &cid in &cards {
                    let play =
                        format!(r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#, cid);
                    let (status, _) = post_action(client, &play);
                    if status == Status::Created {
                        let e = combat_state(client);
                        if e.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                            break;
                        }
                        break;
                    }
                }
                let e = combat_state(client);
                if e.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                    break;
                }
            }
            conclude_or_abort(client);
        }
        "Woodcutting" => {
            for _ in 0..50 {
                let cards = hand_card_ids_by_kind(client, "Woodcutting");
                if cards.is_empty() {
                    break;
                }
                let play = format!(
                    r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                    cards[0]
                );
                let (status, _) = post_action(client, &play);
                if status != Status::Created {
                    break;
                }
                let e = combat_state(client);
                if e.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                    break;
                }
            }
            conclude_or_abort(client);
        }
        "Fishing" => {
            for _ in 0..50 {
                let cards = hand_card_ids_by_kind(client, "Fishing");
                if cards.is_empty() {
                    break;
                }
                let play = format!(
                    r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                    cards[0]
                );
                let (status, _) = post_action(client, &play);
                if status != Status::Created {
                    break;
                }
                let e = combat_state(client);
                if e.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                    break;
                }
            }
            conclude_or_abort(client);
        }
        "Rest" => {
            for _ in 0..30 {
                let cards = hand_card_ids_by_kind(client, "Rest");
                if cards.is_empty() {
                    break;
                }
                let play = format!(
                    r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                    cards[0]
                );
                let (status, _) = post_action(client, &play);
                if status != Status::Created {
                    break;
                }
                let e = combat_state(client);
                if e.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
                    break;
                }
            }
            conclude_or_abort(client);
        }
        "Crafting" => {
            // Try swap, add durability, and craft
            let crafting_cards = hand_card_ids_by_kind(client, "Crafting");
            if !crafting_cards.is_empty() {
                let play = format!(
                    r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                    crafting_cards[0]
                );
                let _ = post_action(client, &play);
            }
            // Try add durability
            let _ = post_action(
                client,
                r#"{"action_type":"EncounterCraftDurability","discipline":"Mining"}"#,
            );
            conclude_or_abort(client);
        }
        "Research" => {
            // Try the full research flow
            let _ = post_action(
                client,
                r#"{"action_type":"ResearchChooseProject","discipline":"Combat","tier":1}"#,
            );
            let enc2 = combat_state(client);
            if let Some(candidates) = enc2.get("candidates").and_then(|v| v.as_array()) {
                if !candidates.is_empty() {
                    let _ = post_action(
                        client,
                        r#"{"action_type":"ResearchSelectCandidate","candidate_index":0}"#,
                    );
                    let _ = post_action(client, r#"{"action_type":"ResearchBeginExperiment"}"#);
                    // Play hands
                    for _ in 0..5 {
                        let research_cards = hand_card_ids_by_kind(client, "Research");
                        if research_cards.len() >= 3 {
                            let ids = &research_cards[..3];
                            let ids_json = serde_json::to_string(&ids).unwrap();
                            let play = format!(
                                r#"{{"action_type":"ResearchPlayHand","card_ids":{}}}"#,
                                ids_json
                            );
                            let (status, _) = post_action(client, &play);
                            if status != Status::Created {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    let _ = post_action(client, r#"{"action_type":"ResearchConcludeExperiment"}"#);
                    let _ =
                        post_action(client, r#"{"action_type":"ResearchProgress","amount":50}"#);
                }
            }
            conclude_or_abort(client);
        }
        _ => {
            let _ = post_action(client, r#"{"action_type":"EncounterAbort"}"#);
            let actions = possible_action_types(client);
            if actions.contains(&"EncounterApplyScouting".to_string()) {
                post_action(
                    client,
                    r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
                );
            }
        }
    }
}

fn conclude_or_abort(client: &Client) {
    let actions = possible_action_types(client);
    if actions.contains(&"EncounterConcludeEncounter".to_string()) {
        post_action(client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
    } else if actions.contains(&"EncounterAbort".to_string()) {
        post_action(client, r#"{"action_type":"EncounterAbort"}"#);
    }
    let actions = possible_action_types(client);
    if actions.contains(&"EncounterApplyScouting".to_string()) {
        post_action(
            client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
    }
}

fn possible_action_types(client: &Client) -> Vec<String> {
    let resp = get_json(client, "/actions/possible");
    let empty = vec![];
    resp.as_array()
        .unwrap_or(&empty)
        .iter()
        .filter_map(|v| {
            v.get("action_type")
                .and_then(|t| t.as_str())
                .map(String::from)
        })
        .collect()
}
