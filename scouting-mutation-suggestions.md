# Suggestions for vision.md and roadmap.md

Based on the scouting mutation implementation, here are suggested updates.

## vision.md suggestions

### 1. Update "Scouting / Recon (system)" section (~line 339)

The current description says scouting "generates Foresight and other reconnaissance benefits" and "can affect resource yields." The actual implementation now generates **mutated encounter variations** based on the just-completed encounter. Suggest updating to:

> Scouting / Recon (system): After any encounter resolves, the scouting phase generates 3 mutated variations of the just-completed encounter in addition to drawing from the encounter deck. Mutations apply difficulty deltas (parameter scaling) and modify ~20% of enemy deck cards while preserving total card counts. Players choose their next encounter from both the mutated variations and existing encounter hand cards. Foresight controls the encounter hand size; scouting mutation is an additional mechanic layered on top.

### 2. Update "Scouting candidate pool" and "Scouting pick count" tokens (~lines 287-289)

These tokens describe a CardEffect-pool-based generation model (draw X effects, pick Y). The current implementation uses a clone-and-mutate model instead. Suggest noting:

> Current implementation: scouting generates encounter variations by cloning and mutating the source encounter (parameter scaling + enemy deck mutation) rather than drawing from a CardEffect pool. The CardEffect-pool model may be revisited in the future as an alternative or complementary system.

### 3. Add "Enemy deck mutation" to the deck/card model description

The vision describes encounter generation at game start but doesn't cover post-encounter mutation of enemy decks. Suggest adding near line 232:

> Enemy deck mutation: during scouting, ~20% of enemy deck entries may be mutated via three operations — ScaleValues (scales effect values), RedistributeCopies (moves copies between entries while keeping total count constant), and SwapTier (copies effects from adjacent entries). This preserves the total card count invariant while creating meaningful difficulty variations.

## roadmap.md suggestions

### 1. Update Step 5 "Encounter replacement and scouting hooks" (~line 138)

This step describes replacement-generation and scouting mechanics as future work. The scouting mutation feature partially implements this. Suggest marking it as "partially implemented":

> Status: Partially implemented via the scouting mutation system (feature/scouting-mutation branch). The mutation system clones and modifies the just-completed encounter rather than generating entirely new encounters from CardEffect pools. The CardEffect-pool replacement model from the vision remains unimplemented and may be a complementary future system.

### 2. Update Step 7 loop description (~line 154)

The loop description says "perform a scouting post-resolution step that biases replacement." The implementation now generates 3 mutated encounter variations. Suggest updating:

> Scouting post-resolution now generates 3 mutated variations of the completed encounter (parameter scaling + enemy deck mutation) as additional choices alongside the normal encounter hand. The pick → fight → scouting → pick loop is fully operational with this mutation-based scouting.

### 3. Add new roadmap entry for scouting mutation refinements

Potential future work discovered during implementation:

> **Scouting mutation refinements (future)**
> - Token-gated mutation depth: let scouting-related tokens (Foresight, Scouting candidate pool) influence mutation intensity (wider delta ranges, more deck entries mutated)
> - Cross-discipline mutation: when the player has high Insight in multiple disciplines, scouting could offer encounters from different disciplines than the one just completed
> - Milestone-aware scouting: milestone encounters currently skip mutation (they go to NoEncounter, not Scouting); consider whether milestone completion should also trigger scouting choices
> - Herbalism characteristic mutation could be extended to other disciplines that have discrete/categorical properties
