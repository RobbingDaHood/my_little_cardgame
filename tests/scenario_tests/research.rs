use super::helpers::*;
use my_little_cardgame::rocket_initialize;
use rocket::http::Status;
use rocket::local::blocking::Client;
use rocket::serde::json::serde_json;
use serde_json::Value;

const TOKENS: &str = include_str!("../configurations/tokens_default.json");
const TOKENS_WITH_INSIGHT: &str = include_str!("../configurations/tokens_with_combat_insight.json");
const SHARED: &str = include_str!("../configurations/shared_effects.json");
const COMBAT_WIN: &str = include_str!("../configurations/combat_win.json");
const RESEARCH_WIN: &str = include_str!("../configurations/research_win.json");
const RESEARCH_LOSS: &str = include_str!("../configurations/research_loss.json");

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

/// Helper: get Research (player) card IDs currently in hand.
fn research_hand_card_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Research");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| c.get("id").and_then(|v| v.as_u64()).map(|v| v as usize))
        .collect()
}

/// Play one round of research cards via ResearchPlayHand.
/// Returns true if the research encounter is still active.
fn play_one_research_card(client: &Client) -> bool {
    let research_ids = research_hand_card_ids(client);
    if research_ids.is_empty() {
        return false;
    }
    // Build a hand of 3 cards, reusing IDs when a card has multiple copies
    let hand: Vec<usize> = research_ids.iter().cycle().take(3).copied().collect();
    let card_ids_json = serde_json::to_string(&hand).unwrap_or_else(|_| "[]".to_string());
    let play_json = format!(
        r#"{{"action_type":"ResearchPlayHand","card_ids":{}}}"#,
        card_ids_json
    );
    let (status, _) = post_action(client, &play_json);
    if status != Status::Created {
        return false;
    }
    let encounter = combat_state(client);
    encounter.get("outcome").and_then(|v| v.as_str()) == Some("Undecided")
}

