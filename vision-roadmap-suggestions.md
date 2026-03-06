# Vision and Roadmap Improvement Suggestions

Generated during the post-9.5 cleanup pass (issues.md resolution + vision-roadmap-suggestions implementation).

## Vision.md suggestions

1. **Add Rest encounter template to Concrete examples section**: The Concrete examples section (after Mining, Woodcutting, Herbalism, Fishing) does not include a Rest encounter template. Rest is now a full encounter type with its own cards, tokens, and mechanics — it deserves a template entry like the other disciplines.

2. **Document the split_gathering_costs() pre-play/post-play pattern**: The two-phase cost model (pre-play costs checked before card play, post-play costs like durability applied after) is a key design pattern for gathering disciplines but is only mentioned briefly inline. A dedicated paragraph under the encounter architecture would make this clearer for future implementers.

3. **Clarify enemy deck generation vs player deck generation**: Vision.md documents player decks well but says little about how enemy decks are composed (number of cards, effect distribution, initial token values). Since scouting (Step 11) will modify enemy decks, having a clear baseline description would help.

4. **Add a "Reproducibility contract" section**: The seed + action log reproducibility is a core design principle referenced in multiple places. A short dedicated section stating the exact contract (same seed + same action log + same version = identical game state) would consolidate scattered references.

5. **Document the MaxHand token pattern**: Each discipline has a `*MaxHand` token (AttackMaxHand, DefenceMaxHand, MiningMaxHand, etc.) that controls draw limits. This is a progression mechanic (milestones can increase MaxHand) but isn't called out as a design pattern.

## Roadmap.md suggestions

1. **Step 9.5 implementation checklist is redundant**: The Step 9.5 section still contains the full implementation checklist (10 items) even though it's marked COMPLETED. Consider collapsing the checklist into a one-line "Original checklist: 10 items, all completed" note, since the implementation summary already covers what was done.

2. **Add sub-step numbering to Steps 10-11**: Steps 10 and 11 are large (Research and Scouting). Following the guideline about splitting large steps, consider pre-numbering sub-steps (10.1, 10.2, etc.) for better progress tracking.

3. **Cross-reference completed mechanics in future steps**: Steps 12-16 reference mechanics (player death, conclude pattern, token scoping) that are now implemented. Consider adding explicit "Prerequisite: implemented in Step X" cross-references to make dependencies clearer.

4. **Consider a "Current game state summary" section**: After all the implementation updates, a short section summarizing what the game currently looks like (what a new player sees, what encounters exist, how many cards) would help orient new contributors without reading the full history.

5. **Step 8 sub-steps lack COMPLETED markers**: Steps 8.1-8.4 are listed as "COMPLETE" but the parent "Step 8 implementation updates" section header doesn't reflect this. Consider adding "Step 8 — COMPLETE" to the header.

6. **Gather "open items" into one place**: Several completed steps have leftover "Open items" notes (e.g., mining min/max ranges). Consider moving unresolved open items to the technical debt section or a dedicated "Open design questions" section.
