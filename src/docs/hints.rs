use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket_okapi::{openapi, JsonSchema};

/// Hints and strategies for a single discipline.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct DisciplineHints {
    pub discipline: String,
    pub overview: String,
    pub key_mechanics: Vec<String>,
    pub strategies: Vec<Strategy>,
    pub common_pitfalls: Vec<String>,
    pub tips: Vec<String>,
}

/// A named strategy with description.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct Strategy {
    pub name: String,
    pub description: String,
}

/// Complete hints guide covering all disciplines and general gameplay.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct HintsGuide {
    pub title: String,
    pub general_tips: Vec<String>,
    pub disciplines: Vec<DisciplineHints>,
}

fn build_hints() -> HintsGuide {
    HintsGuide {
        title: "My Little Card Game — Hints & Strategies".to_string(),
        general_tips: vec![
            "Always check /actions/possible before acting — it shows exactly what's valid.".to_string(),
            "Scouting is crucial: pick diverse encounter types to build flexibility.".to_string(),
            "Death resets materials (Ore, Plant, Lumber, Fish) but preserves Insight, crafted cards, and Renown.".to_string(),
            "Stamina is your universal resource for gathering — manage it carefully.".to_string(),
            "Durability limits how many gathering encounters you can do per type before they degrade.".to_string(),
            "Combat encounters award MilestoneInsight (100 per win) — the primary path to Insight tokens.".to_string(),
            "Use Rest encounters to recover Health and Stamina when running low.".to_string(),
            "Research encounters spend Insight to unlock permanent upgrades across disciplines.".to_string(),
        ],
        disciplines: vec![
            build_combat_hints(),
            build_mining_hints(),
            build_herbalism_hints(),
            build_woodcutting_hints(),
            build_fishing_hints(),
            build_rest_hints(),
            build_crafting_hints(),
            build_research_hints(),
            build_milestone_hints(),
        ],
    }
}

fn build_combat_hints() -> DisciplineHints {
    DisciplineHints {
        discipline: "Combat".to_string(),
        overview: "Combat is a 3-phase turn system (Defending → Attacking → Resourcing). \
            Defeat the enemy by reducing their Health to 0 while surviving their attacks. \
            Damage is mitigated through layers: Dodge → Shield → Health."
            .to_string(),
        key_mechanics: vec![
            "Phase cycling: you can only play Defence cards in Defending phase, Attack cards in Attacking, Resource cards in Resourcing.".to_string(),
            "Damage mitigation: Dodge removes entire attacks, Shield absorbs remaining damage, Health takes the rest.".to_string(),
            "Enemy plays cards each phase too — plan for incoming damage.".to_string(),
            "Combat auto-concludes when either side's Health reaches 0.".to_string(),
            "If all your hand cards become unpayable, you automatically lose.".to_string(),
        ],
        strategies: vec![
            Strategy {
                name: "Aggressive".to_string(),
                description: "Maximize Attack card plays to end combat quickly. Use Resource phase for draw cards to keep hand full. Risk: taking heavy damage if enemy attacks are strong.".to_string(),
            },
            Strategy {
                name: "Defensive".to_string(),
                description: "Focus on Shield and Dodge in Defence phase. Outlast the enemy and chip away with moderate attacks. Works well against high-damage enemies.".to_string(),
            },
            Strategy {
                name: "Balanced".to_string(),
                description: "Mix defence and offence. Use Resource phase to draw cards and generate Stamina/Mana. Adaptable to different enemy types.".to_string(),
            },
        ],
        common_pitfalls: vec![
            "Ignoring the Defence phase — enemy attacks can quickly deplete Health.".to_string(),
            "Running out of Stamina/Mana to pay card costs, leading to unpayable hands.".to_string(),
            "Not using Resource cards to draw more cards — an empty hand means defeat.".to_string(),
        ],
        tips: vec![
            "Dodge is the most efficient defence: it removes the entire attack, not just damage points.".to_string(),
            "MilestoneInsight (100 per win) is valuable for Research — don't skip Combat.".to_string(),
            "Check /encounter to monitor both your and the enemy's Health during combat.".to_string(),
        ],
    }
}

