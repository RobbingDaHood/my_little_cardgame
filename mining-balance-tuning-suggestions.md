# Suggestions for vision.md and roadmap.md

Based on findings from the mining balance tuning session (branch: feature/mining-balance-tuning), updated after PR review feedback and design refinements.

## vision.md Suggestions

### 1. Document the Utility Card Payback Principle

Every utility card (light boost, stamina gain, etc.) must be able to pay back its round cost — the fixed durability damage from the environment card that plays regardless of the player's choice. This is now codified in `balance-tuning-tips` and `mining_balance.md`, but the core vision document should elevate it as a cross-discipline design principle. **Insight cards are the sole exception**, as they intentionally increase difficulty. If a utility card type is never worth playing, the underlying mechanic needs redesign, not the card values.

### 2. Light Level as Ramping Yields — A Design Direction

Mining's light-level mechanic was intended to create a ramping-yields dynamic where investing in light management unlocks higher yields over multiple rounds. Currently, all Tier-2 strategies converge on "play one power card at peak light and conclude immediately" because the fixed per-round durability cost makes multi-round play suboptimal. The vision should document light-as-ramping-yields as an explicit design goal, along with the four candidate directions now in `mining_balance.md`: lower initial light, partial yield on light cards, slower decay, and threshold bonuses.

### 3. Full Utilization as an Aspirational Strategy

A new "Full Utilization" strategy has been added to the mining balance doc — one where every card type is optimally used (except insight cards). The vision should note that **no card type should be worthless** as a general design constraint. If a strategy that ignores a card type always outperforms one that uses it, the mechanic needs adjustment. This applies across all gathering disciplines, not just mining.

### 4. Clarify Non-Yield Tactician Definition

The mining_balance.md requires a "non-yield tactician" that wins without yield-boosting effects. In practice, the Durability Tactician still plays power cards (which produce yield) — it just avoids cost cards. The distinction from the yield-optimizer is thin. The vision should refine what differentiates Tier-2 strategies from each other beyond conclude timing.

## roadmap.md Suggestions

### 1. Add Explicit "Tune to Targets" Steps After Simulation Runners

Building a simulation runner (e.g., B2.4 Mining) and tuning configs to meet documented targets are separate efforts. The roadmap should track tuning work explicitly (e.g., B2.4.1: Mining config tuning) so it's clear when a discipline's balance is actually complete versus just measurable.

### 2. Add a "Mechanic Review" Milestone After Initial Balance Tuning

Mining tuning revealed that the core gather-conclude loop doesn't reward multi-round strategies. Before investing heavily in per-discipline tuning, a mechanic review step should evaluate whether the encounter loop needs design changes (ramping yields, reduced early-round costs, encounter bonuses) to support meaningful strategic depth. This review should happen once per discipline after the first tuning pass.

### 3. Track Cross-Strategy Convergence as a Balance Health Metric

When multiple strategies converge on identical behavior (e.g., Tactician and Durability Tactician both doing ~0.5 rounds/encounter), it signals limited strategic depth. The roadmap could include periodic "strategy differentiation checks" to verify that different strategies exercise different mechanics rather than converging on the same dominant pattern.
