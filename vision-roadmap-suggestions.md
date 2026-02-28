# Suggestions for vision.md and roadmap.md

Based on implementing Steps 9.1 (CardEffects range system) and 9.2 (CardEffects cost system).

---

## vision.md Suggestions

### 1. Update CardEffectKind description (line ~58)

**Current:** `ChangeTokens { target, token_type, amount }` for token manipulation  
**Should be:** `ChangeTokens { target, token_type, min, max, costs }` for token manipulation with range-based values and optional costs

### 2. Update Library implementation notes (line ~36)

**Current:** "Card definitions use `effect_ids: Vec<usize>` to reference their CardEffect entries in the Library by index."  
**Should be:** "Card definitions use `effects: Vec<ConcreteEffect>` where each ConcreteEffect stores an `effect_id` (reference to a CardEffect entry) and a `rolled_value` (fixed value rolled from the CardEffect's min-max range at creation time). Cost effects also store `rolled_costs: Vec<ConcreteEffectCost>` with rolled percentage values."

### 3. Update Library CardEffect deck description (line ~37)

**Current:** "Every card effect on action/enemy cards references a CardEffect deck entry via `effect_ids: Vec<usize>`."  
**Should be:** "Every card effect on action/enemy cards references a CardEffect deck entry via `effects: Vec<ConcreteEffect>`, where each ConcreteEffect contains the effect_id reference, rolled_value, and optional rolled_costs."

### 4. Add section about two-layer card effect model

Suggest adding a new subsection under "Core gameplay elements" describing the two-layer model:

```
### Card Effect Architecture (Two-Layer Model)

Card effects use a two-layer architecture:

- **Template layer (CardEffect entries):** PlayerCardEffect and EnemyCardEffect entries in the Library define min-max ranges for effect values and optional cost percentage ranges. These are the "blueprints" for concrete cards.
- **Concrete layer (cards with rolled values):** Attack, Defence, Resource, and enemy cards store ConcreteEffect entries with fixed rolled values determined at library initialization. When a card is played, its concrete rolled value is used directly — no per-play randomness.

ConcreteEffect structure:
- effect_id: usize — references the CardEffect template
- rolled_value: i64 — specific value rolled from min-max range at creation time
- rolled_costs: Vec<ConcreteEffectCost> — rolled cost percentages

All values are scaled by ~100x (e.g., damage 500, health 2000, durabilities 10000) to enable meaningful ranges and percentage-based costs.
```

### 5. Add section about cost system

Suggest adding to the combat section:

```
### Cost System

Some card effects have optional costs defined as percentage ranges of the effect value:
- Each cost entry specifies a cost_type (e.g., Stamina) and min-max percentage.
- At card creation, cost percentages are rolled and fixed.
- At play time: cost = rolled_value × rolled_percent / 100.
- If the player can't pay the full cost, the card cannot be played.
- Cost cards are more powerful but require resource management.
- Starting decks are weighted toward non-cost cards.
- Cost system extends to gathering encounters (Mining, Woodcutting stamina costs).
```

### 6. Update token values

Any hardcoded token values in vision.md should reflect the 100x scaling:
- Player health: 2000 (was 20)
- Durabilities: 10000 (was 100)
- Damage ranges: 400-600 (was 5)
- Shield ranges: 200-400 (was 3)

### 7. Add MiningCardEffect and WoodcuttingCardEffect stamina_cost field

The vision references MiningCardEffect with `ore_damage, durability_prevent` — should add `stamina_cost` field. Similarly WoodcuttingCardEffect should mention `stamina_cost`.

---

## roadmap.md Suggestions

### 1. Mark Steps 9.1 and 9.2 as complete

Add completion markers and update playable acceptance with actual results:

**Step 9.1:** ✅ Completed
- All cards use ConcreteEffect with rolled values from min-max ranges
- All numeric values scaled ~100x
- Deterministic rolling via game seed RNG
- 35 cards total in library (was 29)
- All scenario tests pass with scaled values

**Step 9.2:** ✅ Completed
- Cost cards for Attack (id 31), Defence (id 32), Mining (id 33), Woodcutting (id 34)
- Cost PlayerCardEffects (ids 29-30) with 30-50% Stamina cost ranges
- Pre-validation prevents card consumption on insufficient resources
- 4 new scenario tests verify cost mechanics
- Starting decks: cost cards at deck:5 vs non-cost at deck:15

### 2. Add implementation insight to Step 9.3 (Rest encounter)

Step 9.3 will benefit from awareness of:
- The ConcreteEffect model is already in place — rest cards should use the same pattern
- Stamina token is already functional and tested as a cost currency
- The cost pre-validation pattern (preview_costs/preview_stamina_cost) is established

### 3. Note about "stuck encounter" edge case

The implementation revealed an edge case: when all remaining hand cards are cost cards and the player has no stamina, the encounter gets stuck (can't play any cards but encounter is still Undecided). The current workaround is for the player to use EncounterAbort. A future improvement could be:
- Auto-detect when no playable cards remain and offer a forced pass or auto-loss
- This affects both combat and gathering encounters
- Consider adding this to a future roadmap step (perhaps as part of UX polish)

### 4. Update Step 9.2 design rules with implementation details

The roadmap says "at least one cost variation for each non-cost variation of attack and defence cards." Current implementation has:
- 1 cost Attack variant (vs 1 non-cost) ✓
- 1 cost Defence variant (vs 1 non-cost) ✓
- 1 cost Mining variant (vs 3 non-cost) ✓
- 1 cost Woodcutting variant (vs 4 non-cost) ✓
- No cost Herbalism or Fishing variants (as per roadmap spec)

Future steps could add more cost variants with different cost percentages for more strategic depth.
