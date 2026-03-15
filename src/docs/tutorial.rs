use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket_okapi::{openapi, JsonSchema};

/// A single step in the new-player tutorial walkthrough.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct TutorialStep {
    pub step: u32,
    pub title: String,
    pub description: String,
    pub endpoint: String,
    pub method: String,
    pub example_body: Option<String>,
    pub tips: Vec<String>,
}

/// Complete new-player tutorial for learning the game through the API.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct Tutorial {
    pub title: String,
    pub introduction: String,
    pub core_concepts: Vec<String>,
    pub steps: Vec<TutorialStep>,
    pub next_steps: Vec<String>,
}

fn build_tutorial() -> Tutorial {
    Tutorial {
        title: "My Little Card Game — New Player Tutorial".to_string(),
        introduction: "Welcome! This game is a single-player card game where \
            everything is modelled as decks and tokens. You explore encounters \
            (combat, mining, herbalism, woodcutting, fishing, crafting, research, rest) \
            by playing cards from your hand. Every game is fully reproducible via a \
            random seed — the same seed and actions always produce the same result."
            .to_string(),
        core_concepts: vec![
            "Everything is a deck: cards move between Library → Deck → Hand → Discard → Deleted states.".to_string(),
            "Tokens track your resources (Health, Stamina, materials like Ore/Plant/Lumber/Fish) and persist across encounters.".to_string(),
            "Encounters are the core gameplay loop: pick an encounter card, play discipline-specific cards, conclude or abort.".to_string(),
            "Scouting happens after each encounter — you can pick bonus encounter cards to expand your options.".to_string(),
            "Death resets your gathering materials but not your progress. You keep Insight tokens and crafted cards.".to_string(),
            "The /actions/possible endpoint always tells you what you can do right now.".to_string(),
        ],
        steps: vec![
            TutorialStep {
                step: 1,
                title: "Start a New Game".to_string(),
                description: "Initialize the game with a seed for reproducible gameplay. \
                    If you omit the seed, a random one is generated."
                    .to_string(),
                endpoint: "/action".to_string(),
                method: "POST".to_string(),
                example_body: Some(r#"{"action_type": "NewGame", "seed": 42}"#.to_string()),
                tips: vec![
                    "Use the same seed to replay a game identically.".to_string(),
                    "The response includes the action log entry with the seed used.".to_string(),
                ],
            },
            TutorialStep {
                step: 2,
                title: "Check Your Starting State".to_string(),
                description: "View your token balances to see starting Health, Stamina, \
                    Durability, and hand size limits. Also check the library cards to see \
                    what's available."
                    .to_string(),
                endpoint: "/player/tokens".to_string(),
                method: "GET".to_string(),
                example_body: None,
                tips: vec![
                    "Health and Stamina start at 1000. Durabilities at 10000.".to_string(),
                    "GET /library/cards?location=Hand shows cards currently in your hand.".to_string(),
                    "GET /library/cards?card_kind=Encounter shows available encounter types.".to_string(),
                ],
            },
            TutorialStep {
                step: 3,
                title: "Check Available Actions".to_string(),
                description: "The /actions/possible endpoint tells you exactly what \
                    you can do right now. Use it whenever you're unsure of valid moves."
                    .to_string(),
                endpoint: "/actions/possible".to_string(),
                method: "GET".to_string(),
                example_body: None,
                tips: vec![
                    "This endpoint is your best friend — it lists valid actions and playable card IDs.".to_string(),
                    "Before an encounter: you'll see EncounterPickEncounter with encounter card IDs.".to_string(),
                    "During an encounter: you'll see EncounterPlayCard with playable discipline cards.".to_string(),
                ],
            },
            TutorialStep {
                step: 4,
                title: "Pick an Encounter".to_string(),
                description: "Choose an encounter card from your hand to begin. Different \
                    encounter types test different skills: Combat tests attack/defence, \
                    Mining tests light management, Fishing tests value prediction, etc."
                    .to_string(),
                endpoint: "/action".to_string(),
                method: "POST".to_string(),
                example_body: Some(
                    r#"{"action_type": "EncounterPickEncounter", "card_id": 62}"#.to_string(),
                ),
                tips: vec![
                    "Get encounter card IDs from /actions/possible or /library/cards?card_kind=Encounter&location=Hand.".to_string(),
                    "Start with gathering encounters (Mining, Herbalism, Woodcutting, Fishing) — they're easier than Combat.".to_string(),
                    "Each encounter type uses its own set of discipline cards.".to_string(),
                ],
            },
            TutorialStep {
                step: 5,
                title: "Play Cards During the Encounter".to_string(),
                description: "Play discipline-specific cards to progress through the encounter. \
                    In Combat, you cycle through Defending → Attacking → Resourcing phases. \
                    In gathering encounters, you play cards to collect resources while \
                    managing encounter-specific constraints."
                    .to_string(),
                endpoint: "/action".to_string(),
                method: "POST".to_string(),
                example_body: Some(
                    r#"{"action_type": "EncounterPlayCard", "card_id": 10}"#.to_string(),
                ),
                tips: vec![
                    "Check /encounter to see the current encounter state (health, tokens, phase).".to_string(),
                    "Check /actions/possible to see which cards you can play right now.".to_string(),
                    "Cards have costs (Stamina, Mana, Durability) — you can't play what you can't afford.".to_string(),
                    "Some encounters auto-conclude when conditions are met (e.g., enemy dies in Combat).".to_string(),
                ],
            },
            TutorialStep {
                step: 6,
                title: "Conclude the Encounter".to_string(),
                description: "When you've achieved your goal (or want to leave), conclude \
                    the encounter. For gathering encounters you can also abort early, \
                    which counts as a loss but lets you escape bad situations."
                    .to_string(),
                endpoint: "/action".to_string(),
                method: "POST".to_string(),
                example_body: Some(
                    r#"{"action_type": "EncounterConcludeEncounter"}"#.to_string(),
                ),
                tips: vec![
                    "Combat concludes automatically when either side's Health reaches 0.".to_string(),
                    "Gathering encounters let you choose when to conclude — balance risk vs reward.".to_string(),
                    "Use EncounterAbort to exit early with a loss if things go badly.".to_string(),
                ],
            },
            TutorialStep {
                step: 7,
                title: "Apply Scouting — Mutated Encounters".to_string(),
                description: "After an encounter concludes, you enter the Scouting phase. \
                    Scouting refills your encounter hand from the deck AND generates 3 mutated \
                    variations of the encounter you just completed. These mutations adjust \
                    difficulty parameters and shuffle the enemy deck composition. Pick any \
                    encounter from your hand (original or mutated) to continue."
                    .to_string(),
                endpoint: "/action".to_string(),
                method: "POST".to_string(),
                example_body: Some(
                    r#"{"action_type": "EncounterApplyScouting", "card_ids": []}"#
                        .to_string(),
                ),
                tips: vec![
                    "Scouting generates 3 mutated copies of your last encounter with varied difficulty.".to_string(),
                    "Un-selected mutations are automatically cleaned up when you pick your next encounter.".to_string(),
                    "The encounter hand also refills from the deck, so new encounter types can appear.".to_string(),
                    "Your Foresight token controls how many encounter cards are drawn from the deck.".to_string(),
                ],
            },
            TutorialStep {
                step: 8,
                title: "Research Experiments".to_string(),
                description: "Research encounters use a hidden-multiplier deduction mechanic. \
                    After choosing a project (ResearchChooseProject + ResearchSelectCandidate), \
                    play 3 Research cards per round via ResearchPlayHand. Each round costs \
                    escalating Insight (round N costs N × 5). Your cards are scored against 3 \
                    hidden symbol slots — position matches score 100 yield, type-only matches \
                    score 10. Use per_card_yield feedback to deduce the hidden symbols, then \
                    optimize future rounds. Stop with ResearchConcludeExperiment when costs \
                    outweigh expected yield."
                    .to_string(),
                endpoint: "/action".to_string(),
                method: "POST".to_string(),
                example_body: Some(
                    r#"{"action_type": "ResearchPlayHand", "card_ids": [40, 42, 45]}"#
                        .to_string(),
                ),
                tips: vec![
                    "Round 1 is information gathering — play diverse symbol types to learn what matches.".to_string(),
                    "Premium cards (dual/triple-symbol) cost Stamina or Health but match more hidden types.".to_string(),
                    "hidden_types are never shown in the API — deduce them from round_history yields.".to_string(),
                    "Accumulated yield > 0 = PlayerWon; yield == 0 = PlayerLost.".to_string(),
                ],
            },
            TutorialStep {
                step: 9,
                title: "Continue the Loop".to_string(),
                description: "After scouting, you're back at encounter selection. Pick another \
                    encounter and repeat! Build up materials from gathering, use them in Crafting \
                    and Research, and tackle Combat encounters for Insight and Milestone rewards."
                    .to_string(),
                endpoint: "/actions/possible".to_string(),
                method: "GET".to_string(),
                example_body: None,
                tips: vec![
                    "Check /encounter/results to see your win/loss history.".to_string(),
                    "Check /metrics for detailed session statistics.".to_string(),
                    "Materials (Ore, Plant, Lumber, Fish) persist across encounters — accumulate them.".to_string(),
                    "Death resets materials but not Insight or crafted cards.".to_string(),
                ],
            },
        ],
        next_steps: vec![
            "Try different encounter types to learn each discipline's unique mechanics.".to_string(),
            "Check /docs/hints for strategies and tips per discipline.".to_string(),
            "Check /docs/designer for how encounters and cards are structured.".to_string(),
            "Use /actions/log to review your action history and replay games.".to_string(),
            "Explore /swagger/ for the complete interactive API documentation.".to_string(),
        ],
    }
}

/// New-player tutorial that walks through a first game session.
///
/// Returns a structured walkthrough covering: starting a game, checking state,
/// picking encounters, playing cards, concluding encounters, and scouting.
/// Each step includes the endpoint to call, example request body, and tips.
/// Follow these steps to learn the game through hands-on API exploration.
#[openapi]
#[get("/docs/tutorial")]
pub fn get_tutorial() -> Json<Tutorial> {
    Json(build_tutorial())
}
