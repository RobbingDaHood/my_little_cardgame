use super::helpers::*;
use my_little_cardgame::rocket_initialize;
use rocket::http::Status;
use rocket::local::blocking::Client;
use rocket::serde::json::serde_json;
use serde_json::Value;

/// Get all encounter cards in hand with their full JSON details.
fn encounter_hand_details(client: &Client) -> Vec<Value> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards.as_array().cloned().unwrap_or_default()
}

/// Get encounter type string from a library encounter card JSON value.
fn encounter_type(card: &Value) -> Option<String> {
    card.get("kind")?
        .get("encounter_kind")?
        .get("encounter_type")?
        .as_str()
        .map(String::from)
}

/// Sum all DeckCounts (deck+hand+discard) across entries in an enemy deck JSON array.
fn sum_enemy_deck_counts(deck: &[Value]) -> u32 {
    deck.iter()
        .filter_map(|entry| {
            let c = entry.get("counts")?;
            let d = c.get("deck")?.as_u64().unwrap_or(0) as u32;
            let h = c.get("hand")?.as_u64().unwrap_or(0) as u32;
            let dis = c.get("discard")?.as_u64().unwrap_or(0) as u32;
            Some(d + h + dis)
        })
        .sum()
}

/// Extract the CombatantDef from an encounter card JSON (only for Combat encounters).
fn combatant_def(card: &Value) -> Option<&Value> {
    card.get("kind")?
        .get("encounter_kind")?
        .get("combatant_def")
}

/// Extract initial_tokens from a CombatantDef JSON.
/// Serialized as {"Health": 2000, "MaxHealth": 2000} (object, not array).
fn initial_token_value(cdef: &Value, token_name: &str) -> Option<u64> {
    cdef.get("initial_tokens")?
        .as_object()?
        .get(token_name)?
        .as_u64()
}

#[test]
fn scenario_scouting_generates_three_mutated_combat_encounters() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Start a new game with a fixed seed
    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    // Record the initial encounter hand
    let initial_hand = encounter_hand_ids(&client);
    assert!(!initial_hand.is_empty(), "Should have initial encounters");

    // Pick a combat encounter
    let combat_enc = combat_encounter_ids(&client);
    assert!(!combat_enc.is_empty(), "Should have combat encounters");

    let picked_id = combat_enc[0];

    // Record the combat encounter's enemy HP before starting
    let pre_cards = encounter_hand_details(&client);
    let source_card = pre_cards
        .iter()
        .find(|c| c.get("id").and_then(|v| v.as_u64()).map(|v| v as usize) == Some(picked_id));
    let source_hp = source_card
        .and_then(combatant_def)
        .and_then(|d| initial_token_value(d, "Health"))
        .unwrap_or(0);

    // Record enemy deck total card counts from the source encounter
    let source_combat_def = source_card.and_then(combatant_def);
    let source_atk_total = source_combat_def
        .and_then(|d| d.get("attack_deck")?.as_array())
        .map(|a| sum_enemy_deck_counts(a))
        .unwrap_or(0);
    let source_def_total = source_combat_def
        .and_then(|d| d.get("defence_deck")?.as_array())
        .map(|a| sum_enemy_deck_counts(a))
        .unwrap_or(0);
    let source_res_total = source_combat_def
        .and_then(|d| d.get("resource_deck")?.as_array())
        .map(|a| sum_enemy_deck_counts(a))
        .unwrap_or(0);
    let source_total = source_atk_total + source_def_total + source_res_total;

    // Start and win the combat
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        picked_id
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created);

    for _ in 0..80 {
        if !play_one_round(&client) {
            break;
        }
    }
    let result = combat_result(&client);
    assert_eq!(
        result.as_deref(),
        Some("PlayerWon"),
        "Should win combat for scouting to activate"
    );

    // Apply scouting
    let (status, _) = post_action(
        &client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    assert_eq!(status, Status::Created, "Scouting should succeed");

    // Check encounter hand — should have new mutated encounters
    let post_hand = encounter_hand_details(&client);

    // Find combat encounters that are NEW (not in initial hand)
    let new_combat_encounters: Vec<&Value> = post_hand
        .iter()
        .filter(|c| {
            let id = c.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            encounter_type(c).as_deref() == Some("Combat") && !initial_hand.contains(&id)
        })
        .collect();

    assert_eq!(
        new_combat_encounters.len(),
        3,
        "Scouting should generate exactly 3 new combat encounters, got {}",
        new_combat_encounters.len()
    );

    // Verify the 3 encounters have distinct HP values (mutation should differentiate them)
    let hp_values: Vec<u64> = new_combat_encounters
        .iter()
        .filter_map(|c| combatant_def(c).and_then(|d| initial_token_value(d, "Health")))
        .collect();
    assert_eq!(hp_values.len(), 3, "All 3 should have Health tokens");

    // At least 2 of the 3 should have different HP (mutations with ≥0.10 separation)
    let unique_hp: std::collections::HashSet<u64> = hp_values.iter().copied().collect();
    assert!(
        unique_hp.len() >= 2,
        "Mutated encounters should have distinct HP values, got {:?}",
        hp_values
    );

    // Verify HP values are in a reasonable range relative to source
    // Factor range: 1.0 + [-0.15, +0.30] = [0.85, 1.30]
    for &hp in &hp_values {
        let ratio = hp as f64 / source_hp as f64;
        assert!(
            (0.80..=1.35).contains(&ratio),
            "Mutated HP {} should be within [0.85, 1.30] of source HP {}, ratio={}",
            hp,
            source_hp,
            ratio
        );
    }

    // Verify enemy deck total card count is preserved (constant count invariant)
    for enc in &new_combat_encounters {
        let cdef = combatant_def(enc).expect("should have combatant_def");
        let atk = cdef
            .get("attack_deck")
            .and_then(|v| v.as_array())
            .map(|a| sum_enemy_deck_counts(a))
            .unwrap_or(0);
        let def = cdef
            .get("defence_deck")
            .and_then(|v| v.as_array())
            .map(|a| sum_enemy_deck_counts(a))
            .unwrap_or(0);
        let res = cdef
            .get("resource_deck")
            .and_then(|v| v.as_array())
            .map(|a| sum_enemy_deck_counts(a))
            .unwrap_or(0);
        let total = atk + def + res;
        assert_eq!(
            total, source_total,
            "Enemy deck total count should be preserved: expected {}, got {}",
            source_total, total
        );
    }
}

