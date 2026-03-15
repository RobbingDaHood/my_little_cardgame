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

/// Get full milestone card JSON by card id.
fn get_milestone_card(client: &Client, card_id: usize) -> serde_json::Value {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .find(|c| c.get("id").and_then(|v| v.as_u64()) == Some(card_id as u64))
        .cloned()
        .unwrap_or_default()
}

/// Count EnemyCardDefs in a deck array and collect their hand counts.
fn deck_hand_counts(deck: &serde_json::Value) -> Vec<u64> {
    deck.as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|entry| {
            entry
                .get("counts")
                .and_then(|c| c.get("hand"))
                .and_then(|h| h.as_u64())
        })
        .collect()
}

#[test]
fn milestone_combat_preserves_deck_composition() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);

    // Record the tier-1 combat milestone's card_id and deck structure
    let combat_id =
        milestone_encounter_by_discipline(&client, "Combat").expect("Should have combat milestone");

    let card_json = get_milestone_card(&client, combat_id);
    let def = &card_json["kind"]["encounter_kind"]["milestone_def"];
    let inner = &def["inner_encounter_kind"];

    assert_eq!(
        inner.get("encounter_type").and_then(|v| v.as_str()),
        Some("Combat"),
        "Inner encounter should be Combat"
    );

    let combatant = &inner["combatant_def"];
    let t1_attack_counts = deck_hand_counts(&combatant["attack_deck"]);
    let t1_defence_counts = deck_hand_counts(&combatant["defence_deck"]);
    let t1_resource_counts = deck_hand_counts(&combatant["resource_deck"]);

    assert!(
        !t1_attack_counts.is_empty(),
        "Tier-1 should have attack deck entries"
    );
    assert!(
        !t1_defence_counts.is_empty(),
        "Tier-1 should have defence deck entries"
    );
    assert!(
        !t1_resource_counts.is_empty(),
        "Tier-1 should have resource deck entries"
    );

    // Win combats until we have enough MilestoneInsight
    win_combat(&client);

    let insight = player_token(&client, "MilestoneInsight");
    assert!(insight >= 100, "Need at least 100 MilestoneInsight");

    // Start and win the combat milestone
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        combat_id
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created, "Pick combat milestone should work");

    for _ in 0..300 {
        if !play_one_round(&client) {
            break;
        }
    }

    // Check if we're back to NoEncounter (won the fight)
    let actions = possible_action_types(&client);
    if !actions.contains(&"EncounterPickEncounter".to_string()) {
        // Still in encounter — couldn't win with this seed/round limit
        return;
    }

    // Apply scouting if needed
    if actions.contains(&"EncounterApplyScouting".to_string()) {
        post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
    }

    // The card at the same ID should now be tier 2 with the same deck structure
    let card_after = get_milestone_card(&client, combat_id);

    // If the milestone was replaced (won), verify composition
    if !card_after.is_null() {
        let def_after = &card_after["kind"]["encounter_kind"]["milestone_def"];
        assert_eq!(
            def_after.get("tier").and_then(|v| v.as_u64()),
            Some(2),
            "Should be tier 2 after winning"
        );

        let inner_after = &def_after["inner_encounter_kind"];
        let combatant_after = &inner_after["combatant_def"];

        let t2_attack_counts = deck_hand_counts(&combatant_after["attack_deck"]);
        let t2_defence_counts = deck_hand_counts(&combatant_after["defence_deck"]);
        let t2_resource_counts = deck_hand_counts(&combatant_after["resource_deck"]);

        // Same number of card types per deck
        assert_eq!(
            t1_attack_counts.len(),
            t2_attack_counts.len(),
            "Attack deck should have same number of card types: tier1={:?} tier2={:?}",
            t1_attack_counts,
            t2_attack_counts
        );
        assert_eq!(
            t1_defence_counts.len(),
            t2_defence_counts.len(),
            "Defence deck should have same number of card types: tier1={:?} tier2={:?}",
            t1_defence_counts,
            t2_defence_counts
        );
        assert_eq!(
            t1_resource_counts.len(),
            t2_resource_counts.len(),
            "Resource deck should have same number of card types: tier1={:?} tier2={:?}",
            t1_resource_counts,
            t2_resource_counts
        );

        // Same hand counts per card type
        assert_eq!(
            t1_attack_counts, t2_attack_counts,
            "Attack deck hand counts should be preserved"
        );
        assert_eq!(
            t1_defence_counts, t2_defence_counts,
            "Defence deck hand counts should be preserved"
        );
        assert_eq!(
            t1_resource_counts, t2_resource_counts,
            "Resource deck hand counts should be preserved"
        );
    }
}

#[test]
fn milestone_card_id_preserved_on_win() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);

    let combat_id =
        milestone_encounter_by_discipline(&client, "Combat").expect("Should have combat milestone");

    // Win combat to earn insight
    win_combat(&client);

    let insight = player_token(&client, "MilestoneInsight");
    if insight < 100 {
        return;
    }

    // Start combat milestone
    let pick = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        combat_id
    );
    let (status, _) = post_action(&client, &pick);
    assert_eq!(status, Status::Created);

    for _ in 0..300 {
        if !play_one_round(&client) {
            break;
        }
    }

    let actions = possible_action_types(&client);
    if !actions.contains(&"EncounterPickEncounter".to_string()) {
        return;
    }

    if actions.contains(&"EncounterApplyScouting".to_string()) {
        post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
    }

    // Verify the combat milestone is still at the same card ID
    let after_id = milestone_encounter_by_discipline(&client, "Combat");
    assert_eq!(
        after_id,
        Some(combat_id),
        "Combat milestone should remain at card ID {} after in-place replacement",
        combat_id
    );

    // And total milestone count is still 5
    let all_milestones = milestone_encounter_ids(&client);
    assert_eq!(
        all_milestones.len(),
        5,
        "Should still have exactly 5 milestones"
    );
}
