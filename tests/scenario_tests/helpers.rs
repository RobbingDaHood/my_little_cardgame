use rocket::http::uncased::Uncased;
use rocket::http::{Header, Status};
use rocket::local::blocking::Client;
use rocket::serde::json::serde_json;
use std::borrow::Cow;

pub fn json_header() -> Header<'static> {
    Header {
        name: Uncased::from("Content-Type"),
        value: Cow::from("application/json"),
    }
}

pub fn post_action(client: &Client, json: &str) -> (Status, serde_json::Value) {
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

pub fn get_json(client: &Client, uri: &str) -> serde_json::Value {
    let resp = client.get(uri).dispatch();
    serde_json::from_str(&resp.into_string().unwrap_or_default()).unwrap_or_default()
}

pub fn player_health(client: &Client) -> i64 {
    player_token(client, "Health")
}

pub fn player_token(client: &Client, token_type_name: &str) -> i64 {
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

pub fn combat_state(client: &Client) -> serde_json::Value {
    get_json(client, "/encounter")
}

pub fn encounter_hand_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| c.get("id").and_then(|v| v.as_u64()).map(|v| v as usize))
        .collect()
}

pub fn combat_result(client: &Client) -> Option<String> {
    let resp = client.get("/encounter/results").dispatch();
    if resp.status() == Status::Ok {
        let body: Vec<serde_json::Value> =
            serde_json::from_str(&resp.into_string().unwrap_or_default()).unwrap_or_default();
        body.last().and_then(|v| v.as_str()).map(String::from)
    } else {
        None
    }
}

