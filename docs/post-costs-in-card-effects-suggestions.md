# Vision & Roadmap Suggestions — Costs-in-Card-Effects Refactoring

Suggested updates to `docs/design/vision.md` and `docs/design/roadmap.md` based on the costs-in-card-effects refactoring.

---

## vision.md Updates

### Cost System section (~line 193)

**Current text** describes the three-step rolling pipeline but doesn't mention `is_absolute` costs or encounter-scoped costs.

**Suggested update**: Replace/extend the Cost System section with:

> Card effects use a rolling pipeline to determine values and costs at creation time:
>
> 1. **Roll cap:** From the CardEffect template's `cap_min..cap_max` range, producing `rolled_cap`.
> 2. **Roll value:** `rolled_value = rolled_cap * rolled_gain_percent / 100`.
> 3. **Roll costs:** For each `CardEffectCost { token_type, min_percent, max_percent, is_absolute }`, roll to produce `ConcreteEffectCost { token_type, rolled_percent, is_absolute }`.
>
> At play time, cost resolution depends on the `is_absolute` flag:
> - **Percentage costs** (`is_absolute: false`): `cost = cost_base × rolled_percent / 100` where `cost_base = card_value.unwrap_or(rolled_value)`. Used for effects with meaningful numeric values (GainTokens, FishingValue).
> - **Absolute costs** (`is_absolute: true`): `cost = rolled_percent` (the rolled value IS the cost amount). Used for effects without meaningful numeric values (HerbalismMatch, WoodcuttingChop, CraftingReduction) and encounter-scoped costs (RestToken).
>
> Encounter-scoped costs (e.g., `TokenType::RestToken`) are deducted from encounter state, not player `token_balances`. `TokenType::is_encounter_scoped()` identifies these.

### CardEffectKind section (~line 142)

**Add**: Each `CardEffectKind` variant that participates in gathering can carry `costs: Vec<CardEffectCost>`. The following variants support costs: `GainTokens`, `LoseTokens`, `FishingValue`, `WoodcuttingChop`, `HerbalismMatch`, `CraftingReduction`. Costs are defined on the template and rolled by `roll_concrete_effect()` — no post-hoc cost injection.

### Card architecture notes (~line 59)

**Update** the text about shared vs discipline-specific effects to note:
- `compute_*_card_value()` and `apply_*_costs()` helper functions have been removed from all disciplines.
- Cost definitions are now self-contained within CardEffectKind templates. Each template variant carries its costs directly.
- When the same effect type needs different cost combinations (e.g., HeavyChop with dur-only vs dur+stam), separate templates are created for each cost combination. Original no-cost templates are kept for non-first effect positions in multi-effect cards.

### Rest cards (~line 147)

**Update**:
> - **Rest action cards:** Rest — with ConcreteEffect/GainTokens pattern; material costs via CardEffectCost; encounter-scoped RestToken costs via CardEffectCost with `is_absolute: true`. `CardKind::Rest { effects }` has the same shape as all other gathering card kinds (no standalone `rest_token_cost` field).

### TokenType enum documentation

**Add** note about the `RestToken` variant and the `is_encounter_scoped()` method pattern. This establishes the convention for future encounter-scoped cost tokens.

### Line ~177

**Current**: "The unifying pattern for all card effects across all disciplines is now ConcreteEffect + library templates ... with costs expressed as LoseTokens effects and gains as GainTokens effects."

**Suggested fix**: Costs are no longer expressed as LoseTokens effects. Update to:
> "...with costs expressed as `CardEffectCost` entries on the CardEffectKind template, rolled into `ConcreteEffectCost` at creation time."

---

## roadmap.md Updates

### Step 9.2 — CardEffects cost system (~line 345)

**Current** describes the original percentage-based cost system. Extend with:

> **Post-refactoring (costs-in-card-effects branch):**
> - All discipline costs are now defined IN the CardEffectKind template, not applied post-hoc via `apply_*_costs()` helpers.
> - `CardEffectCost` supports both percentage-based costs (`is_absolute: false`) and absolute costs (`is_absolute: true`).
> - Percentage costs work well for effects with meaningful `rolled_value` (GainTokens, FishingValue). Absolute costs are used when the effect's `rolled_value` is zero or tiny (HerbalismMatch, WoodcuttingChop 1-9, CraftingReduction 20-40).
> - `rest_token_cost` moved from `CardKind::Rest` into the effect model as `TokenType::RestToken` with `is_absolute: true`.
> - `compute_*_card_value()` / `apply_*_costs()` patterns eliminated from all 4 gathering disciplines + rest.
> - When the same effect type needs different cost combos across cards, separate templates are created (e.g., `wc_heavy_chop_dur_id` vs `wc_heavy_chop_dur_stam_id`).

### Fishing step (~line 290)

**Update** the description to note costs are now in FishingValue templates, not separate `apply_fishing_costs()` calls. Remove mention of `costs: Vec<GatheringCost>` vectors (replaced by `CardEffectCost` on templates).

### Herbalism step (~line 214)

**Update**: Remove mention of `costs: Vec<GatheringCost>` on the CardKind. Note that costs are on HerbalismMatch templates with `is_absolute: true`.

### Woodcutting step (~line 241)

**Update**: Remove mention of `costs`/`gains` vecs. Note that costs are on WoodcuttingChop templates with `is_absolute: true`.

### Crafting step (if present)

**Update**: Note costs are on CraftingReduction templates with `is_absolute: true`.

### Potential new roadmap item

Consider adding a future step:
> **Rename `min_percent`/`max_percent` → `min`/`max` and `rolled_percent` → `rolled_value`**: The current field names are misleading when `is_absolute: true`. A rename would improve clarity. Low priority since the `is_absolute` flag disambiguates, but would improve code readability.