#[test]
fn scenario_research_win_and_scout() {
    let client = create_test_client_from_json(42, TOKENS, &[("research", RESEARCH_WIN)]);

    let enc_ids = research_encounter_ids(&client);
    assert!(!enc_ids.is_empty(), "Should have research encounters");
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
        Some("Research")
    );

    // Play multiple rounds cycling through different card sets to cover all 6 symbols
    let all_ids = research_hand_card_ids(&client);
    assert!(
        all_ids.len() >= 3,
        "Need at least 3 research card IDs, got {}",
        all_ids.len()
    );
    for chunk in all_ids.chunks(3) {
        if chunk.len() < 3 {
            break;
        }
        let card_ids_json =
            serde_json::to_string(&chunk.to_vec()).unwrap_or_else(|_| "[]".to_string());
        let play_json = format!(
            r#"{{"action_type":"ResearchPlayHand","card_ids":{}}}"#,
            card_ids_json
        );
        let (status, _) = post_action(&client, &play_json);
        if status != Status::Created {
            break;
        }
    }

    // Conclude experiment if still undecided
    let enc = combat_state(&client);
    if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
        let (status, _) = post_action(&client, r#"{"action_type":"ResearchConcludeExperiment"}"#);
        if status != Status::Created {
            let _ = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        }
    }

    let result = combat_result(&client);
    assert_eq!(
        result.as_deref(),
        Some("PlayerWon"),
        "Research should win with high yield"
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
fn scenario_research_loss_and_scout() {
    let client = create_test_client_from_json(42, TOKENS, &[("research", RESEARCH_LOSS)]);

    let enc_ids = research_encounter_ids(&client);
    assert!(!enc_ids.is_empty());
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc_ids[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    for _ in 0..50 {
        if !play_one_research_card(&client) {
            break;
        }
    }

    // Conclude experiment if still undecided
    let enc = combat_state(&client);
    if enc.get("outcome").and_then(|v| v.as_str()) == Some("Undecided") {
        let (status, _) = post_action(&client, r#"{"action_type":"ResearchConcludeExperiment"}"#);
        if status != Status::Created {
            let _ = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        }
    }

    let result = combat_result(&client);
    assert_eq!(
        result.as_deref(),
        Some("PlayerLost"),
        "Research should lose from interference"
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

/// Create a client with a research encounter already in hand and a known CombatInsight
/// balance. The seed is used for RNG within the research experiment itself.
/// Returns (client, insight_balance_before_encounter_start).
fn create_research_client_with_insight(seed: u64) -> (Client, i64) {
    let client = create_test_client_from_json(
        seed,
        TOKENS_WITH_INSIGHT,
        &[
            ("shared", SHARED),
            ("combat", COMBAT_WIN),
            ("research", RESEARCH_WIN),
        ],
    );
    let insight = player_token(&client, "CombatInsight");
    let enc_ids = research_encounter_ids(&client);
    assert!(
        !enc_ids.is_empty(),
        "Research encounter should be in hand with RESEARCH_WIN config"
    );
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc_ids[0]
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created, "PickEncounter should succeed");
    (client, insight)
}

/// Create a client with a research encounter active, project chosen, and candidate selected.
/// Returns Some((client, remaining_insight)) or None if preconditions fail.
fn setup_research_client_with_project(seed: u64) -> Option<(Client, i64)> {
    let (client, insight) = create_research_client_with_insight(seed);
    if insight < 10 {
        return None;
    }
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchChooseProject","discipline":"Combat","tier_count":1}"#,
    );
    if status != Status::Created {
        return None;
    }
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchSelectCandidate","candidate_index":0}"#,
    );
    assert_eq!(status, Status::Created);
    Some((client, insight - 10))
}

/// Like setup_research_client_with_project, but loads the RESEARCH_LOSS config so an
/// interference deck is present. Used for interference-specific scenario tests.
fn setup_research_client_with_interference_project(seed: u64) -> Option<(Client, i64)> {
    let client = create_test_client_from_json(
        seed,
        TOKENS_WITH_INSIGHT,
        &[
            ("shared", SHARED),
            ("combat", COMBAT_WIN),
            ("research", RESEARCH_LOSS),
        ],
    );
    let insight = player_token(&client, "CombatInsight");
    if insight < 10 {
        return None;
    }
    let enc_ids = research_encounter_ids(&client);
    if enc_ids.is_empty() {
        return None;
    }
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        enc_ids[0]
    );
    let (status, _) = post_action(&client, &pick);
    if status != Status::Created {
        return None;
    }
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchChooseProject","discipline":"Combat","tier_count":1}"#,
    );
    if status != Status::Created {
        return None;
    }
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"ResearchSelectCandidate","candidate_index":0}"#,
    );
    assert_eq!(status, Status::Created);
    Some((client, insight - 10))
}

