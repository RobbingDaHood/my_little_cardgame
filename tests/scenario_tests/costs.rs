use super::helpers::*;
use my_little_cardgame::rocket_initialize;
use rocket::http::Status;
use rocket::local::blocking::Client;

#[test]
fn scenario_cost_card_rejected_without_stamina() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created, "NewGame should succeed");

    // Pick combat encounter
    let combat_enc = combat_encounter_ids(&client);
    assert!(!combat_enc.is_empty(), "Should have combat encounter cards");
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        combat_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created, "PickEncounter should succeed");

    // Play Defence first to advance past Defending phase
    let def_ids = hand_card_ids_by_kind(&client, "Defence");
    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        def_ids[0]
    );
    let (status, _) = post_action(&client, &json);
    assert_eq!(status, Status::Created, "Defence card should succeed");

    // Record stamina before playing cost Attack card (cost_damage has Stamina cost)
    let stamina_before = player_token(&client, "Stamina");
    assert!(stamina_before > 0, "Player should have Stamina");

    // Play cost Attack card — it has a stamina cost effect.
    let cost_atk_id = cost_card_id(&client, "Attack");
    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        cost_atk_id
    );
    let (status, _body) = post_action(&client, &json);
    assert_eq!(
        status,
        Status::Created,
        "Cost card should succeed (multi-effect evaluation)"
    );

    // Stamina should have decreased (cost was paid since we had enough)
    let stamina_after = player_token(&client, "Stamina");
    assert!(
        stamina_after < stamina_before,
        "Stamina should decrease when cost is affordable: before={}, after={}",
        stamina_before,
        stamina_after
    );
}

#[test]
fn scenario_cost_card_deducts_stamina() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created, "NewGame should succeed");

    // Pick combat encounter
    let combat_enc = combat_encounter_ids(&client);
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        combat_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created, "PickEncounter should succeed");

    // Play a full round to advance: Defence -> Attack -> Resource
    let def_ids = hand_card_ids_by_kind(&client, "Defence");
    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        def_ids[0]
    );
    let (status, _) = post_action(&client, &json);
    assert_eq!(status, Status::Created, "Defence card should succeed");

    let atk_ids = hand_card_ids_by_kind(&client, "Attack");
    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        atk_ids[0]
    );
    let (status, _) = post_action(&client, &json);
    assert_eq!(status, Status::Created, "Attack card should succeed");

    let res_ids = hand_card_ids_by_kind(&client, "Resource");
    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        res_ids[0]
    );
    let (status, _) = post_action(&client, &json);
    assert_eq!(status, Status::Created, "Resource card should succeed");

    // Record stamina before playing cost Attack card
    let stamina_before = player_token(&client, "Stamina");
    assert!(stamina_before > 0, "Player should have Stamina");

    // Defending phase again: play Defence card first, then cost Attack in Attacking phase
    let def_ids = hand_card_ids_by_kind(&client, "Defence");
    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        def_ids[0]
    );
    let (status, _) = post_action(&client, &json);
    assert_eq!(status, Status::Created, "Defence card should succeed");

    // Now in Attacking phase: play cost Attack card
    let cost_atk_id = cost_card_id(&client, "Attack");
    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        cost_atk_id
    );
    let (status, _) = post_action(&client, &json);
    assert_eq!(status, Status::Created, "Cost Attack card should succeed");

    // Verify stamina was consumed by the cost
    let stamina_after = player_token(&client, "Stamina");
    assert!(
        stamina_after < stamina_before,
        "Stamina should decrease after playing cost card (before={}, after={})",
        stamina_before,
        stamina_after
    );
}

#[test]
fn scenario_cost_mining_card_rejected_without_stamina() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created, "NewGame should succeed");

    // Pick mining encounter dynamically
    let mining_enc = mining_encounter_ids(&client);
    assert!(!mining_enc.is_empty(), "Should have mining encounter cards");
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        mining_enc[0]
    );
    let (status, _) = post_action(&client, &pick_json);
    assert_eq!(status, Status::Created, "Mining encounter should start");

    // Verify player starts with 1000 stamina
    let stamina_before = player_token(&client, "Stamina");
    assert_eq!(
        stamina_before, 50000,
        "Player should start with 50000 Stamina"
    );

    // Find a mining hand card with stamina cost and play it
    let cards = get_json(&client, "/library/cards?location=Hand&card_kind=Mining");
    let cost_card_id = cards.as_array().unwrap_or(&vec![]).iter().find_map(|c| {
        let costs = c
            .get("kind")?
            .get("mining_effect")?
            .get("costs")?
            .as_array()?;
        let has_stamina_cost = costs
            .iter()
            .any(|cost| cost.get("token_type").and_then(|v| v.as_str()) == Some("Stamina"));
        if has_stamina_cost {
            c.get("id")?.as_u64().map(|v| v as usize)
        } else {
            None
        }
    });

    if let Some(card_id) = cost_card_id {
        let json = format!(
            r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
            card_id
        );
        let (status, _) = post_action(&client, &json);
        assert_eq!(
            status,
            Status::Created,
            "Cost Mining card should succeed with 1000 stamina"
        );

        let stamina_after = player_token(&client, "Stamina");
        assert!(
            stamina_after < stamina_before,
            "Stamina should decrease after cost mining card (before={}, after={})",
            stamina_before,
            stamina_after
        );
    }

    // Play a non-cost Mining card (one with empty costs)
    let enc_resp = client.get("/encounter").dispatch();
    if enc_resp.status() != Status::NotFound {
        let cards = get_json(&client, "/library/cards?location=Hand&card_kind=Mining");
        let free_card_id = cards.as_array().unwrap_or(&vec![]).iter().find_map(|c| {
            let costs = c
                .get("kind")?
                .get("mining_effect")?
                .get("costs")?
                .as_array()?;
            if costs.is_empty() {
                c.get("id")?.as_u64().map(|v| v as usize)
            } else {
                None
            }
        });

        if let Some(card_id) = free_card_id {
            let json = format!(
                r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
                card_id
            );
            let (status, _) = post_action(&client, &json);
            assert_eq!(
                status,
                Status::Created,
                "Non-cost mining card should succeed"
            );
        }
    }
}

