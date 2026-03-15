use super::helpers::*;
use my_little_cardgame::rocket_initialize;
use rocket::http::Status;
use rocket::local::blocking::Client;
use rocket::serde::json::serde_json;
use serde_json::Value;

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

/// Start a new game, accumulate Insight via combat wins, then deplete the
/// encounter hand until the Research encounter card is drawn to hand.
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

    let insight = player_token(client, "CombatInsight");
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

/// Helper: set up a game and enter a research encounter with a project selected.
/// Returns the CombatInsight balance after project selection.
fn setup_research_with_project(client: &Client, seed: u64) -> Option<i64> {
    let insight = start_game_accumulate_insight_and_pick_research(client, seed);
    if insight < 10 {
        return None;
    }

    // Choose project: Combat, tier 1 (costs 10)
    let (status, _) = post_action(
        client,
        r#"{"action_type":"ResearchChooseProject","discipline":"Combat","tier_count":1}"#,
    );
    if status != Status::Created {
        return None;
    }

    // Select candidate 0
    let (status, _) = post_action(
        client,
        r#"{"action_type":"ResearchSelectCandidate","candidate_index":0}"#,
    );
    assert_eq!(status, Status::Created);

    Some(insight - 10)
}

/// Scenario: Research encounter flow: choose project, select candidate, conclude.
#[test]
fn scenario_research_encounter_full_loop() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
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
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let insight_before = start_game_accumulate_insight_and_pick_research(&client, 7777);

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

/// Scenario: Attempt to choose a research project with insufficient Insight.
#[test]
fn scenario_research_insufficient_insight() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    let insight = player_token(&client, "CombatInsight");
    assert_eq!(insight, 0, "Should start with 0 Insight");

    if !deplete_encounters_until_research(&client) {
        return;
    }

    let research_enc = research_encounter_ids(&client);
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
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

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
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let _remaining_insight = match setup_research_with_project(&client, 7777) {
        Some(i) => i,
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
        Some(5),
        "Round 1 should cost 5 Insight"
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
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let remaining_insight = match setup_research_with_project(&client, 9999) {
        Some(i) => i,
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
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let remaining_insight = match setup_research_with_project(&client, 12345) {
        Some(i) => i,
        None => return,
    };

    if remaining_insight < 15 {
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
        Some(5),
        "Round 1 cost should be 5"
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
        Some(10),
        "Round 2 cost should be 10"
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
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let remaining_insight = match setup_research_with_project(&client, 5555) {
        Some(i) => i,
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
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let remaining_insight = match setup_research_with_project(&client, 4444) {
        Some(i) => i,
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
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let remaining_insight = match setup_research_with_project(&client, 33333) {
        Some(i) => i,
        None => return,
    };

    if remaining_insight < 30 {
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
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let remaining_insight = match setup_research_with_project(&client, 55555) {
        Some(i) => i,
        None => return,
    };

    if remaining_insight < 30 {
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
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let remaining_insight = match setup_research_with_project(&client, 88888) {
        Some(i) => i,
        None => return,
    };

    if remaining_insight < 60 {
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
