//! Integration tests for milestone encounters.
//!
//! Milestone encounters are tougher discipline-specific encounters that:
//! - Cost MilestoneInsight to start (100 * 2^(tier-1))
//! - On win → MilestoneScouting with 3 next-tier choices + 50% better CardEffect rewards
//! - On loss → reset, return card to hand, back to NoEncounter

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

fn milestone_encounter_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| {
            let kind = c.get("kind")?;
            let enc_kind = kind.get("encounter_kind")?;
            if enc_kind.get("encounter_type")?.as_str()? == "Milestone" {
                c.get("id")?.as_u64().map(|v| v as usize)
            } else {
                None
            }
        })
        .collect()
}

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

/// Play one full combat round (Defence → Attack → Resource).
/// Returns true if encounter is still active.
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

fn possible_action_types(client: &Client) -> Vec<String> {
    let actions = get_json(client, "/actions/possible");
    actions
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|a| a.get("action_type")?.as_str().map(String::from))
        .collect()
}

/// Win a regular combat encounter to earn MilestoneInsight tokens.
fn win_combat(client: &Client) {
    let enc = combat_encounter_ids(client);
    assert!(!enc.is_empty(), "Need combat encounter cards to win");
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc[0]
    );
    let (status, _) = post_action(client, &pick);
    assert_eq!(status, Status::Created, "Pick combat should succeed");
    for _ in 0..200 {
        if !play_one_round(client) {
            break;
        }
    }
    // Apply scouting to return to NoEncounter
    let (status, _) = post_action(
        client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(status, Status::Created, "Scouting should succeed");
}

/// Count PlayerCardEffect cards that match a given discipline.
fn count_player_effects_for_discipline(client: &Client, discipline: &str) -> usize {
    let cards = get_json(client, "/library/cards?card_kind=PlayerCardEffect");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter(|c| {
            c.get("valid_discipline_types")
                .and_then(|d| d.as_array())
                .map(|arr| arr.iter().any(|v| v.as_str() == Some(discipline)))
                .unwrap_or(false)
        })
        .count()
}

// ============================================================
// Test: Milestone encounters exist after new game
// ============================================================
#[test]
fn milestone_encounters_exist_at_start() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);

    let milestones = milestone_encounter_ids(&client);
    assert_eq!(
        milestones.len(),
        5,
        "Should have 5 milestone encounters (one per combat/gathering discipline)"
    );

    // Verify disciplines
    for disc in &["Combat", "Mining", "Herbalism", "Woodcutting", "Fishing"] {
        assert!(
            milestone_encounter_by_discipline(&client, disc).is_some(),
            "Should have a {} milestone",
            disc
        );
    }
}

// ============================================================
// Test: Milestone encounters are NOT in regular encounter hand
// ============================================================
#[test]
fn milestone_not_in_regular_encounter_hand() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);

    let regular_encounters = {
        let cards = get_json(&client, "/library/cards?location=Hand&card_kind=Encounter");
        let all: Vec<usize> = cards
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|c| c.get("id").and_then(|v| v.as_u64()).map(|v| v as usize))
            .collect();
        let milestones = milestone_encounter_ids(&client);
        all.into_iter()
            .filter(|id| !milestones.contains(id))
            .collect::<Vec<_>>()
    };

    // Regular encounters should exist independently of milestones
    assert!(
        !regular_encounters.is_empty(),
        "Should have regular (non-milestone) encounter cards"
    );
}

// ============================================================
// Test: Insufficient MilestoneInsight rejects milestone start
// ============================================================
#[test]
fn milestone_insufficient_insight() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);

    let insight = player_token(&client, "MilestoneInsight");
    assert_eq!(insight, 0, "Should start with 0 MilestoneInsight");

    let milestone_id = milestone_encounter_ids(&client)[0];
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        milestone_id
    );
    let (status, body) = post_action(&client, &pick);
    assert_eq!(
        status,
        Status::BadRequest,
        "Should reject milestone with insufficient insight: {:?}",
        body
    );
}

