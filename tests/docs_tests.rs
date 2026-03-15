//! Integration tests for documentation and metrics endpoints.
//!
//! These tests verify that /docs/tutorial, /docs/hints, /docs/designer,
//! and /metrics return valid structured JSON with expected content.

use my_little_cardgame::rocket_initialize;
use rocket::http::uncased::Uncased;
use rocket::http::{Header, Status};
use rocket::local::blocking::Client;
use rocket::serde::json::serde_json;
use std::borrow::Cow;

fn json_header() -> Header<'static> {
    Header {
        name: Uncased::from("Content-Type"),
        value: Cow::from("application/json"),
    }
}

fn post_action(client: &Client, json: &str) -> (Status, serde_json::Value) {
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

fn get_json(client: &Client, uri: &str) -> serde_json::Value {
    let resp = client.get(uri).dispatch();
    serde_json::from_str(&resp.into_string().unwrap_or_default()).unwrap_or_default()
}

fn action_types(client: &Client) -> Vec<String> {
    let actions = get_json(client, "/actions/possible");
    actions
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|a| a["action_type"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn card_ids_in_hand(client: &Client, card_kind_filter: Option<&str>) -> Vec<i64> {
    let uri = match card_kind_filter {
        Some(kind) => format!("/library/cards?location=Hand&card_kind={kind}"),
        None => "/library/cards?location=Hand".to_string(),
    };
    let cards = get_json(client, &uri);
    cards
        .as_array()
        .map(|arr| arr.iter().filter_map(|c| c["id"].as_i64()).collect())
        .unwrap_or_default()
}

// ── Tutorial endpoint ──────────────────────────────────────────────

#[test]
fn tutorial_returns_valid_structure() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let tutorial = get_json(&client, "/docs/tutorial");

    assert!(
        tutorial["title"].is_string(),
        "tutorial should have a title"
    );
    assert!(
        tutorial["introduction"].is_string(),
        "tutorial should have an introduction"
    );

    let steps = tutorial["steps"].as_array().expect("steps should be array");
    assert!(steps.len() >= 6, "tutorial should have at least 6 steps");

    // Verify step structure
    let first = &steps[0];
    assert!(first["step"].is_number());
    assert!(first["title"].is_string());
    assert!(first["description"].is_string());
    assert!(first["endpoint"].is_string());
    assert!(first["method"].is_string());

    let concepts = tutorial["core_concepts"]
        .as_array()
        .expect("core_concepts should be array");
    assert!(
        !concepts.is_empty(),
        "tutorial should have core concepts listed"
    );

    let next = tutorial["next_steps"]
        .as_array()
        .expect("next_steps should be array");
    assert!(!next.is_empty(), "tutorial should have next steps");
}

#[test]
fn tutorial_covers_key_actions() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let tutorial = get_json(&client, "/docs/tutorial");

    let steps = tutorial["steps"].as_array().expect("steps array");
    let all_text: String = steps
        .iter()
        .map(|s| {
            format!(
                "{} {} {} {}",
                s["title"].as_str().unwrap_or_default(),
                s["description"].as_str().unwrap_or_default(),
                s["endpoint"].as_str().unwrap_or_default(),
                s["example_body"].as_str().unwrap_or_default(),
            )
        })
        .collect::<Vec<_>>()
        .join(" ");

    assert!(all_text.contains("NewGame"), "should cover NewGame");
    assert!(
        all_text.contains("EncounterPickEncounter"),
        "should cover picking encounters"
    );
    assert!(
        all_text.contains("EncounterPlayCard"),
        "should cover playing cards"
    );
    assert!(
        all_text.contains("Scouting") || all_text.contains("scouting"),
        "should cover scouting"
    );
}

// ── Hints endpoint ─────────────────────────────────────────────────

#[test]
fn hints_returns_all_disciplines() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let hints = get_json(&client, "/docs/hints");

    assert!(hints["title"].is_string(), "hints should have a title");

    let general = hints["general_tips"]
        .as_array()
        .expect("general_tips array");
    assert!(!general.is_empty(), "should have general tips");

    let disciplines = hints["disciplines"]
        .as_array()
        .expect("disciplines should be array");

    let expected = [
        "Combat",
        "Mining",
        "Herbalism",
        "Woodcutting",
        "Fishing",
        "Rest",
        "Crafting",
        "Research",
    ];

    let found: Vec<String> = disciplines
        .iter()
        .map(|d| d["discipline"].as_str().unwrap_or_default().to_string())
        .collect();

    for exp in &expected {
        assert!(
            found.contains(&exp.to_string()),
            "hints should cover discipline: {exp}"
        );
    }
}

