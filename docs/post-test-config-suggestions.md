# Suggestions After Per-Test JSON Configuration Implementation

## Vision.md Suggestions

### Testing as First-Class Design Constraint
The per-test JSON config work revealed that the game engine's testability improves dramatically when configs are composable. Consider adding to the vision:
- **Config-driven testing** as a design principle: every game mechanic should be testable with a minimal JSON config that exercises just that mechanic in isolation.
- **Deterministic replay** guarantee: with fixed seeds + custom configs, any game scenario can be reproduced exactly.

### Research Encounter Symbol Coverage
The research experiment's hidden type selection draws from all 6 ResearchSymbol variants (Alpha–Zeta). Tests must cover all symbols to avoid RNG-dependent failures. This suggests a design note:
- When adding new ResearchSymbol variants, ensure test configs include probes for all variants.

## Roadmap.md Suggestions

### Test Config Library (Near-Term)
- Create a shared test config library that other tools (fuzzing, benchmarks, integration environments) can reuse.
- Document the minimal config format in `tests/configurations/README.md` so contributors know how to write focused test configs.

### Config Validation Endpoint (Medium-Term)
- Add a `/validate-config` endpoint that accepts a discipline JSON and returns validation errors/warnings. This would help both tests and game designers verify configs without running the full game.

### Encounter Win/Loss Parity
Currently, Rest encounters always win (abort = win). Consider whether Rest should have a loss condition (e.g., exhausting rest tokens without recovering enough stamina) to maintain parity with other encounter types.

### Research target_size Flexibility
Research experiments require exactly `target_size` unique card IDs per round. With limited card variety, this creates tight coupling between config and gameplay. Consider:
- Allowing duplicate card IDs in a hand (play same card template multiple times)
- Or making target_size configurable per encounter (already done) but also documenting the implication for card diversity requirements.