fn build_mining_hints() -> DisciplineHints {
    DisciplineHints {
        discipline: "Mining".to_string(),
        overview: "Mining encounters revolve around managing your Light Level. \
            The mine gets darker each round. Play mining cards to extract ore \
            and restore light. Conclude before light runs out for your rewards."
            .to_string(),
        key_mechanics: vec![
            "Light Level decreases each round — when it hits 0, the encounter fails.".to_string(),
            "Mining Power cards extract ore but don't restore light.".to_string(),
            "Light Restore cards maintain visibility but don't extract ore.".to_string(),
            "Mining Yield tracks total ore accumulated — this becomes your reward.".to_string(),
            "Stamina is spent when concluding, proportional to yield extracted.".to_string(),
        ],
        strategies: vec![
            Strategy {
                name: "Sustained mining".to_string(),
                description: "Alternate between Power and Light Restore cards to maximize light level, allowing more rounds and larger yields.".to_string(),
            },
            Strategy {
                name: "Rush mining".to_string(),
                description: "Play all Power cards first for maximum yield, then conclude before light depletes. Higher risk but faster encounters.".to_string(),
            },
        ],
        common_pitfalls: vec![
            "Letting Light Level reach 0 — this auto-fails the encounter.".to_string(),
            "Spending all Stamina on mining yield — you need Stamina for other encounters too.".to_string(),
        ],
        tips: vec![
            "Mining Durability limits how many mining encounters you can do — pace yourself.".to_string(),
            "Ore is a key crafting material — stockpile it for Crafting encounters.".to_string(),
        ],
    }
}

fn build_herbalism_hints() -> DisciplineHints {
    DisciplineHints {
        discipline: "Herbalism".to_string(),
        overview: "Herbalism is a deterministic puzzle: plants on the board have \
            characteristics (type, color, size, etc.) and you play cards that match \
            these characteristics to remove them. Remove all plants to win."
            .to_string(),
        key_mechanics: vec![
            "Plants have characteristics that your cards must match.".to_string(),
            "Match modes: Or (any match), And (all must match), MostCommon/LeastCommon (characteristic frequency).".to_string(),
            "No RNG on the board — this is a pure skill puzzle.".to_string(),
            "Rewards are granted based on plants harvested.".to_string(),
        ],
        strategies: vec![
            Strategy {
                name: "Narrow targeting".to_string(),
                description: "Use precise single-type removal cards to eliminate specific plants. More efficient per card but requires exact matches.".to_string(),
            },
            Strategy {
                name: "Broad targeting".to_string(),
                description: "Use multi-type removal cards (Or mode) to clear multiple plants per play. Less efficient but more flexible.".to_string(),
            },
        ],
        common_pitfalls: vec![
            "Not reading plant characteristics carefully before choosing cards.".to_string(),
            "Using broad cards early when precise targeting would be more efficient.".to_string(),
        ],
        tips: vec![
            "Plant rewards yield Plant tokens — useful for crafting.".to_string(),
            "The board state is fully visible — plan your removal order.".to_string(),
        ],
    }
}

fn build_woodcutting_hints() -> DisciplineHints {
    DisciplineHints {
        discipline: "Woodcutting".to_string(),
        overview: "Woodcutting encounters require hitting target chop values. \
            Play chop cards that add up to meet or exceed the enemy's requirements. \
            Multi-type cards enable synergies for higher totals."
            .to_string(),
        key_mechanics: vec![
            "Each round you need to meet a target chop value.".to_string(),
            "Cards have chop values that stack — play multiple cards per round.".to_string(),
            "Pattern multipliers: certain card combinations unlock bonus rewards.".to_string(),
            "Woodcutting Durability is consumed per encounter.".to_string(),
        ],
        strategies: vec![
            Strategy {
                name: "Pattern focused".to_string(),
                description: "Play all 8 cards seeking rare patterns for maximum multiplier and reward. Higher risk but potentially much higher payout.".to_string(),
            },
            Strategy {
                name: "Conservative".to_string(),
                description: "Conclude at 4-5 cards for a guaranteed moderate reward. Lower ceiling but consistent income.".to_string(),
            },
        ],
        common_pitfalls: vec![
            "Overcommitting to a pattern that doesn't materialize.".to_string(),
            "Ignoring the target value — failing to meet it wastes the round.".to_string(),
        ],
        tips: vec![
            "Multi-type cards are more valuable — a single 5-value + 3-value = 8 total.".to_string(),
            "Lumber is the woodcutting reward — needed for certain crafting recipes.".to_string(),
        ],
    }
}