#[test]
fn hints_discipline_structure() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let hints = get_json(&client, "/docs/hints");

    let disciplines = hints["disciplines"].as_array().expect("disciplines array");

    for disc in disciplines {
        let name = disc["discipline"].as_str().unwrap_or("unknown");
        assert!(
            disc["overview"].is_string(),
            "{name} should have an overview"
        );
        assert!(
            disc["key_mechanics"].as_array().is_some(),
            "{name} should have key_mechanics"
        );
        assert!(
            disc["strategies"].as_array().is_some(),
            "{name} should have strategies"
        );

        let strategies = disc["strategies"].as_array().unwrap();
        assert!(
            !strategies.is_empty(),
            "{name} should have at least one strategy"
        );
        assert!(
            strategies[0]["name"].is_string(),
            "strategy should have name"
        );
        assert!(
            strategies[0]["description"].is_string(),
            "strategy should have description"
        );

        assert!(
            disc["common_pitfalls"].as_array().is_some(),
            "{name} should have common_pitfalls"
        );
        assert!(disc["tips"].as_array().is_some(), "{name} should have tips");
    }
}

// ── Designer guide endpoint ────────────────────────────────────────

#[test]
fn designer_returns_valid_structure() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let guide = get_json(&client, "/docs/designer");

    assert!(guide["title"].is_string(), "should have a title");
    assert!(
        guide["introduction"].is_string(),
        "should have an introduction"
    );

    let sections = guide["sections"]
        .as_array()
        .expect("sections should be array");
    assert!(
        sections.len() >= 5,
        "should have at least 5 sections (encounters, cards, tokens, effects, balance)"
    );

    for section in sections {
        let title = section["title"].as_str().unwrap_or("untitled");
        assert!(
            section["description"].is_string(),
            "section '{title}' should have description"
        );
        let entries = section["entries"]
            .as_array()
            .unwrap_or_else(|| panic!("section '{title}' should have entries"));
        assert!(
            !entries.is_empty(),
            "section '{title}' should have at least one entry"
        );
        assert!(
            entries[0]["name"].is_string(),
            "entries should have name field"
        );
        assert!(
            entries[0]["description"].is_string(),
            "entries should have description field"
        );
    }
}

#[test]
fn designer_covers_all_encounter_types() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let guide = get_json(&client, "/docs/designer");

    let sections = guide["sections"].as_array().expect("sections array");
    let encounter_section = sections
        .iter()
        .find(|s| {
            s["title"]
                .as_str()
                .unwrap_or_default()
                .contains("Encounter")
        })
        .expect("should have an Encounter section");

    let entries: Vec<String> = encounter_section["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["name"].as_str().unwrap_or_default().to_string())
        .collect();

    let all_entries = entries.join(" ");
    for keyword in &[
        "Combat",
        "Mining",
        "Herbalism",
        "Woodcutting",
        "Fishing",
        "Rest",
        "Crafting",
        "Research",
    ] {
        assert!(
            all_entries.contains(keyword),
            "encounter section should mention {keyword}"
        );
    }
}

#[test]
fn designer_covers_token_lifecycles() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");
    let guide = get_json(&client, "/docs/designer");

    let sections = guide["sections"].as_array().expect("sections array");
    let token_section = sections
        .iter()
        .find(|s| s["title"].as_str().unwrap_or_default().contains("Token"))
        .expect("should have a Token Lifecycles section");

    let all_text: String = token_section["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            format!(
                "{} {}",
                e["name"].as_str().unwrap_or_default(),
                e["description"].as_str().unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join(" ");

    assert!(all_text.contains("Health"), "should mention Health");
    assert!(all_text.contains("Stamina"), "should mention Stamina");
    assert!(all_text.contains("Insight"), "should mention Insight");
    assert!(all_text.contains("Durability"), "should mention Durability");
}

// ── Metrics endpoint ───────────────────────────────────────────────

#[test]
fn metrics_fresh_game_returns_zeroed_stats() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type": "NewGame", "seed": 99}"#);
    assert_eq!(status, Status::Created);

    let metrics = get_json(&client, "/metrics");

    assert_eq!(metrics["total_encounters"], 0);
    assert_eq!(metrics["total_deaths"], 0);

    let per_discipline = metrics["per_discipline"]
        .as_array()
        .expect("per_discipline should be an array");
    // Fresh game: no encounters played, so per_discipline filters out zeros
    assert!(
        per_discipline.is_empty(),
        "fresh game should have no discipline stats"
    );
}

