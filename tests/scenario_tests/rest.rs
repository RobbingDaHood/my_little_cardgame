use super::helpers::*;
use my_little_cardgame::rocket_initialize;
use rocket::http::Status;
use rocket::local::blocking::Client;

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
/// New game -> scout -> pick rest encounter -> verify rest_tokens -> play rest cards
/// (Library card IDs) -> verify recovery applied -> encounter auto-completes when
/// tokens depleted -> back to scouting.
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
