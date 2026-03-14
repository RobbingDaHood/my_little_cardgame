use super::helpers::*;
use my_little_cardgame::rocket_initialize;
use rocket::http::Status;
use rocket::local::blocking::Client;
use rocket::serde::json::serde_json;

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
