# Suggestions for vision.md and roadmap.md

Based on findings from the refactor-token-scope-costs-tests worktree.

## vision.md

### 1. Update Cost System section — `is_absolute` removed from ConcreteEffectCost

The Cost System section should note that `ConcreteEffectCost` now stores pre-computed absolute amounts (field `amount: u32`). The `is_absolute` flag was removed from `ConcreteEffectCost` — percentage-to-absolute conversion now happens at roll time in `roll_costs()`, not at play time. `CardEffectCost` templates still have `is_absolute` to distinguish percentage ranges from absolute ranges during rolling.

**Affected section:** "Cost System" — update to reflect the two-layer cost model:
- Template layer (`CardEffectCost`): `min_percent`, `max_percent`, `is_absolute`
- Concrete layer (`ConcreteEffectCost`): `amount` (always absolute, pre-computed at roll time)

### 2. Remove `card_value` references from ConcreteEffect

`card_value: Option<i64>` was removed from `ConcreteEffect` — it was always `None` and never assigned. Remove any references to it as a cost-base override mechanism.

### 3. Update encounter-scoped token documentation

The `is_encounter_scoped()` method on `TokenType` now correctly covers all encounter-scoped tokens:
- RestToken, RestMaxHand
- EnemyAttackMaxHand, EnemyDefenceMaxHand, EnemyResourceMaxHand
- FishingRangeMin, FishingRangeMax, FishAmount
- MiningLightLevel, MiningYield, MiningPower
- CraftingToken, CraftingMaxHand
- MilestoneMaxHand

Vision.md already documents which tokens are encounter-scoped (section on encounter-scoped token storage), but should note that `is_encounter_scoped()` now enforces this consistently in cost-checking logic.

### 4. Update milestone encounter description

Milestones no longer use a `MilestoneScouting` phase. On win, a single next-tier encounter is auto-assigned. Remove references to "pick 1 of 3 next-tier milestones" or "3 scouting choices".

## roadmap.md

### 1. Update Step 13 (Milestone Encounters) win flow description

**Line ~648:** Change:
> Win flow: On win, the player receives 50%-improved versions of all existing PlayerCardEffects for that discipline (expanding the research pool), then enters MilestoneScouting with 3 next-tier milestone variations to choose from.

To:
> Win flow: On win, the player receives 50%-improved versions of all existing PlayerCardEffects for that discipline (expanding the research pool), then a single next-tier milestone encounter is auto-assigned for that discipline.

### 2. Update player actions list

**Line ~14:** The `MilestonePickScoutingChoice` action no longer exists. The current player actions are: NewGame, EncounterPickEncounter, EncounterPlayCard, EncounterApplyScouting, EncounterAbort, EncounterConcludeEncounter, EncounterCraftSwap, EncounterCraftCard, EncounterCraftDurability, ResearchChooseProject, ResearchSelectCandidate, ResearchProgress, ResearchPlayHand, ResearchConcludeExperiment.

### 3. Add note about test restructuring

Add a note in the implementation updates section:
> Tests restructured: `tests/scenario_tests.rs` split into `tests/scenario_tests/` folder with per-discipline modules (combat, mining, herbalism, woodcutting, fishing, rest, crafting, research, milestone, costs, api). Milestone tests migrated from `tests/milestone_tests.rs` into the same folder using shared helpers.

### 4. Add note about ConcreteEffectCost simplification

Add BREAKING change note:
> BREAKING: `ConcreteEffectCost.is_absolute` removed; `rolled_percent` renamed to `amount`. All concrete costs are now pre-computed as absolute values at roll time. `ConcreteEffect.card_value` removed (was always None). API responses for card effects will have `amount` instead of `rolled_percent`/`is_absolute` fields.
