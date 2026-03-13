# Post-Step-10 Suggestions for vision.md and roadmap.md

## Sections that are getting too long

1. **vision.md line ~120 (Mining encounter mechanics):** This is a single bullet point spanning 10+ lines with sub-bullets covering light-level mechanics, turn flow, conclude logic, lose conditions, MiningDef, MiningDurability, and endpoints. Consider splitting into a dedicated `### Mining Encounter` subsection with clear sub-headings for each concern.

2. **vision.md Token identifiers list (line ~136):** The `TokenType` enum description is a single massive paragraph listing 40+ tokens with inline explanations. Consider restructuring as a table or categorized list (persistent tokens, encounter-scoped tokens, max-hand tokens, insight tokens, material tokens).

3. **roadmap.md Post-9.3 implementation (lines 446-461):** This changelog entry has 15+ bullet points covering many unrelated changes (GainTokens/LoseTokens split, stamina_grant removal, durability_cost removal, HerbalismMatchMode, MaxHand changes, autoloss, DRY refactors, discipline module splits). Consider splitting into sub-sections by theme (breaking API changes, internal refactors, test changes).

4. **roadmap.md Post-Step-10 implementation batch (lines 570-584):** Similarly dense. Would benefit from grouping into "Breaking changes", "New features", and "Cleanup/testing" sub-sections.

## Inconsistencies found during the update

1. **vision.md line ~74 CardKind listing is stale:** The Library implementation notes paragraph lists CardKind variants as `(Attack{effects}, Defence{effects}, Resource{effects}, Encounter{kind: EncounterKind}, PlayerCardEffect{kind: CardEffectKind}, EnemyCardEffect{kind: CardEffectKind})` — this omits Mining, Crafting, Herbalism, Woodcutting, Fishing, and Rest variants.

2. **vision.md CardEffectKind count:** Line 141 previously said "four variants" but there are now eight (GainTokens, LoseTokens, DrawCards, Insight, WoodcuttingChop, HerbalismMatch, FishingValue, CraftingReduction). This was fixed in this update, but the paragraph at line ~74 also references CardEffectKind without listing the newer variants.

3. **vision.md Gathering Token Amount section title:** Was renamed to "Gathering Card Effect Model (Library-Referenced)" in this update, but the `TokenAmount` struct description on line ~172 still references the old role. The section could be further clarified about what TokenAmount is still used for (encounter token initialization, internal operations) vs. what replaced it (ConcreteEffect).

4. **roadmap.md `split_gathering_costs()` reference (line 450):** The Post-9.3 changelog still describes `split_gathering_costs()` as being "added" — but the vision.md now correctly no longer references it (since costs are now ConcreteEffect-based). The roadmap should note that this helper was later superseded.

5. **vision.md `all_gathering_hand_cards_unpayable()` (line ~458 in roadmap):** The Post-9.3 entry describes unifying four methods into `all_gathering_hand_cards_unpayable()`, but this method was then replaced by `all_effects_hand_cards_unpayable()` in the post-Step-10 fixes. The roadmap changelog captures this progression, but a reader only looking at vision.md might miss the history.

## Areas where the docs could better describe the current architecture

1. **Unified card effect model diagram:** Now that ALL disciplines use the same `effects: Vec<ConcreteEffect>` → PlayerCardEffect template pattern, a concise architecture summary showing this uniformity would be valuable. Currently the reader has to piece this together from multiple scattered sections (Card Effect Architecture, Gathering Card Effect Model, per-discipline descriptions).

2. **CardEffectKind variant reference table:** With 8 variants now, a reference table listing each variant, which disciplines use it, and what it does would be more scannable than the current inline paragraph.

3. **PossibleAction/actions endpoint:** The `/actions/possible` endpoint is mentioned in the Post-Step-10 batch but isn't prominently described in vision.md's endpoint section. Worth adding to the endpoint reference.

4. **Cost classification (pre-play vs post-play):** This important concept appears in the Gathering Card Effect Model section but isn't cross-referenced from the per-discipline encounter descriptions. Each discipline section should link back to or briefly mention this classification.

5. **Discipline-specific mechanics carried by CardEffectKind:** The new variants (WoodcuttingChop, HerbalismMatch, FishingValue, CraftingReduction) carry mechanics that were previously in dedicated structs. vision.md should describe what data each variant carries — currently only mentioned in passing.

## Suggested reorganization ideas

1. **Extract a "Card System" top-level section in vision.md:** Currently card-related information is spread across "Implementation details", "Card Effect Architecture", "Cost System", "Gathering Card Effect Model", and per-discipline descriptions. A unified "Card System" section with sub-sections for CardKind, CardEffectKind, ConcreteEffect, and the cost pipeline would reduce duplication and improve discoverability.

2. **Separate "Current Implementation" from "Future Vision" in discipline descriptions:** Each discipline has a "Current simplified implementation" and a "Future refined version" block. Consider splitting these into two separate sections (or even files) so readers can quickly see current state vs. aspirational design.

3. **Move changelog entries from roadmap.md to a dedicated CHANGELOG.md:** The roadmap is growing with implementation changelog entries (Post-7.6, Post-7.7, Post-9.3, Post-Step-10, Post-Step-10 fixes). These are valuable history but make the forward-looking roadmap harder to navigate. Consider a separate changelog file and keeping roadmap.md focused on planned steps.

4. **Add a "Quick Reference" section to vision.md:** A condensed reference covering the current CardKind variants, CardEffectKind variants, TokenType categories, and key endpoints would help developers orient quickly without reading the full document.