fn build_fishing_hints() -> DisciplineHints {
    DisciplineHints {
        discipline: "Fishing".to_string(),
        overview: "Fishing is about predicting fish values. Each turn a fish is drawn \
            with a random value; you win a turn if the fish value falls within your \
            valid range. Win enough turns (e.g., 4 out of 8) to succeed."
            .to_string(),
        key_mechanics: vec![
            "Fish values are randomly drawn within a range each turn.".to_string(),
            "Your valid range (min to max) determines which fish you catch.".to_string(),
            "Cards can modify your range bounds or provide value bonuses.".to_string(),
            "You need to win a certain number of turns (e.g., 4 of 8) to succeed.".to_string(),
        ],
        strategies: vec![
            Strategy {
                name: "Wide range".to_string(),
                description: "Expand your valid range to catch more fish. Higher catch rate but potentially lower-value fish.".to_string(),
            },
            Strategy {
                name: "High-value focus".to_string(),
                description: "Play high-value cards and narrow range to maximize reward per catch. Riskier but more rewarding per fish.".to_string(),
            },
            Strategy {
                name: "Range manipulation".to_string(),
                description: "Dynamically adjust your range based on remaining turns and fish needed. A swing mechanic for recovering from bad starts.".to_string(),
            },
        ],
        common_pitfalls: vec![
            "Setting range too narrow early — you might miss too many fish.".to_string(),
            "Ignoring the turns remaining vs fish needed ratio.".to_string(),
        ],
        tips: vec![
            "Fish tokens are the reward — useful for crafting.".to_string(),
            "The FishingRangeMin and FishingRangeMax tokens show your current valid range.".to_string(),
        ],
    }
}

fn build_rest_hints() -> DisciplineHints {
    DisciplineHints {
        discipline: "Rest".to_string(),
        overview: "Rest encounters let you recover Health and Stamina by spending \
            materials. It's the inverse of gathering — you're in control and the \
            difficulty comes from resource management."
            .to_string(),
        key_mechanics: vec![
            "Rest cards restore Health, Stamina, or both.".to_string(),
            "Cards cost materials (Ore, Plant, Lumber, Fish) to play.".to_string(),
            "Rest encounters always succeed (PlayerWon) — you can abort at any time.".to_string(),
        ],
        strategies: vec![
            Strategy {
                name: "Emergency recovery".to_string(),
                description: "Only rest when Health or Stamina are critically low. Maximizes time spent in productive encounters.".to_string(),
            },
            Strategy {
                name: "Preventive resting".to_string(),
                description: "Rest proactively to maintain high Health/Stamina. Reduces death risk at the cost of material spending.".to_string(),
            },
        ],
        common_pitfalls: vec![
            "Resting too often and depleting materials needed for crafting.".to_string(),
            "Not resting enough and dying, which resets all materials.".to_string(),
        ],
        tips: vec![
            "Death is worse than resting — better to spend some materials on recovery than lose them all.".to_string(),
            "Rest encounter cards show their material costs — check before playing.".to_string(),
        ],
    }
}

