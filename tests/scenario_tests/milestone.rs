//! Integration tests for milestone encounters.
//!
//! Milestone encounters are tougher discipline-specific encounters that:
//! - Cost MilestoneInsight to start (100 * 2^(tier-1))
//! - On win -> auto-assigned next-tier encounter + 50% better CardEffect rewards
//! - On loss -> reset, return card to hand, back to NoEncounter

use super::helpers::*;
use my_little_cardgame::rocket_initialize;
use rocket::http::Status;
use rocket::local::blocking::Client;

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

const TOKENS: &str = include_str!("../configurations/tokens_default.json");
const TOKENS_LOW_HP: &str = include_str!("../configurations/tokens_low_health.json");
const SHARED: &str = include_str!("../configurations/shared_effects.json");
const MILESTONE_WIN: &str = include_str!("../configurations/milestone_combat_win.json");
const MILESTONE_LOSS: &str = include_str!("../configurations/milestone_combat_loss.json");

#[test]
fn scenario_milestone_combat_win_and_scout() {
    let client = create_test_client_from_json(
        42,
        TOKENS,
        &[("shared", SHARED), ("milestone", MILESTONE_WIN)],
    );

    let enc_ids = milestone_encounter_ids(&client);
    assert!(!enc_ids.is_empty(), "Should have milestone encounters");
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc_ids[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    // Inside milestone, play combat cards until resolved
    for _ in 0..100 {
        let enc = combat_state(&client);
        let outcome = enc
            .get("outcome")
            .and_then(|v| v.as_str())
            .unwrap_or("Undecided");
        if outcome != "Undecided" {
            break;
        }
        if !play_one_round(&client) {
            break;
        }
    }

    let result = combat_result(&client);
    assert_eq!(
        result.as_deref(),
        Some("PlayerWon"),
        "Milestone combat should win"
    );

    // Milestone win goes directly to NoEncounter (no scouting phase).
    // The milestone card is replaced in-place with next-tier milestone.
    let enc_after = milestone_encounter_ids(&client);
    assert!(
        !enc_after.is_empty(),
        "Should have next-tier milestone encounter after win"
    );

    // Verify we are in NoEncounter (can pick encounters)
    let state = combat_state(&client);
    assert!(
        state.get("encounter_state_type").is_none()
            || state.get("encounter_state_type").and_then(|v| v.as_str()) == Some("NoEncounter"),
        "Should be in NoEncounter after milestone win"
    );
}

#[test]
fn scenario_milestone_combat_loss_and_scout() {
    let client = create_test_client_from_json(
        42,
        TOKENS_LOW_HP,
        &[("shared", SHARED), ("milestone", MILESTONE_LOSS)],
    );

    let enc_ids = milestone_encounter_ids(&client);
    assert!(!enc_ids.is_empty());
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc_ids[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    // Play rounds — the enemy does 99999 damage → player dies
    for _ in 0..100 {
        let enc = combat_state(&client);
        let outcome = enc
            .get("outcome")
            .and_then(|v| v.as_str())
            .unwrap_or("Undecided");
        if outcome != "Undecided" {
            break;
        }
        if !play_one_round(&client) {
            break;
        }
    }

    let result = combat_result(&client);
    assert_eq!(
        result.as_deref(),
        Some("PlayerLost"),
        "Milestone combat should lose"
    );

    // Milestone loss goes directly to NoEncounter (no scouting phase).
    // The milestone card returns to hand, so we can pick it again.
    let enc_after = milestone_encounter_ids(&client);
    assert!(
        !enc_after.is_empty(),
        "Milestone card should return to hand on loss"
    );
    let pick2 = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc_after[0]
    );
    let (status, _) = post_action(&client, &pick2);
    assert_eq!(
        status,
        Status::Created,
        "Should re-pick milestone after loss"
    );
}

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

    for disc in &["Combat", "Mining", "Herbalism", "Woodcutting", "Fishing"] {
        assert!(
            milestone_encounter_by_discipline(&client, disc).is_some(),
            "Should have a {} milestone",
            disc
        );
    }
}

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

    assert!(
        !regular_encounters.is_empty(),
        "Should have regular (non-milestone) encounter cards"
    );
}

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

    let effects_before = count_player_effects_for_discipline(&client, "Combat");

    let milestone_id =
        milestone_encounter_by_discipline(&client, "Combat").expect("Should have combat milestone");
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        milestone_id
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created, "Pick combat milestone should work");

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

    let insight_after = player_token(&client, "MilestoneInsight");
    assert!(
        insight_after < insight,
        "MilestoneInsight should be deducted"
    );

    for _ in 0..200 {
        if !play_one_round(&client) {
            break;
        }
    }

    let actions = possible_action_types(&client);

    if actions.contains(&"EncounterPickEncounter".to_string())
        && !actions.contains(&"EncounterPlayCard".to_string())
    {
        let effects_after = count_player_effects_for_discipline(&client, "Combat");

        if effects_after > effects_before {
            let remaining = milestone_encounter_by_discipline(&client, "Combat");
            assert!(
                remaining.is_some(),
                "Should have an auto-assigned combat milestone (tier 2)"
            );

            let all_milestones = milestone_encounter_ids(&client);
            assert_eq!(
                all_milestones.len(),
                5,
                "Should have 5 milestones: 4 other disciplines + 1 auto-assigned combat tier 2"
            );

            assert!(
                milestone_encounter_by_discipline(&client, "Mining").is_some(),
                "Mining milestone should be unaffected"
            );
        } else {
            let milestone_still = milestone_encounter_by_discipline(&client, "Combat");
            assert!(
                milestone_still.is_some(),
                "Combat milestone should still be available after loss"
            );
        }
    }
}

#[test]
fn milestone_abort_treated_as_loss() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);

    win_combat(&client);

    let milestone_id =
        milestone_encounter_by_discipline(&client, "Combat").expect("Should have combat milestone");
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        milestone_id
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    let enc = combat_state(&client);
    assert_eq!(
        enc.get("encounter_state_type").and_then(|v| v.as_str()),
        Some("Milestone")
    );

    let (status, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    assert_eq!(status, Status::Created, "Abort should succeed");

    let actions = possible_action_types(&client);
    assert!(
        actions.contains(&"EncounterPickEncounter".to_string()),
        "Should be in NoEncounter after abort"
    );

    let milestone_still = milestone_encounter_by_discipline(&client, "Combat");
    assert!(
        milestone_still.is_some(),
        "Combat milestone should be returned to hand after abort"
    );
}

#[test]
fn milestone_max_hand_token() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);

    let max_hand = player_token(&client, "MilestoneMaxHand");
    assert_eq!(max_hand, 5, "MilestoneMaxHand should be initialized to 5");
}

#[test]
fn milestone_mining_encounter() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);

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

    let mining_cards = hand_card_ids_by_kind(&client, "Mining");
    if !mining_cards.is_empty() {
        let play = format!(
            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
            mining_cards[0]
        );
        let (status, _) = post_action(&client, &play);
        assert!(
            status == Status::Created || status == Status::BadRequest,
            "Mining card play should be handled"
        );
    }

    let (status, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    assert_eq!(status, Status::Created, "Abort should work");

    assert!(
        milestone_encounter_by_discipline(&client, "Mining").is_some(),
        "Mining milestone should be returned after abort"
    );
}

#[test]
fn milestone_tier_escalation() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);

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