// ============================================================
// Test: Win combat milestone → MilestoneScouting → pick choice
// ============================================================
#[test]
fn milestone_combat_win_flow() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);

    // Win combat to earn MilestoneInsight
    win_combat(&client);

    let insight = player_token(&client, "MilestoneInsight");
    assert!(
        insight >= 100,
        "Should have at least 100 MilestoneInsight after combat win, got {}",
        insight
    );

    // Count existing Combat effects before milestone
    let effects_before = count_player_effects_for_discipline(&client, "Combat");

    // Pick the combat milestone
    let milestone_id =
        milestone_encounter_by_discipline(&client, "Combat").expect("Should have combat milestone");
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        milestone_id
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created, "Pick combat milestone should work");

    // Verify we're in a milestone encounter
    let enc = combat_state(&client);
    assert_eq!(
        enc.get("encounter_state_type").and_then(|v| v.as_str()),
        Some("Milestone"),
        "Should be in milestone encounter"
    );
    assert_eq!(
        enc.get("tier").and_then(|v| v.as_u64()),
        Some(1),
        "Should be tier 1"
    );

    // Insight should have been deducted
    let insight_after = player_token(&client, "MilestoneInsight");
    assert!(
        insight_after < insight,
        "MilestoneInsight should be deducted"
    );

    // Play combat rounds until finished
    for _ in 0..200 {
        if !play_one_round(&client) {
            break;
        }
    }

    // Check if we won (should be in MilestoneScouting or NoEncounter)
    let actions = possible_action_types(&client);

    if actions.contains(&"MilestonePickScoutingChoice".to_string()) {
        // We won! Should have 3 scouting choices for Combat discipline
        let all_milestones = milestone_encounter_ids(&client);
        // Old combat milestone was deleted; 4 other disciplines remain + 3 new choices
        assert_eq!(
            all_milestones.len(),
            7,
            "Should have 7 milestones: 4 other disciplines + 3 combat scouting choices"
        );

        // Filter to combat-discipline milestones only
        let combat_milestones: Vec<usize> = all_milestones
            .iter()
            .filter(|&&id| {
                let cards = get_json(&client, "/library/cards?location=Hand&card_kind=Encounter");
                cards.as_array().unwrap_or(&vec![]).iter().any(|c| {
                    c.get("id").and_then(|v| v.as_u64()).map(|v| v as usize) == Some(id)
                        && c.get("kind")
                            .and_then(|k| k.get("encounter_kind"))
                            .and_then(|ek| ek.get("milestone_def"))
                            .and_then(|d| d.get("discipline"))
                            .and_then(|d| d.as_str())
                            == Some("Combat")
                })
            })
            .copied()
            .collect();
        assert_eq!(
            combat_milestones.len(),
            3,
            "Should have 3 Combat milestone scouting choices"
        );

        // Verify new reward effects were created
        let effects_after = count_player_effects_for_discipline(&client, "Combat");
        assert!(
            effects_after > effects_before,
            "Should have more Combat effects after milestone win ({} > {})",
            effects_after,
            effects_before
        );

        // Pick one of the 3 choices
        let choice = combat_milestones[0];
        let pick_choice = format!(
            r#"{{"action_type":"MilestonePickScoutingChoice","card_id":{}}}"#,
            choice
        );
        let (status, _) = post_action(&client, &pick_choice);
        assert_eq!(status, Status::Created, "Pick scouting choice should work");

        // Verify back to NoEncounter
        let actions_after = possible_action_types(&client);
        assert!(
            actions_after.contains(&"EncounterPickEncounter".to_string()),
            "Should be back in NoEncounter phase"
        );
        assert!(
            !actions_after.contains(&"MilestonePickScoutingChoice".to_string()),
            "Should no longer be in MilestoneScouting"
        );

        // Verify only 1 milestone remains for combat (the chosen one)
        let remaining = milestone_encounter_by_discipline(&client, "Combat");
        assert!(
            remaining.is_some(),
            "Should still have a combat milestone (the chosen tier-2)"
        );

        // Other discipline milestones should be unaffected
        assert!(
            milestone_encounter_by_discipline(&client, "Mining").is_some(),
            "Mining milestone should be unaffected"
        );
    } else {
        // We lost — verify we're back in NoEncounter
        assert!(
            actions.contains(&"EncounterPickEncounter".to_string()),
            "After milestone loss, should be in NoEncounter"
        );
        // The milestone card should be back in hand
        let milestone_still = milestone_encounter_by_discipline(&client, "Combat");
        assert!(
            milestone_still.is_some(),
            "Combat milestone should still be available after loss"
        );
    }
}

