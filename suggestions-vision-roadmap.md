# Suggestions for vision.md and roadmap.md — Issue 6 (Enemy draws from hand)

## Suggested changes to vision.md

1. **Add enemy deck lifecycle description**: vision.md line 44 says "Enemy turns play one random card matching the current CombatPhase." This should be updated to reflect the new hand/deck/discard model:
   > Enemy decks use `EnemyCardCounts { deck, hand, discard }` to track card locations. At combat start, hand cards are shuffled back into the deck and redrawn randomly. Enemies play from hand only; played cards go to discard. Resource cards with DrawCards effects draw from deck to hand. When a deck is empty, the discard pile is recycled into the deck.

2. **Document `EnemyCardDef` has counts**: vision.md line 22 mentions enemy card definitions are inline within Encounter definitions. Add that each `EnemyCardDef` includes `counts: EnemyCardCounts` to track card location state during combat.

3. **Document enemy deck state on CombatState**: vision.md line 40 says "Combat state lives in GameState (current_combat: Option<CombatSnapshot>...)". Update to note that `CombatState` now holds mutable copies of enemy decks (`enemy_attack_deck`, `enemy_defence_deck`, `enemy_resource_deck`), cloned from `CombatantDef` at combat start.

4. **Clarify `start_combat` signature change**: vision.md line 40 references `start_combat` — note it now requires an RNG parameter for shuffling enemy hands at combat start.

## Suggested changes to roadmap.md

1. **Add a post-implementation update entry** for this work:
   > ### Enemy hand management (Issue 6)
   > - Added `EnemyCardCounts { deck, hand, discard }` to `EnemyCardDef` for enemy card location tracking.
   > - `CombatState` now holds mutable copies of enemy decks, cloned from `CombatantDef` at combat start.
   > - At combat start, enemy hands are shuffled: all cards move to deck, then random cards drawn to restore hand size.
   > - Enemy plays from hand only; played cards move to discard.
   > - Resource cards with `DrawCards` effect trigger draws from deck to hand, with discard-to-deck recycling.
   > - `start_combat` now takes an RNG parameter.
   > - Gnome enemy updated: 10 copies per card type in hand, resource card includes DrawCards effect.

## Contradictions and areas for improvement

1. **Ambiguity in "the deck" for enemy draws**: issues.md says "take a random card from 'the deck'" when resource card triggers. It's unclear whether this means the resource deck specifically, or any enemy deck. Currently implemented as drawing from the resource deck only. Vision.md should clarify this.

2. **No mechanism for attack/defence deck replenishment**: After initial hand is played, attack and defence decks have no draw mechanism (only resource cards trigger draws). Vision.md should document whether this is intentional or if a round-based draw-to-hand mechanic should exist.

3. **CombatState naming**: vision.md alternates between "CombatSnapshot" and "CombatState". The code uses `CombatState`. Vision.md should consistently use `CombatState`.