/// Find hand card IDs of a given card_kind (e.g. "Defence", "Attack", "Resource").
pub fn hand_card_ids_by_kind(client: &Client, kind: &str) -> Vec<usize> {
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

/// Find a hand card of the given kind that has non-empty `rolled_costs` in its effects.
pub fn cost_card_id(client: &Client, kind: &str) -> usize {
    let cards = get_json(
        client,
        &format!("/library/cards?location=Hand&card_kind={}", kind),
    );
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .find_map(|c| {
            let effects = c.get("kind")?.get("effects")?.as_array()?;
            let has_cost = effects.iter().any(|e| {
                e.get("rolled_costs")
                    .and_then(|c| c.as_array())
                    .map(|costs| !costs.is_empty())
                    .unwrap_or(false)
            });
            if has_cost {
                c.get("id")?.as_u64().map(|v| v as usize)
            } else {
                None
            }
        })
        .expect("Should find a cost card in hand")
}

/// Play one full round (Defence -> Attack -> Resource) using dynamically
/// discovered card IDs.
/// Returns true if combat is still active after the round.
pub fn play_one_round(client: &Client) -> bool {
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

/// Helper: sum (deck, hand, discard) counts across ALL cards of a given kind.
pub fn total_counts_by_kind(client: &Client, kind: &str) -> (u32, u32, u32) {
    let cards = get_json(client, &format!("/library/cards?card_kind={}", kind));
    let empty = vec![];
    let arr = cards.as_array().unwrap_or(&empty);
    let mut deck_total = 0u32;
    let mut hand_total = 0u32;
    let mut discard_total = 0u32;
    for card in arr {
        if let Some(counts) = card.get("counts") {
            deck_total += counts["deck"].as_u64().unwrap_or(0) as u32;
            hand_total += counts["hand"].as_u64().unwrap_or(0) as u32;
            discard_total += counts["discard"].as_u64().unwrap_or(0) as u32;
        }
    }
    (deck_total, hand_total, discard_total)
}

/// Helper: read an encounter-scoped token from `/encounter`'s `encounter_tokens` field.
pub fn encounter_token(client: &Client, token_type_name: &str) -> i64 {
    let encounter = combat_state(client);
    encounter
        .get("encounter_tokens")
        .and_then(|v| v.as_object())
        .and_then(|obj| obj.get(token_type_name))
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
}

/// Helper: sum (deck, hand, discard) across all entries of an enemy deck from combat state.
pub fn enemy_deck_totals(combat: &serde_json::Value, deck_key: &str) -> (u32, u32, u32) {
    let deck = combat
        .get(deck_key)
        .and_then(|v| v.as_array())
        .expect("enemy deck array");
    let mut total_deck = 0u32;
    let mut total_hand = 0u32;
    let mut total_discard = 0u32;
    for entry in deck {
        let c = entry.get("counts").expect("enemy card counts");
        total_deck += c["deck"].as_u64().unwrap_or(0) as u32;
        total_hand += c["hand"].as_u64().unwrap_or(0) as u32;
        total_discard += c["discard"].as_u64().unwrap_or(0) as u32;
    }
    (total_deck, total_hand, total_discard)
}

/// Find combat encounter card IDs in the encounter hand.
pub fn combat_encounter_ids(client: &Client) -> Vec<usize> {
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

/// Find mining encounter card IDs in the encounter hand.
pub fn mining_encounter_ids(client: &Client) -> Vec<usize> {
    let cards = get_json(client, "/library/cards?location=Hand&card_kind=Encounter");
    cards
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| {
            let kind = c.get("kind")?;
            let enc_kind = kind.get("encounter_kind")?;
            if enc_kind.get("encounter_type")?.as_str()? == "Mining" {
                c.get("id")?.as_u64().map(|v| v as usize)
            } else {
                None
            }
        })
        .collect()
}

/// Win a combat encounter while specifically playing Insight resource cards
/// when available, and scout. Returns true if combat was won.
pub fn win_combat_and_scout(client: &Client) -> bool {
    let combat_enc = combat_encounter_ids(client);
    if combat_enc.is_empty() {
        return false;
    }
    let pick_json = format!(
        r#"{{"action_type":"EncounterPickEncounter","card_id":{}}}"#,
        combat_enc[0]
    );
    let (status, _) = post_action(client, &pick_json);
    if status != Status::Created {
        return false;
    }
    for _ in 0..80 {
        if !play_one_round_prefer_insight(client) {
            break;
        }
    }
    let result = combat_result(client);
    if result.as_deref() != Some("PlayerWon") {
        return false;
    }
    let (status, _) = post_action(
        client,
        r#"{"action_type":"EncounterApplyScouting","card_ids":[]}"#,
    );
    status == Status::Created
}

/// Like `play_one_round` but prefers Insight Resource cards when available.
pub fn play_one_round_prefer_insight(client: &Client) -> bool {
    // Play Defence
    let def_ids = hand_card_ids_by_kind(client, "Defence");
    if def_ids.is_empty() {
        return false;
    }
    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        def_ids[0]
    );
    let (status, _) = post_action(client, &json);
    if status != Status::Created {
        return false;
    }
    let combat = combat_state(client);
    if combat.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
        return false;
    }

    // Play Attack
    let atk_ids = hand_card_ids_by_kind(client, "Attack");
    if atk_ids.is_empty() {
        return false;
    }
    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        atk_ids[0]
    );
    let (status, _) = post_action(client, &json);
    if status != Status::Created {
        return false;
    }
    let combat = combat_state(client);
    if combat.get("outcome").and_then(|v| v.as_str()) != Some("Undecided") {
        return false;
    }

    // Play Resource — prefer Insight Resource cards
    let res_cards = get_json(client, "/library/cards?location=Hand&card_kind=Resource");
    let empty = vec![];
    let res_arr = res_cards.as_array().unwrap_or(&empty);
    if res_arr.is_empty() {
        return false;
    }

    // Find an Insight resource card (one whose effects reference the Insight effect)
    let insight_card_id = res_arr.iter().find_map(|c| {
        let id = c.get("id")?.as_u64()? as usize;
        let effects = c.get("kind")?.get("effects")?.as_array()?;
        // Insight cards are identified by having exactly 1 effect
        if effects.len() == 1 {
            Some(id)
        } else {
            None
        }
    });

    let card_to_play = insight_card_id
        .unwrap_or_else(|| res_arr[0].get("id").and_then(|v| v.as_u64()).unwrap() as usize);

    let json = format!(
        r#"{{"action_type":"EncounterPlayCard","card_id":{}}}"#,
        card_to_play
    );
    let (status, _) = post_action(client, &json);
    if status != Status::Created {
        return false;
    }
    let combat = combat_state(client);
    combat.get("outcome").and_then(|v| v.as_str()) == Some("Undecided")
}
