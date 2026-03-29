# Vision & Roadmap Suggestions (from Issue #66 work)

These suggestions are based on learnings from implementing issue #66
(updating balancing goals for gathering disciplines).

## vision.md suggestions

1. **Add a "Deck Size Limits" section** — vision.md should document the
   MaxDeck token pattern as a first-class design principle. Currently five
   decks (Herbalism, Woodcutting, Fishing, Rest, Crafting) start with 52
   cards and the limit is 55. The vision should state whether the goal is
   to keep initial decks near or well-below the cap.

2. **Tier-differentiated yield targets** — the vision mentions gathering
   disciplines but doesn't explicitly state the 2-tier model (T1: simple
   strategies 0.5–2.0, T2: tactical strategies 1.5–4.0). Adding this to
   the vision would make it the canonical source rather than the individual
   balance docs.

3. **Define "tier-2 strategy" precisely** — the vision could benefit from a
   formal definition: "A tier-2 strategy is one that requires the player to
   read game state (hand, encounter tokens, turn number) and make
   conditional decisions, rather than following a fixed rule."

4. **Scouting as a meta-strategy** — clarify in the vision that scouting
   (encounter selection) is a valid tier-2 tactic but does NOT count as
   "beat tier 1 without adjusting yield outcome". This distinction matters
   for ensuring each discipline has internal tactical depth.

## roadmap.md suggestions

1. **Track MaxDeck tuning** — the roadmap should note that MaxDeck=55 is a
   placeholder. A future task should investigate whether initial deck sizes
   should be reduced below 50 or whether 55 is the right permanent value.

2. **Add a "Tier-2 runner implementation" milestone** — while this PR
   documents the requirement for 2+ tier-2 runners per discipline, the
   actual implementation of the second tier-2 runner for each gathering
   discipline is not tracked in the roadmap yet.

3. **Cross-discipline balance comparison** — the roadmap could benefit from
   a task that compares yield/durability ratios across all 4 gathering
   disciplines after individual tuning is complete, ensuring they feel
   equivalently rewarding.

4. **Simulation lower-bound enforcement** — current test bands have very
   permissive lower bounds (0.01) to avoid false failures. A future
   roadmap item should tighten these once actual tuning brings strategies
   above the aspirational 0.5 minimum.
