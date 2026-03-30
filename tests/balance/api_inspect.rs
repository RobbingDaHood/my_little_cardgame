use crate::game_driver::{get_json, get_possible_actions, get_snapshot, post_action};

/// Dump actual API JSON for herbalism cards, effects, encounters, and game state.
/// Run with: cargo test --features simulation --test balance api_inspect_herbalism -- --nocapture
#[test]
fn api_inspect_herbalism() {
    let client = rocket::local::blocking::Client::tracked(my_little_cardgame::rocket_initialize())
        .expect("valid rocket");

    post_action(
        &client,
        &serde_json::json!({"action_type": "NewGame", "seed": 42_u64}),
    );

    println!("\n╔══════════════════════════════════════╗");
    println!("║   HERBALISM API FORMAT INSPECTOR     ║");
    println!("╚══════════════════════════════════════╝\n");

    // 1. Card effects (templates)
    let effects = get_json(&client, "/library/card-effects");
    let all_effects = effects.as_array().cloned().unwrap_or_default();
    let herb_effects: Vec<&serde_json::Value> = all_effects
        .iter()
        .filter(|e| {
            let json = serde_json::to_string(e).unwrap_or_default();
            json.contains("Herbalism")
        })
        .collect();

    println!(
        "── /library/card-effects ({} total, {} Herbalism) ──",
        all_effects.len(),
        herb_effects.len()
    );
    for (i, eff) in herb_effects.iter().take(2).enumerate() {
        println!(
            "  Effect {}: {}",
            i,
            serde_json::to_string_pretty(eff).unwrap_or_default()
        );
    }
    println!("\n  KEY PATHS for card effect templates:");
    if let Some(eff) = herb_effects.first().or(all_effects.first().as_ref()) {
        println!(
            "    card_kind:   .kind.card_kind = {:?}",
            eff.get("kind")
                .and_then(|k| k.get("card_kind"))
                .and_then(|v| v.as_str())
        );
        println!(
            "    effect_type: .kind.kind.effect_type = {:?}",
            eff.get("kind")
                .and_then(|k| k.get("kind"))
                .and_then(|k| k.get("effect_type"))
                .and_then(|v| v.as_str())
        );
        println!(
            "    match_mode:  .kind.kind.match_mode = {:?}",
            eff.get("kind")
                .and_then(|k| k.get("kind"))
                .and_then(|k| k.get("match_mode"))
        );
    }

    // 2. Concrete hand cards
    let cards = get_json(&client, "/library/cards?location=Hand&card_kind=Herbalism");
    println!("\n── /library/cards?location=Hand&card_kind=Herbalism (first 2) ──");
    if let Some(arr) = cards.as_array() {
        for (i, card) in arr.iter().take(2).enumerate() {
            println!(
                "  Card {}: {}",
                i,
                serde_json::to_string_pretty(card).unwrap_or_default()
            );
        }
        println!("\n  KEY PATHS:");
        if let Some(card) = arr.first() {
            println!(
                "    id:          .id = {:?}",
                card.get("id").and_then(|v| v.as_u64())
            );
            let effects_arr = card
                .get("kind")
                .and_then(|k| k.get("effects"))
                .and_then(|e| e.as_array());
            if let Some(effs) = effects_arr {
                if let Some(eff) = effs.first() {
                    println!(
                        "    effect_id:   .kind.effects[0].effect_id = {:?}",
                        eff.get("effect_id").and_then(|v| v.as_u64())
                    );
                    let costs = eff
                        .get("rolled_costs")
                        .and_then(|c| c.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|c| {
                                    c.get("token_type")
                                        .and_then(|v| v.as_str())
                                        .map(String::from)
                                })
                                .collect::<Vec<_>>()
                        });
                    println!(
                        "    cost tokens: .kind.effects[0].rolled_costs[].token_type = {:?}",
                        costs
                    );
                }
            }
        }
    }

    // 3. Pick an herbalism encounter and dump encounter state
    let encounter_cards = get_json(&client, "/library/cards?location=Hand&card_kind=Encounter");
    let enc_cards = encounter_cards.as_array().cloned().unwrap_or_default();
    let herb_enc = enc_cards.iter().find(|c| {
        let json = serde_json::to_string(c).unwrap_or_default();
        json.contains("Herbalism")
    });

    println!("\n── Encounter cards ({} total) ──", enc_cards.len());
    if let Some(enc) = herb_enc {
        println!(
            "  Herbalism encounter card: {}",
            serde_json::to_string_pretty(enc).unwrap_or_default()
        );
        let enc_id = enc.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
        post_action(
            &client,
            &serde_json::json!({"action_type": "EncounterPickEncounter", "card_id": enc_id}),
        );

        let snapshot = get_snapshot(&client);
        println!("\n── /encounter (after picking herbalism) ──");
        if let Some(encounter) = &snapshot.encounter {
            println!(
                "  Encounter state: {}",
                serde_json::to_string_pretty(encounter).unwrap_or_default()
            );
            println!("\n  KEY PATHS:");
            println!(
                "    state_type:     .encounter_state_type = {:?}",
                encounter
                    .get("encounter_state_type")
                    .and_then(|v| v.as_str())
            );
            println!(
                "    outcome:        .outcome = {:?}",
                encounter.get("outcome").and_then(|v| v.as_str())
            );
            let plants = encounter
                .get("plant_hand")
                .and_then(|v| v.as_array())
                .map(|a| a.len());
            println!("    plant_count:    .plant_hand.len() = {:?}", plants);
            if let Some(plant) = encounter
                .get("plant_hand")
                .and_then(|v| v.as_array())
                .and_then(|a| a.first())
            {
                println!(
                    "    first plant:    .plant_hand[0] = {}",
                    serde_json::to_string_pretty(plant).unwrap_or_default()
                );
            }
        }

        // 4. Possible actions during encounter
        let actions = get_possible_actions(&client);
        println!("\n── /actions/possible (during encounter) ──");
        let action_types: Vec<String> = actions
            .iter()
            .filter_map(|a| {
                a.get("action_type")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .collect();
        println!("  Action types: {:?}", action_types);
        if let Some(play) = actions
            .iter()
            .find(|a| a.get("action_type").and_then(|v| v.as_str()) == Some("EncounterPlayCard"))
        {
            println!(
                "  First EncounterPlayCard: {}",
                serde_json::to_string_pretty(play).unwrap_or_default()
            );
        }
    } else {
        println!("  ⚠ No herbalism encounter card found (may need scouting first)");
        if let Some(first) = enc_cards.first() {
            println!(
                "  First encounter card for reference: {}",
                serde_json::to_string_pretty(first).unwrap_or_default()
            );
        }
    }

    // 5. Player tokens
    let tokens = get_json(&client, "/player/tokens");
    println!("\n── /player/tokens (herbalism-relevant) ──");
    if let Some(arr) = tokens.as_array() {
        for token in arr {
            let name = token
                .get("token_type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if name.contains("erbalism")
                || name.contains("urability")
                || name == "Plant"
                || name == "Health"
                || name == "Stamina"
            {
                println!(
                    "  {:30} = {}",
                    name,
                    token.get("amount").unwrap_or(&serde_json::Value::Null)
                );
            }
        }
    }

    println!("\n════════════════════════════════════════");
    println!("  Cross-check these against driver.rs!");
    println!("════════════════════════════════════════\n");
}
