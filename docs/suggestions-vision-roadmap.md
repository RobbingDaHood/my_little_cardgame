# Suggestions for vision.md and roadmap.md

Based on discoveries during the B2.1 combat rebalance (35 iterations, shield lifecycle fix, dodge mechanic, target adjustment).

## vision.md suggestions

### 1. Document the death spiral mechanic as a design decision
After player death, scouting generates from the KILLING encounter's difficulty level (not base). This creates a compounding death spiral where each subsequent encounter is harder. This is the primary factor driving win streaks down and is currently undocumented. Vision should state whether this is intentional (punishment for death) or a bug to fix (reset difficulty on death).

### 2. Add scouting mutation asymmetry as a known design constraint
Enemy HP scales 100% with difficulty factor, but enemy card effects only scale probabilistically (~10% per step with current config: mutation_fraction × scale_probability = 0.20 × 0.50). This creates "HP sponge" encounters at high difficulty — long fights with moderate danger. Vision should document whether proportional scaling (matching HP and damage growth) is the intended design, or whether the current asymmetric scaling is deliberate to create a "war of attrition" feel.

### 3. Document dodge vs shield design intent
The B2.1 rebalance established dodge as the "skill card" (high absorption, FixedTypeDuration, 1 round, rewards timing) and shield as the "safety card" (low absorption, PersistentCounter, consumed on damage — persists across encounters when not fully consumed). This design intent should be explicitly stated to guide future card designers — new defence mechanics should fit into this spectrum.

### 4. Consider adding health regeneration mechanics
Currently health only decreases (except on death reset). Cost_damage cards drain HP over time with no recovery. This makes all strategies eventually fatal. A health regeneration mechanic (e.g., rest encounters or resource cards that heal) could extend game sessions and create more strategic depth around health management.

## roadmap.md suggestions

### 1. Add a B2.2 step for scouting difficulty reset on death
The death spiral (difficulty compounds after death) is the single largest factor affecting balance. A dedicated step to implement and tune death-difficulty interaction would be high-impact. Options: (a) reset to base difficulty, (b) reduce difficulty by N%, (c) keep current behavior. Each has different balance implications.

### 2. Add a B2.3 step for proportional mutation scaling
The current mutation system creates HP sponges because only 10% of enemy cards scale per step. A dedicated step to implement proportional scaling (all enemy effects scale with difficulty factor) would dramatically improve the feel of high-difficulty encounters. This could be a game_rules.json config change (mutation_fraction=1.0, scale_probability=1.0) or a code change to scale effects alongside initial_tokens.

### 3. Document the 35-iteration tuning methodology
The B2.1 rebalance required 35 iterations across multiple mechanical discoveries (GainTokens duration bug, FixedTypeDuration expiration not implemented, shield accumulation, encounter hand size, scouting infinite loops). Future discipline tuning should expect similar iteration counts and be budgeted accordingly. Consider adding an "expected iteration count" field to each balancing step.

### 4. Add strategy differentiation testing to B3+
The current balance test only measures win rate and streak. Strategy ORDERING (Tactician > Random > Greedy > Conservative for streaks) is a critical invariant that should be explicitly tested. Add ordering assertions to the balance test framework so future config changes can't accidentally make Random outperform Tactician.

### 5. Consider larger simulation sample sizes
Current: 10 games × 50 encounters = 500 per strategy. With high variance, results fluctuate significantly between seeds. Consider increasing to 50+ games for final validation runs (configurable via feature flag or env var) to reduce variance in pass/fail decisions.
