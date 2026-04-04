# Vision & Roadmap Improvement Suggestions

Based on the auto-draw documentation audit (April 2026).

## vision.md Suggestions

1. **Crafting encounter section (6. Fabrication/Weaponcraft) is aspirational-only**: Lines 1004-1016 describe a vision-state crafting system (Hammer, Temper, Anneal actions) that differs from the current implementation (CraftingReduction cards, material cost inflation). Consider updating this section with a "Current implementation" block (like Mining, Woodcutting, Herbalism, Fishing have) to reflect the actual Step 10 crafting system, then keep the aspirational version as a "Future refined version."

2. **Provisioning section (7) has no implementation marker**: The Provisioning encounter type (lines 1018-1033) is entirely aspirational with no "IMPLEMENTED" tag. Consider adding a "NOT YET IMPLEMENTED" marker to make this clear, consistent with the Research section which has "✅ IMPLEMENTED."

3. **ResearchMaxHand token is missing**: Unlike all other disciplines that have a `{Discipline}MaxHand` token (MiningMaxHand, FishingMaxHand, CraftingMaxHand, RestMaxHand, etc.), Research has no `ResearchMaxHand` token. The research draw implementation also doesn't use `draw_player_cards_of_kind()` (it uses a manual sequential loop). Consider adding `ResearchMaxHand` for consistency and using the shared draw function to align with other disciplines.

4. **Research draw is sequential, not random**: The research card draw implementation (research.rs:527-540) draws cards in library index order rather than randomly. All other disciplines use `draw_player_cards_of_kind()` which selects randomly from available deck cards. This is a minor inconsistency that could matter if research card order affects strategy.

## roadmap.md Suggestions

1. **Consider a "Card draw audit" task**: Now that the auto-draw rule is documented, a future roadmap item could formalize the draw pattern as a shared abstraction. Currently each discipline implements its own `draw_player_{discipline}_card()` wrapper around `draw_player_cards_of_kind()`. A trait or shared helper could enforce the pattern and reduce per-discipline boilerplate.

2. **Research draw alignment**: Consider adding a small task to align Research's draw implementation with the standard `draw_player_cards_of_kind()` pattern used by all other disciplines, including adding a `ResearchMaxHand` token.