#[test]
fn metrics_accumulate_after_encounters() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    let (status, _) = post_action(&client, r#"{"action_type": "NewGame", "seed": 42}"#);
    assert_eq!(status, Status::Created);

    // Find a non-combat encounter card (abortable)
    let cards = get_json(&client, "/library/cards?location=Hand&card_kind=Encounter");
    let card_arr = cards.as_array().expect("encounter cards array");

    let non_combat = card_arr
        .iter()
        .find(|c| c["kind"]["encounter_kind"]["encounter_type"].as_str() != Some("Combat"))
        .or_else(|| card_arr.first())
        .expect("should have at least one encounter card");

    let enc_id = non_combat["id"].as_i64().expect("encounter card id");

    let (status, _) = post_action(
        &client,
        &format!(r#"{{"action_type": "EncounterPickEncounter", "card_id": {enc_id}}}"#),
    );
    assert!(
        status == Status::Ok || status == Status::Created,
        "EncounterPickEncounter should succeed, got {status}"
    );

    // Abort the encounter to get a quick record
    let (abort_status, _) = post_action(&client, r#"{"action_type": "EncounterAbort"}"#);

    // If abort didn't work, play through the encounter
    if abort_status != Status::Ok && abort_status != Status::Created {
        for _ in 0..100 {
            let types = action_types(&client);

            if types
                .iter()
                .any(|t| t == "EncounterApplyScouting" || t == "EncounterPickEncounter")
            {
                break;
            }

            if types.iter().any(|t| t == "EncounterPlayCard") {
                let playable = card_ids_in_hand(&client, None);
                if let Some(&cid) = playable.first() {
                    post_action(
                        &client,
                        &format!(r#"{{"action_type": "EncounterPlayCard", "card_id": {cid}}}"#),
                    );
                    continue;
                }
            }

            if types.iter().any(|t| t == "EncounterConcludeEncounter") {
                post_action(&client, r#"{"action_type": "EncounterConcludeEncounter"}"#);
                break;
            }

            break;
        }
    }

    // Handle scouting phase if present
    let types = action_types(&client);
    if types.iter().any(|t| t == "EncounterApplyScouting") {
        post_action(
            &client,
            r#"{"action_type": "EncounterApplyScouting", "card_ids": []}"#,
        );
        post_action(&client, r#"{"action_type": "EncounterConcludeEncounter"}"#);
    }

    // Check metrics reflect the encounter
    let metrics = get_json(&client, "/metrics");
    let total = metrics["total_encounters"].as_i64().unwrap_or(0);
    assert!(
        total >= 1,
        "should have at least 1 encounter recorded after playing, got {total}. Metrics: {metrics}"
    );
}

#[test]
fn metrics_reset_on_new_game() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

    // Start first game and play an encounter
    let (_, _) = post_action(&client, r#"{"action_type": "NewGame", "seed": 42}"#);

    let cards = get_json(&client, "/library/cards?location=Hand&card_kind=Encounter");
    if let Some(enc) = cards
        .as_array()
        .and_then(|a| a.first())
        .and_then(|c| c["id"].as_i64())
    {
        let (_, _) = post_action(
            &client,
            &format!(r#"{{"action_type": "EncounterPickEncounter", "card_id": {enc}}}"#),
        );
    }

    // Start new game — metrics should reset
    let (_, _) = post_action(&client, r#"{"action_type": "NewGame", "seed": 99}"#);

    let metrics = get_json(&client, "/metrics");
    assert_eq!(
        metrics["total_encounters"].as_i64().unwrap_or(-1),
        0,
        "metrics should reset after NewGame"
    );
}

// ── /version endpoint tests ──

#[test]
fn version_returns_ok_with_expected_fields() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket");
    let version = get_json(&client, "/version");

    assert!(
        version["version"].is_string(),
        "version field should be a string"
    );
    assert!(
        version["game_version"].is_string(),
        "game_version field should be a string"
    );
    assert!(
        version["config_hash"].is_string(),
        "config_hash field should be a string"
    );

    let full = version["version"].as_str().unwrap();
    let game_ver = version["game_version"].as_str().unwrap();
    let hash = version["config_hash"].as_str().unwrap();

    assert_eq!(game_ver, "0.0.1", "game version should be 0.0.1");
    assert_eq!(hash.len(), 8, "config hash should be 8 hex chars");
    assert!(
        hash.chars().all(|c| c.is_ascii_hexdigit()),
        "config hash should be hex"
    );
    assert_eq!(
        full,
        format!("{game_ver}-{hash}"),
        "version should be game_version-config_hash"
    );
}

#[test]
fn version_is_deterministic() {
    let client = Client::tracked(rocket_initialize()).expect("valid rocket");
    let v1 = get_json(&client, "/version");
    let v2 = get_json(&client, "/version");
    assert_eq!(
        v1["version"], v2["version"],
        "version should be deterministic across calls"
    );
}