#[test]
fn scenario_scouting_cleanup_removes_unpicked_mutations() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":100}"#);
    assert_eq!(status, Status::Created);

    // Win a combat to trigger scouting mutations
    assert!(win_combat_and_scout(&client), "Should win combat and scout");

    // Now we should have mutated encounters in hand plus existing ones
    let hand = encounter_hand_details(&client);
    let hand_ids: Vec<usize> = hand
        .iter()
        .filter_map(|c| c.get("id").and_then(|v| v.as_u64()).map(|v| v as usize))
        .collect();

    // Pick one of the encounters (any encounter)
    assert!(!hand_ids.is_empty(), "Should have encounters to pick");
    let picked = hand_ids[0];
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        picked
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created, "Should pick encounter");

    // After picking, the un-selected mutation cards should be cleaned up.
    // The total encounter hand should be smaller than before picking.
    let post_pick_hand = encounter_hand_ids(&client);
    assert!(
        post_pick_hand.len() < hand_ids.len(),
        "Hand should shrink after picking (cleanup removes un-selected mutations)"
    );
}

#[test]
fn scenario_scouting_still_draws_from_deck() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created);

    // Win multiple combats and scout — the deck should eventually cycle
    // through all encounter types (including non-combat ones)
    let mut all_seen_types = std::collections::HashSet::new();
    for _ in 0..5 {
        let hand = encounter_hand_details(&client);
        for card in &hand {
            if let Some(t) = encounter_type(card) {
                all_seen_types.insert(t);
            }
        }

        let enc_ids = encounter_hand_ids(&client);
        if enc_ids.is_empty() {
            break;
        }
        let picked = enc_ids[0];
        let pick_json = format!(
            r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
            picked
        );
        let (status, _) = post_action(&client, &pick_json);
        if status != Status::Created {
            break;
        }
        // Abort if non-combat, play through if combat
        let encounter = combat_state(&client);
        if encounter.get("outcome").is_some() {
            // Combat encounter — play through
            for _ in 0..80 {
                if !play_one_round(&client) {
                    break;
                }
            }
        } else {
            let _ = post_action(&client, r#"{"action_type":"EncounterAbort"}"#);
        }
        let _ = post_action(
            &client,
            r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
        );
    }

    // We should see at least 2 different encounter types (not just Combat)
    assert!(
        all_seen_types.len() >= 2,
        "Should see multiple encounter types via deck cycling, saw: {:?}",
        all_seen_types
    );
}
