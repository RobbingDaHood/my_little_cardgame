# Vision & Roadmap Suggestions — Milestone Encounters

Based on the implementation of Step 13 (Milestone Encounters), here are suggested improvements to `docs/design/vision.md` and `docs/design/roadmap.md`:

## Changes Already Applied

### vision.md
- **Line 167**: Removed "may factor into future mechanics (e.g., milestone difficulty scaling)" from PlayerDeaths description. Death counter now purely tracks lifetime deaths for statistics.

### roadmap.md
- **Step 13**: Replaced speculative description with implemented design:
  - Per-discipline milestones (Combat, Mining, Herbalism, Woodcutting, Fishing)
  - Difficulty scales by tier only (not deaths)
  - Exponential insight cost: `100 * 2^(tier-1)`
  - Win → 50% better CardEffects + 3 next-tier scouting choices
  - Loss → reset + NoEncounter (no forced replay)
  - Dedicated milestone hand (max 5 via MilestoneMaxHand)
- **Line 629**: Removed death-based milestone difficulty scaling implementation note.

## Suggested Additions

### vision.md — New Section: "Milestone Progression"
Consider adding a section after "Player death and recovery" that describes the milestone system:

> ### Milestone progression
>
> Milestone encounters are the primary CardEffect progression system. Each combat/gathering
> discipline has a dedicated milestone track with escalating tiers:
>
> - **Entry cost**: MilestoneInsight tokens (100 × 2^(tier−1)) — earned from combat wins.
> - **Win reward**: 50%-improved versions of all existing PlayerCardEffects for that discipline
>   are added to the research pool. The player picks 1 of 3 next-tier milestone variations.
> - **Loss handling**: The encounter resets and returns to the milestone hand — no forced replay.
> - **Progression**: Each tier wraps a harder version of the discipline's encounters with
>   stats scaled by 1.5^(tier−1).
>
> Milestones live in a dedicated hand separate from regular encounters, capped at 5 cards
> via the MilestoneMaxHand token. Utility disciplines (Rest, Crafting, Research) do not
> have milestones.

### vision.md — Token Enum Update
The `MilestoneMaxHand` token should be added to the token enum list at line 137:
- Add `MilestoneMaxHand` to the TokenType enum documentation.

### roadmap.md — Future Work
Consider adding to Step 14 or a new step:
- **Milestone prerequisite chains**: Some milestones could require beating other discipline
  milestones first (cross-discipline synergies).
- **Milestone-specific cards**: Special milestone-only player cards that provide bonuses
  specifically during milestone encounters.
- **Milestone leaderboard/metrics**: Track milestone tier progression in `/metrics` endpoint.