fn build_crafting_hints() -> DisciplineHints {
    DisciplineHints {
        discipline: "Crafting".to_string(),
        overview: "Crafting encounters let you create new cards for your library by \
            spending materials and tokens. The enemy 'threat' in crafting is cost \
            inflation — enemy cards increase your costs, not deal damage."
            .to_string(),
        key_mechanics: vec![
            "Select a target card to craft, then play cost-reduction cards.".to_string(),
            "Enemy crafting cards inflate costs each round.".to_string(),
            "Crafting cards reduce material costs for the current craft.".to_string(),
            "Use EncounterCraftSwap to change your craft target.".to_string(),
            "Use EncounterCraftDurability to reinforce a discipline's durability.".to_string(),
        ],
        strategies: vec![
            Strategy {
                name: "Quick craft".to_string(),
                description: "Rush to craft before costs inflate too much. Play all reduction cards immediately and commit.".to_string(),
            },
            Strategy {
                name: "Strategic swapping".to_string(),
                description: "Swap between craft targets based on which reductions you draw. Flexible but uses more rounds.".to_string(),
            },
        ],
        common_pitfalls: vec![
            "Letting costs inflate too high before committing to a craft.".to_string(),
            "Not having enough materials stockpiled before entering crafting.".to_string(),
            "Trying to abort while a craft is in progress — you must complete or fail the active craft first.".to_string(),
        ],
        tips: vec![
            "Crafted cards permanently join your library — they're the primary form of progression.".to_string(),
            "Check /library/cards to see what you can craft and its base costs.".to_string(),
            "Durability crafting is important — gathering disciplines degrade without it.".to_string(),
        ],
    }
}

fn build_research_hints() -> DisciplineHints {
    DisciplineHints {
        discipline: "Research".to_string(),
        overview: "Research encounters use a hidden-multiplier deduction mechanic with \
            an interference deck that disrupts your experiments. After choosing a project \
            (discipline + tier), you play 3 Research cards per round against 3 hidden \
            symbol slots. Position matches yield 100, type-only matches yield 10. Each \
            round, an interference card auto-plays: it may nullify your best match, swap \
            or shuffle hidden slots, reduce yield, or tax next round's Insight cost. \
            Insight costs escalate linearly each round (round N costs N × 5). Deduce \
            hidden symbols from round feedback while adapting to interference disruptions."
            .to_string(),
        key_mechanics: vec![
            "Choose a discipline and tier to research (ResearchChooseProject + ResearchSelectCandidate).".to_string(),
            "Play 3 Research cards per round via ResearchPlayHand — order matters for position matching.".to_string(),
            "Each round costs escalating Insight: round 1 = 5, round 2 = 10, round 3 = 15, etc.".to_string(),
            "Cards are scored via 1:1 optimal matching against hidden symbol slots.".to_string(),
            "Position match (right type + right slot) = 100 yield; type match (right type, wrong slot) = 10.".to_string(),
            "After scoring, one interference card auto-plays from a 5-card enemy hand.".to_string(),
            "Interference types: BlockBestMatch, SwapHiddenSlots, ReduceYield, ShuffleHiddenSlots, InsightTax.".to_string(),
            "Premium cards have multiple symbols (better matching) but cost Stamina or Health.".to_string(),
            "Conclude with ResearchConcludeExperiment — accumulated yield is applied to research progress.".to_string(),
        ],
        strategies: vec![
            Strategy {
                name: "Information-first".to_string(),
                description: "Round 1: play 3 cards with different symbols to maximize \
                    information. Use per_card_yield feedback to narrow down which symbols \
                    match and in which positions. Factor in interference_played when \
                    interpreting yields (BlockBestMatch zeroes one card's yield)."
                    .to_string(),
            },
            Strategy {
                name: "Premium card burst".to_string(),
                description: "Use multi-symbol premium cards to guarantee matches even \
                    without deduction. Costs Stamina/Health but yields more per round, \
                    letting you profit before costs escalate and interference accumulates."
                    .to_string(),
            },
            Strategy {
                name: "Interference adaptation".to_string(),
                description: "Watch interference_played each round. After SwapHiddenSlots \
                    or ShuffleHiddenSlots, previous deductions are invalidated — switch to \
                    information-gathering mode. After InsightTax, consider concluding early \
                    to avoid the cost spike next round."
                    .to_string(),
            },
            Strategy {
                name: "Deep specialization".to_string(),
                description: "Focus Insight into one discipline's higher tiers for powerful \
                    upgrades. Higher tiers cost exponentially more to start but produce \
                    stronger cards."
                    .to_string(),
            },
        ],
        common_pitfalls: vec![
            "Playing too many rounds — escalating Insight costs plus interference can exceed yield gains.".to_string(),
            "Ignoring interference_played feedback — it tells you what disrupted your round.".to_string(),
            "Ignoring per_card_yield feedback — it tells you which cards scored (but not why).".to_string(),
            "Not adapting after SwapHiddenSlots/ShuffleHiddenSlots — your previous deductions may be wrong.".to_string(),
            "Using only basic cards — premium multi-symbol cards are worth the Stamina/Health cost.".to_string(),
            "Neglecting to earn Insight from Combat encounters before trying to research.".to_string(),
        ],
        tips: vec![
            "hidden_types are never shown in the API — deduce them from round_history yields.".to_string(),
            "interference_played tells you WHAT happened but not the specifics (e.g., which slots swapped).".to_string(),
            "6 possible symbols: Alpha, Beta, Gamma, Delta, Epsilon, Zeta. Hidden slots can repeat.".to_string(),
            "Research upgrades persist permanently — prioritize based on your play style.".to_string(),
            "Check discipline-specific Insight balances at /player/tokens before researching.".to_string(),
        ],
    }
}