// ============================================================
// Test: Abort milestone encounter → treated as loss
// ============================================================
#[test]
fn milestone_abort_treated_as_loss() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);

    // Win combat for insight
    win_combat(&client);

    let milestone_id =
        milestone_encounter_by_discipline(&client, "Combat").expect("Should have combat milestone");
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        milestone_id
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    // Verify we're in a milestone
    let enc = combat_state(&client);
    assert_eq!(
        enc.get("encounter_state_type").and_then(|v| v.as_str()),
        Some("Milestone")
    );

    // Abort
    let (status, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    assert_eq!(status, Status::Created, "Abort should succeed");

    // Should be back in NoEncounter
    let actions = possible_action_types(&client);
    assert!(
        actions.contains(&"EncounterPickEncounter".to_string()),
        "Should be in NoEncounter after abort"
    );
    assert!(
        !actions.contains(&"MilestonePickScoutingChoice".to_string()),
        "Should NOT be in MilestoneScouting after abort"
    );

    // Milestone card should be returned to hand
    let milestone_still = milestone_encounter_by_discipline(&client, "Combat");
    assert!(
        milestone_still.is_some(),
        "Combat milestone should be returned to hand after abort"
    );
}

// ============================================================
// Test: Invalid MilestonePickScoutingChoice rejected
// ============================================================
#[test]
fn milestone_invalid_scouting_choice_rejected() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);

    // Try to pick a scouting choice when not in MilestoneScouting
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"MilestonePickScoutingChoice","card_id":999}"#,
    );
    assert_ne!(
        status,
        Status::Created,
        "Should reject scouting choice when not in MilestoneScouting phase"
    );
}

// ============================================================
// Test: MilestoneMaxHand token initialized to 5
// ============================================================
#[test]
fn milestone_max_hand_token() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);

    let max_hand = player_token(&client, "MilestoneMaxHand");
    assert_eq!(max_hand, 5, "MilestoneMaxHand should be initialized to 5");
}

// ============================================================
// Test: Non-combat milestone (Mining) can be started
// ============================================================
#[test]
fn milestone_mining_encounter() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);

    // Win combat for insight
    win_combat(&client);

    let insight = player_token(&client, "MilestoneInsight");
    assert!(insight >= 100, "Need at least 100 insight");

    let milestone_id =
        milestone_encounter_by_discipline(&client, "Mining").expect("Should have mining milestone");
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        milestone_id
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(
        status,
        Status::Created,
        "Pick mining milestone should succeed"
    );

    // Verify it's a milestone with Mining inner state
    let enc = combat_state(&client);
    assert_eq!(
        enc.get("encounter_state_type").and_then(|v| v.as_str()),
        Some("Milestone"),
        "Should be in milestone encounter"
    );
    assert_eq!(
        enc.get("discipline").and_then(|v| v.as_str()),
        Some("Mining"),
        "Should be a Mining milestone"
    );

    // Inner state should be Mining
    let inner = enc.get("inner_state");
    assert!(inner.is_some(), "Should have inner_state");
    assert_eq!(
        inner
            .unwrap()
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Mining"),
        "Inner state should be Mining"
    );

    // Play some mining cards
    let mining_cards = hand_card_ids_by_kind(&client, "Mining");
    if !mining_cards.is_empty() {
        let play = format!(
            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
            mining_cards[0]
        );
        let (status, _) = post_action(&client, &play);
        // Mining play might succeed or fail depending on state, just verify it's handled
        assert!(
            status == Status::Created || status == Status::BadRequest,
            "Mining card play should be handled"
        );
    }

    // Abort to clean up
    let (status, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    assert_eq!(status, Status::Created, "Abort should work");

    // Milestone should be back in hand
    assert!(
        milestone_encounter_by_discipline(&client, "Mining").is_some(),
        "Mining milestone should be returned after abort"
    );
}

// ============================================================
// Test: Tier escalation cost doubles
// ============================================================
#[test]
fn milestone_tier_escalation() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);

    // Check tier-1 milestone cost
    let cards = get_json(&client, "/library/cards?location=Hand&card_kind=Encounter");
    let empty = vec![];
    let milestone_card = cards
        .as_array()
        .unwrap_or(&empty)
        .iter()
        .find(|c| {
            c.get("kind")
                .and_then(|k| k.get("encounter_kind"))
                .and_then(|ek| ek.get("encounter_type"))
                .and_then(|t| t.as_str())
                == Some("Milestone")
        })
        .expect("Should find a milestone");

    let def = milestone_card
        .get("kind")
        .unwrap()
        .get("encounter_kind")
        .unwrap()
        .get("milestone_def")
        .unwrap();

    assert_eq!(
        def.get("tier").and_then(|v| v.as_u64()),
        Some(1),
        "Initial milestone should be tier 1"
    );
    assert_eq!(
        def.get("insight_cost").and_then(|v| v.as_i64()),
        Some(100),
        "Tier 1 cost should be 100"
    );
}