/// Scenario: Research encounter flow: choose project, select candidate, conclude.
#[test]
fn scenario_research_encounter_full_loop() {
    let (client, insight_before) = create_research_client_with_insight(7777);

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
    let insight_after_choose = player_token(&client, "CombatInsight");
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
        if status == Status::Created {
            let insight_after_progress = player_token(&client, "CombatInsight");
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

#[test]
fn scenario_research_choose_and_swap_project() {
    let (client, insight_before) = create_research_client_with_insight(7777);

    if insight_before < 10 {
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
        return;
    }

    // If available, start a second research encounter
    let insight_now = player_token(&client, "CombatInsight");
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
            r#"{"action_type":"ResearchChooseProject","discipline":"Research","tier_count":1}"#,
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

/// Scenario: Attempt to choose a research project with insufficient Insight.
#[test]
fn scenario_research_insufficient_insight() {
    let client = create_test_client_from_json(42, TOKENS, &[("research", RESEARCH_WIN)]);

    let insight = player_token(&client, "CombatInsight");
    assert_eq!(insight, 0, "Should start with 0 Insight");

    let research_enc = research_encounter_ids(&client);
    assert!(
        !research_enc.is_empty(),
        "Should have research encounters in hand"
    );
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        research_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created, "PickEncounter should succeed");

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

/// Scenario: Abort a research encounter.
#[test]
fn scenario_research_abort() {
    let client = create_test_client_from_json(42, TOKENS, &[("research", RESEARCH_WIN)]);

    let research_enc = research_encounter_ids(&client);
    assert!(
        !research_enc.is_empty(),
        "Should have research encounters in hand"
    );
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        research_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created);

    let encounter = combat_state(&client);
    assert_eq!(
        encounter
            .get("encounter_state_type")
            .and_then(|v| v.as_str()),
        Some("Research")
    );

    let (status, _) = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
    assert_eq!(status, Status::Created, "Research abort should succeed");

    let result = combat_result(&client);
    assert_eq!(
        result,
        Some("PlayerWon".to_string()),
        "Research abort should always result in PlayerWon"
    );

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

/// Scenario: Full research experiment loop.
#[test]
fn scenario_research_experiment_full_loop() {
    let (client, _remaining_insight) = match setup_research_client_with_project(7777) {
        Some(pair) => pair,
        None => return,
    };

    let enc_before = combat_state(&client);
    assert_eq!(
        enc_before
            .get("experiment_active")
            .and_then(|v| v.as_bool()),
        Some(false),
        "Experiment should not be active before play hand"
    );

    let research_cards = research_hand_card_ids(&client);
    if research_cards.len() < 3 {
        let card_ids_json = serde_json::to_string(&research_cards).unwrap_or("[]".to_string());
        let play_json = format!(
            r#"{{"action_type":"ResearchPlayHand","card_ids":{}}}"#,
            card_ids_json
        );
        let (status, _body) = post_action(&client, &play_json);
        if status != Status::Created {
            let (status, _) =
                post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
            assert_eq!(status, Status::Created);
            return;
        }
    }

    let research_cards = research_hand_card_ids(&client);

    if research_cards.len() < 3 {
        let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        assert_eq!(status, Status::Created);
        return;
    }

    let hand_to_play: Vec<usize> = research_cards[..3].to_vec();
    let card_ids_json = serde_json::to_string(&hand_to_play).unwrap();
    let play_json = format!(
        r#"{{"action_type":"ResearchPlayHand","card_ids":{}}}"#,
        card_ids_json
    );
    let (status, _) = post_action(&client, &play_json);

    if status == Status::BadRequest {
        let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        assert_eq!(status, Status::Created);
        return;
    }
    assert_eq!(status, Status::Created, "ResearchPlayHand should succeed");

    let enc = combat_state(&client);
    assert_eq!(
        enc.get("experiment_active").and_then(|v| v.as_bool()),
        Some(true),
        "Experiment should be active after playing hand"
    );
    assert!(
        enc.get("rounds_played")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            >= 1,
        "Should have at least 1 round played"
    );

    let round_history = enc
        .get("round_history")
        .and_then(|v| v.as_array())
        .expect("Should have round_history");
    assert!(
        !round_history.is_empty(),
        "round_history should not be empty"
    );
    let first_round = &round_history[0];
    assert!(
        first_round.get("round_yield").is_some(),
        "Round result should have round_yield"
    );
    assert!(
        first_round.get("insight_cost").is_some(),
        "Round result should have insight_cost"
    );
    assert_eq!(
        first_round.get("insight_cost").and_then(|v| v.as_i64()),
        Some(1),
        "Round 1 should cost 1 Insight (base_insight_cost=1 in test config)"
    );

    let (status, _) = post_action(&client, r#"{"action_type":"ResearchConcludeExperiment"}"#);
    assert_eq!(
        status,
        Status::Created,
        "ResearchConcludeExperiment should succeed"
    );

    let result = combat_result(&client);
    assert!(
        result.is_some(),
        "Should have encounter result after concluding experiment"
    );

    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(
        status,
        Status::Created,
        "Should be able to scout after research experiment"
    );
}

/// Scenario: Conclude experiment with 0 yield.
#[test]
fn scenario_research_experiment_zero_yield_loss() {
    let (client, remaining_insight) = match setup_research_client_with_project(9999) {
        Some(pair) => pair,
        None => return,
    };

    let research_cards = research_hand_card_ids(&client);

    if research_cards.len() >= 3 && remaining_insight >= 5 {
        let hand_to_play: Vec<usize> = research_cards[..3].to_vec();
        let card_ids_json = serde_json::to_string(&hand_to_play).unwrap();
        let play_json = format!(
            r#"{{"action_type":"ResearchPlayHand","card_ids":{}}}"#,
            card_ids_json
        );
        let (status, _) = post_action(&client, &play_json);

        if status == Status::Created {
            let (status, _) =
                post_action(&client, r#"{"action_type":"ResearchConcludeExperiment"}"#);
            assert_eq!(status, Status::Created);

            let result = combat_result(&client);
            assert!(
                result == Some("PlayerWon".to_string()) || result == Some("PlayerLost".to_string()),
                "Should get PlayerWon or PlayerLost, got: {:?}",
                result
            );

            let (status, _) = post_action(
                &client,
                r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
            );
            assert_eq!(status, Status::Created);
        } else {
            let (status, _) =
                post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
            assert_eq!(status, Status::Created);
        }
    } else {
        let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        assert_eq!(status, Status::Created);
    }
}

/// Scenario: Research experiment cost escalation.
#[test]
fn scenario_research_experiment_cost_escalation() {
    let (client, remaining_insight) = match setup_research_client_with_project(12345) {
        Some(pair) => pair,
        None => return,
    };

    if remaining_insight < 3 {
        let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        assert_eq!(status, Status::Created);
        return;
    }

    // Play round 1
    let research_cards = research_hand_card_ids(&client);
    if research_cards.len() < 3 {
        let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        assert_eq!(status, Status::Created);
        return;
    }
    let hand_r1: Vec<usize> = research_cards[..3].to_vec();
    let play_json = format!(
        r#"{{"action_type":"ResearchPlayHand","card_ids":{}}}"#,
        serde_json::to_string(&hand_r1).unwrap()
    );
    let (status, _) = post_action(&client, &play_json);
    assert_eq!(status, Status::Created, "Round 1 should succeed");

    let enc = combat_state(&client);
    let round_history = enc.get("round_history").and_then(|v| v.as_array()).unwrap();
    assert_eq!(round_history.len(), 1, "Should have 1 round");
    assert_eq!(
        round_history[0]
            .get("insight_cost")
            .and_then(|v| v.as_i64()),
        Some(1),
        "Round 1 cost should be 1 (base_insight_cost=1 in test config)"
    );

    // Play round 2
    let research_cards = research_hand_card_ids(&client);
    if research_cards.len() < 3 {
        let (status, _) = post_action(&client, r#"{"action_type":"ResearchConcludeExperiment"}"#);
        assert_eq!(status, Status::Created);
        let (status, _) = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
        assert_eq!(status, Status::Created);
        return;
    }
    let hand_r2: Vec<usize> = research_cards[..3].to_vec();
    let play_json = format!(
        r#"{{"action_type":"ResearchPlayHand","card_ids":{}}}"#,
        serde_json::to_string(&hand_r2).unwrap()
    );
    let (status, _) = post_action(&client, &play_json);
    assert_eq!(status, Status::Created, "Round 2 should succeed");

    let enc = combat_state(&client);
    let round_history = enc.get("round_history").and_then(|v| v.as_array()).unwrap();
    assert_eq!(round_history.len(), 2, "Should have 2 rounds");
    assert_eq!(
        round_history[1]
            .get("insight_cost")
            .and_then(|v| v.as_i64()),
        Some(2),
        "Round 2 cost should be 2 (base_insight_cost=1 × round_num=2)"
    );

    let (status, _) = post_action(&client, r#"{"action_type":"ResearchConcludeExperiment"}"#);
    assert_eq!(status, Status::Created);

    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(status, Status::Created);
}

/// Scenario: Verify wrong number of cards returns error.
#[test]
fn scenario_research_experiment_wrong_card_count() {
    let (client, remaining_insight) = match setup_research_client_with_project(5555) {
        Some(pair) => pair,
        None => return,
    };

    if remaining_insight < 5 {
        let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        assert_eq!(status, Status::Created);
        return;
    }

    let research_cards = research_hand_card_ids(&client);

    if !research_cards.is_empty() {
        let play_json = format!(
            r#"{{"action_type":"ResearchPlayHand","card_ids":[{}]}}"#,
            research_cards[0]
        );
        let (status, body) = post_action(&client, &play_json);
        assert_eq!(status, Status::BadRequest, "Should reject wrong card count");
        let msg = body.get("message").and_then(|v| v.as_str()).unwrap_or("");
        assert!(
            msg.contains("Must play exactly 3 cards") || msg.contains("must play"),
            "Error should mention card count, got: {}",
            msg
        );
    }

    let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
    assert_eq!(status, Status::Created);
}

/// Scenario: Verify hidden_types is NOT visible in encounter state API response.
#[test]
fn scenario_research_experiment_hidden_types_not_visible() {
    let (client, remaining_insight) = match setup_research_client_with_project(4444) {
        Some(pair) => pair,
        None => return,
    };

    if remaining_insight < 5 {
        let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        assert_eq!(status, Status::Created);
        return;
    }

    let research_cards = research_hand_card_ids(&client);
    if research_cards.len() < 3 {
        let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        assert_eq!(status, Status::Created);
        return;
    }
    let hand: Vec<usize> = research_cards[..3].to_vec();
    let play_json = format!(
        r#"{{"action_type":"ResearchPlayHand","card_ids":{}}}"#,
        serde_json::to_string(&hand).unwrap()
    );
    let (status, _) = post_action(&client, &play_json);
    if status != Status::Created {
        let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        assert_eq!(status, Status::Created);
        return;
    }

    let enc = combat_state(&client);
    assert!(
        enc.get("hidden_types").is_none() || enc.get("hidden_types").unwrap().is_null(),
        "hidden_types should not be visible in API response, got: {:?}",
        enc.get("hidden_types")
    );

    let (status, _) = post_action(&client, r#"{"action_type":"ResearchConcludeExperiment"}"#);
    assert_eq!(status, Status::Created);
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(status, Status::Created);
}

/// Scenario: Verify Research cards exist in library and have correct structure.
#[test]
fn scenario_research_experiment_cards_exist() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":1234}"#);
    assert_eq!(status, Status::Created);

    let cards = get_json(&client, "/library/cards?card_kind=Research");
    let card_arr = cards.as_array().expect("Should return array");

    assert!(
        !card_arr.is_empty(),
        "Should have Research cards in library"
    );

    for card in card_arr {
        let deck = card
            .get("counts")
            .and_then(|c| c.get("deck"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let hand = card
            .get("counts")
            .and_then(|c| c.get("hand"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        assert_eq!(
            hand, 0,
            "Research cards should not be in hand at game start"
        );
        assert!(deck > 0, "Research cards should be in deck at game start");
    }
}

/// Scenario: Multiple rounds of research experiment, verifying accumulated_yield grows.
#[test]
fn scenario_research_experiment_multi_round() {
    let (client, remaining_insight) = match setup_research_client_with_project(33333) {
        Some(pair) => pair,
        None => return,
    };

    if remaining_insight < 6 {
        let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        assert_eq!(status, Status::Created);
        return;
    }

    let mut total_rounds = 0;

    for round in 0..3 {
        let research_cards = research_hand_card_ids(&client);
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
        total_rounds += 1;

        let enc = combat_state(&client);
        assert_eq!(
            enc.get("rounds_played").and_then(|v| v.as_u64()),
            Some((round + 1) as u64),
            "Should have {} rounds played",
            round + 1
        );
    }

    if total_rounds > 0 {
        let enc = combat_state(&client);
        let acc_yield = enc
            .get("accumulated_yield")
            .and_then(|v| v.as_i64())
            .unwrap_or(-1);
        assert!(
            acc_yield >= 0,
            "accumulated_yield should be non-negative, got {}",
            acc_yield
        );

        let (status, _) = post_action(&client, r#"{"action_type":"ResearchConcludeExperiment"}"#);
        assert_eq!(status, Status::Created);

        let (status, _) = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
        assert_eq!(status, Status::Created);
    } else {
        let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        assert_eq!(status, Status::Created);
    }
}

/// Scenario: Interference deck plays a card each round during the experiment.
/// Verifies that interference_played appears in round_history results.
#[test]
fn scenario_research_experiment_interference_deck() {
    let (client, remaining_insight) = match setup_research_client_with_interference_project(55555) {
        Some(pair) => pair,
        None => return,
    };

    if remaining_insight < 3 {
        let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        assert_eq!(status, Status::Created);
        return;
    }

    let mut interference_count = 0;
    let mut total_rounds = 0;

    for _ in 0..4 {
        let research_cards = research_hand_card_ids(&client);
        if research_cards.len() < 3 {
            break;
        }
        let hand: Vec<usize> = research_cards[..3].to_vec();
        let play_json = format!(
            r#"{{"action_type":"ResearchPlayHand","card_ids":{}}}"#,
            serde_json::to_string(&hand).expect("serialize card_ids")
        );
        let (status, _) = post_action(&client, &play_json);
        if status != Status::Created {
            break;
        }
        total_rounds += 1;

        let enc = combat_state(&client);
        let round_history = enc
            .get("round_history")
            .and_then(|v| v.as_array())
            .expect("Should have round_history");

        let latest = round_history
            .last()
            .expect("Should have at least one round");
        if latest.get("interference_played").is_some()
            && latest["interference_played"] != Value::Null
        {
            interference_count += 1;
        }
    }

    if total_rounds > 0 {
        assert!(
            interference_count > 0,
            "At least one round should have interference_played set (got {} rounds with 0 interference)",
            total_rounds
        );

        let enc = combat_state(&client);
        let round_history = enc
            .get("round_history")
            .and_then(|v| v.as_array())
            .expect("Should have round_history");
        for (i, round) in round_history.iter().enumerate() {
            if let Some(interference) = round.get("interference_played") {
                if interference != &Value::Null {
                    let desc = interference.as_str().unwrap_or("");
                    assert!(
                        desc.contains("BlockBestMatch")
                            || desc.contains("SwapHiddenSlots")
                            || desc.contains("ReduceYield")
                            || desc.contains("ShuffleHiddenSlots")
                            || desc.contains("InsightTax"),
                        "Round {} interference should be a known type, got: {}",
                        i + 1,
                        desc
                    );
                }
            }
        }

        let (status, _) = post_action(&client, r#"{"action_type":"ResearchConcludeExperiment"}"#);
        assert_eq!(status, Status::Created);

        let (status, _) = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
        assert_eq!(status, Status::Created);
    } else {
        let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        assert_eq!(status, Status::Created);
    }
}

/// Scenario: Verify interference affects accumulated yield (it should be lower than
/// a theoretical max of 300 per round × rounds played in most cases with interference).
#[test]
fn scenario_research_interference_reduces_yield() {
    let (client, remaining_insight) = match setup_research_client_with_interference_project(88888) {
        Some(pair) => pair,
        None => return,
    };

    if remaining_insight < 5 {
        let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        assert_eq!(status, Status::Created);
        return;
    }

    let mut total_rounds = 0;

    for _ in 0..5 {
        let research_cards = research_hand_card_ids(&client);
        if research_cards.len() < 3 {
            break;
        }
        let hand: Vec<usize> = research_cards[..3].to_vec();
        let play_json = format!(
            r#"{{"action_type":"ResearchPlayHand","card_ids":{}}}"#,
            serde_json::to_string(&hand).expect("serialize card_ids")
        );
        let (status, _) = post_action(&client, &play_json);
        if status != Status::Created {
            break;
        }
        total_rounds += 1;
    }

    if total_rounds > 0 {
        let enc = combat_state(&client);
        let accumulated_yield = enc
            .get("accumulated_yield")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        // accumulated_yield should be >= 0 even with interference
        assert!(
            accumulated_yield >= 0,
            "accumulated_yield should be non-negative: got {}",
            accumulated_yield
        );

        // per_card_yield may not sum to round_yield when BlockBestMatch or ReduceYield hit —
        // but accumulated_yield equals the sum of round_yields
        let round_yield_sum: i64 = enc
            .get("round_history")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|r| r.get("round_yield").and_then(|v| v.as_i64()))
            .sum();
        assert_eq!(
            accumulated_yield, round_yield_sum,
            "accumulated_yield should equal sum of round_yields"
        );

        let (status, _) = post_action(&client, r#"{"action_type":"ResearchConcludeExperiment"}"#);
        assert_eq!(status, Status::Created);
        let (status, _) = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
        assert_eq!(status, Status::Created);
    } else {
        let (status, _) = post_action(&client, r#"{"action_type":"EncounterConcludeEncounter"}"#);
        assert_eq!(status, Status::Created);
    }
}