fn build_milestone_hints() -> DisciplineHints {
    DisciplineHints {
        discipline: "Milestone".to_string(),
        overview: "Milestone encounters are tougher discipline-specific challenges that reward \
            50%-improved CardEffects. Each discipline (Combat, Mining, Herbalism, Woodcutting, \
            Fishing) has its own milestone progression track."
            .to_string(),
        key_mechanics: vec![
            "Cost scales exponentially: 100 * 2^(tier-1) MilestoneInsight per attempt.".to_string(),
            "Win → 50% better CardEffects for that discipline + auto-assigned next-tier milestone."
                .to_string(),
            "Loss → reset encounter, return to NoEncounter (no forced replay).".to_string(),
            "Milestone hand is separate from regular encounters (max 5 via MilestoneMaxHand)."
                .to_string(),
        ],
        strategies: vec![
            Strategy {
                name: "Farm Combat First".to_string(),
                description: "Win regular combats to accumulate MilestoneInsight before \
                    attempting milestones."
                    .to_string(),
            },
            Strategy {
                name: "Diversify Milestones".to_string(),
                description: "Spread milestone attempts across disciplines to build a wide \
                    pool of improved CardEffects for Research."
                    .to_string(),
            },
        ],
        common_pitfalls: vec![
            "Don't attempt milestones without enough Insight — the cost is deducted on start."
                .to_string(),
            "Higher tiers are exponentially harder AND more expensive — prepare thoroughly."
                .to_string(),
        ],
        tips: vec![
            "Abort is always available — treat milestones as low-risk scouting for your limits."
                .to_string(),
            "Reward effects expand the Research pool — milestone wins compound over time."
                .to_string(),
            "Milestone wins automatically grant the next-tier encounter — no scouting needed."
                .to_string(),
        ],
    }
}

/// Hints, strategies, and tips for all disciplines and general gameplay.
///
/// Returns structured advice covering each of the 8 disciplines (Combat, Mining,
/// Herbalism, Woodcutting, Fishing, Rest, Crafting, Research) plus general
/// gameplay tips. Each discipline section includes an overview, key mechanics,
/// named strategies, common pitfalls, and specific tips. Use this guide to
/// discover interesting gameplay patterns and avoid common mistakes.
#[openapi]
#[get("/docs/hints")]
pub fn get_hints() -> Json<HintsGuide> {
    Json(build_hints())
}
