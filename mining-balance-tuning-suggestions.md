# Suggestions for vision.md and roadmap.md

Based on findings from the mining balance tuning session (branch: feature/mining-balance-tuning).

## vision.md Suggestions

### 1. Document the "Light Card Trap" Dynamic

During mining tuning, we discovered that playing non-yield cards (e.g., light-boosting cards) is always suboptimal because each round triggers an ore card (~19 durability cost) regardless of what the player plays. A round spent on a non-yield card generates 0 yield for the same durability cost. This generalizes to all gathering disciplines: **any card that doesn't directly produce yield wastes its round's durability cost**. Consider documenting this as a design constraint or as a future design challenge — if utility cards are always traps, the strategic depth of mining is limited to "play power cards and conclude early."

### 2. Note That Tactician and Durability Tactician Converge

Both Tier-2 mining strategies converge on the same core behavior: play one power card at peak light (200) and conclude immediately (~0.5 rounds/encounter). This suggests the current mining mechanic doesn't support differentiated tactical approaches. The vision could note this as a future design goal — making light management or multi-round strategies viable by, for example, adding a "warm-up" mechanic where yield scales with rounds played, or making early-conclude less efficient.

### 3. Clarify "Non-Yield Tactician" Definition

The mining_balance.md requires a "non-yield tactician" that wins without "yield-boosting effects." In practice, the Durability Tactician still plays power cards (which produce yield) — it just avoids cost cards. The distinction from the Yield-optimizer is very thin. Consider refining the definition of "non-yield tactician" in the vision to better specify what differentiates T2 strategies from each other.

## roadmap.md Suggestions

### 1. Add a "Balance Config Tuning" Step After Each Simulation Runner

The roadmap marks B2.4 (Mining simulation runner) as complete, but building the runner and tuning the config to meet targets are separate efforts. The runner was complete but all strategies were below target ranges. Consider adding explicit "tune to targets" steps (e.g., B2.4.1: Mining config tuning) to track this work separately from runner creation.

### 2. Consider a "Mechanic Review" Step After Balance Tuning

The mining tuning revealed that the core mining mechanic (play card → ore plays → repeat) doesn't reward multi-round strategies because each round's fixed durability cost makes "play one card and leave" optimal. A mechanic review step could evaluate whether the gather-conclude loop needs design changes to create richer strategic depth (e.g., increasing yield over consecutive rounds, reducing ore cost for early rounds, or adding encounter-specific bonuses).

### 3. Track Cross-Strategy Convergence as a Balance Health Metric

If multiple strategies produce nearly identical results (Tactician 2.27 vs Dur. Tactician 2.17 via the same behavior), it suggests limited strategic depth. The roadmap could include a periodic "strategy differentiation check" — verifying that different strategies actually use different mechanics rather than converging on the same dominant play pattern.
