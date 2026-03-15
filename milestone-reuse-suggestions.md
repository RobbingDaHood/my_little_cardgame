# Vision & Roadmap Suggestions — Milestone Reuse Discipline Effects

Based on the refactoring that unified milestone card effects with discipline effects.

---

## Suggestions for `docs/design/roadmap.md`

### 1. Update Step 13 description to reflect the unified effect model

The current Step 13 description says milestones are "a tougher version of that discipline's encounters" but doesn't mention the implementation detail that milestones **reuse the discipline's existing card effects**. Suggested addition to the Step 13 notes:

> **Effect reuse**: Milestone encounters do not define their own card effects. Instead, they look up the discipline's existing `EnemyCardEffect` templates at the target tier and use `roll_best_concrete_effect` (always picking maximum values) to produce the most powerful version of each effect. This ensures milestones are always calibrated to the discipline's current tier and that any config-driven changes to a discipline's effects automatically flow into its milestone encounters.

### 2. Add "tier tracking" to the implementation notes

Step 13 doesn't mention the `tier` field on `LibraryCard`. Suggested note:

> **Tier tracking**: `LibraryCard` now carries a `tier: u32` field (default 1). When milestone rewards are generated (50% scaled copies), the new effects are tagged with `tier + 1`. This enables tier-aware lookups (`card_effects_for_discipline_and_tier`, `enemy_effects_for_discipline_and_tier`) so that each tier's milestone can find the correct effect templates.

### 3. Mention that milestone reward scaling covers both player AND enemy effects

The current description says "50%-improved versions of all existing PlayerCardEffects". The refactored code now also scales EnemyCardEffects. Suggested clarification:

> **Win flow**: On win, the player receives 50%-improved versions of all existing **PlayerCardEffects and EnemyCardEffects** for that discipline. The scaled EnemyCardEffects are tagged at the next tier so the next milestone encounter can look them up and use them as its enemy cards.

### 4. Consider a future step for "config-driven milestone parameters"

The milestone environment parameters (enemy HP, light level, fish range, etc.) are still computed in code with tier-based scaling formulas. A natural follow-up would be:

> **Future**: Consider externalizing milestone environment scaling (enemy HP multipliers, light level adjustments, fishing range spans, etc.) into JSON configuration alongside the discipline's encounter configs. This would complete the separation between "what effects to use" (already config-driven via discipline effects) and "how to scale the environment" (still code-driven).

---

## Suggestions for `docs/design/vision.md`

### 1. Add a "Tier Progression" concept to the token/progression section

The vision document describes milestone encounters and CardEffect-Choice/Picks tokens but doesn't explicitly describe the **tier system** as a first-class concept. Suggested addition near the "Goals and milestones" section:

> **Tier Progression**: Each discipline's card effects exist at numbered tiers. Tier 1 effects are the base set loaded from configuration. When a player beats a tier-N milestone, all discipline effects are scaled by 50% and registered at tier N+1. Milestone encounters at tier N+1 then use the **best possible roll** (maximum values) of the tier-N+1 effects as their enemy cards. This creates a clear power curve: each milestone is calibrated to be exactly as strong as the best version of what the player just unlocked.

### 2. Clarify the "best version" concept in the CardEffect model description

The vision describes the two-layer CardEffect model (templates → rolled concrete values) but doesn't mention the "roll best" variant. Suggested note in the CardEffect section:

> **Roll best variant**: In addition to random rolls within effect ranges, the system supports `roll_best_concrete_effect` which always selects the maximum values. This is used for milestone encounters so that enemy cards represent the ceiling of the current tier's power.

### 3. Update milestone description to emphasize effect reuse

The vision's milestone description (line ~357) says milestones require "good decks and strategic play." Consider adding:

> Milestone enemy cards are derived directly from the discipline's effect templates at the current tier, ensuring milestone difficulty scales naturally with the discipline's progression curve rather than using independently-tuned parameters.