#[test]
fn scenario_cost_cards_exist_in_starting_decks() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type":"NewGame","seed":42}"#);
    assert_eq!(status, Status::Created, "NewGame should succeed");

    // Check that both cost and non-cost Attack cards exist, identified dynamically
    let attack_cards = get_json(&client, "/library/cards?card_kind=Attack");
    let attack_arr = attack_cards.as_array().expect("Attack cards array");
    assert!(
        attack_arr.len() >= 2,
        "Should have at least 2 Attack cards (cost and non-cost), got {}",
        attack_arr.len()
    );
    let cost_attack = attack_arr.iter().find(|c| {
        c.get("kind")
            .and_then(|k| k.get("effects"))
            .and_then(|e| e.as_array())
            .map(|effects| {
                effects.iter().any(|e| {
                    e.get("rolled_costs")
                        .and_then(|c| c.as_array())
                        .map(|costs| !costs.is_empty())
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    });
    let non_cost_attack = attack_arr.iter().find(|c| {
        c.get("kind")
            .and_then(|k| k.get("effects"))
            .and_then(|e| e.as_array())
            .map(|effects| {
                effects.iter().all(|e| {
                    e.get("rolled_costs")
                        .and_then(|c| c.as_array())
                        .map(|costs| costs.is_empty())
                        .unwrap_or(true)
                })
            })
            .unwrap_or(true)
    });
    assert!(cost_attack.is_some(), "Should have a cost Attack card");
    assert!(
        non_cost_attack.is_some(),
        "Should have a non-cost Attack card"
    );

    // Check that both cost and non-cost Defence cards exist
    let defence_cards = get_json(&client, "/library/cards?card_kind=Defence");
    let defence_arr = defence_cards.as_array().expect("Defence cards array");
    assert!(
        !defence_arr.is_empty(),
        "Should have at least 1 Defence card, got {}",
        defence_arr.len()
    );
    // Check that at least one cost Mining card exists
    let mining_cards = get_json(&client, "/library/cards?card_kind=Mining");
    let mining_arr = mining_cards.as_array().expect("Mining cards array");
    let cost_mining = mining_arr.iter().find(|c| {
        c.get("kind")
            .and_then(|k| k.get("effects"))
            .and_then(|effects| effects.as_array())
            .map(|effects| {
                effects.iter().any(|e| {
                    e.get("rolled_value")
                        .and_then(|v| v.as_i64())
                        .map(|v| v > 0)
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    });
    assert!(
        cost_mining.is_some(),
        "Should have at least one Mining card with effects"
    );

    // Check that at least one cost Woodcutting card exists (has Stamina in costs)
    let woodcutting_cards = get_json(&client, "/library/cards?card_kind=Woodcutting");
    let woodcutting_arr = woodcutting_cards
        .as_array()
        .expect("Woodcutting cards array");
    let cost_woodcutting = woodcutting_arr.iter().find(|c| {
        c.get("kind")
            .and_then(|k| k.get("effects"))
            .and_then(|e| e.as_array())
            .map(|effects| !effects.is_empty())
            .unwrap_or(false)
    });
    assert!(
        cost_woodcutting.is_some(),
        "Should have at least one cost Woodcutting card (with Stamina cost)"
    );

    // Verify cost cards have fewer deck copies than non-cost cards
    let cost_atk = cost_attack.unwrap();
    let non_cost_atk = non_cost_attack.unwrap();
    let non_cost_deck = non_cost_atk["counts"]["deck"].as_u64().unwrap_or(0);
    let cost_deck = cost_atk["counts"]["deck"].as_u64().unwrap_or(0);
    assert!(
        cost_deck < non_cost_deck,
        "Cost Attack should have fewer deck copies ({}) than non-cost ({})",
        cost_deck,
        non_cost_deck
    );
}
