# Vision & Roadmap Suggestions — Durability Generalization

These suggestions are based on the durability generalization work (replacing per-discipline durability tokens in card definitions with a generic `TokenType::Durability`).

## vision.md

- Consider mentioning the "generic card effect → encounter-resolved concrete token" pattern as a design principle. This pattern (used for Durability) could apply to other token types (e.g., a generic `Insight` token resolved per-encounter, or a generic `Material` token). Documenting it as a first-class design concept helps guide future card effect authoring.

## roadmap.md

- **Cross-discipline card sharing**: Now that card effects use `TokenType::Durability` instead of discipline-specific tokens, a natural next step is allowing cards to be shared across gathering disciplines. A single "Rest" card definition with `Durability` cost could work in mining, herbalism, woodcutting, and fishing encounters without duplication.
- **Generic token resolution pattern**: Consider generalizing the `resolve_durability()` approach into a broader `resolve_token(discipline)` mechanism that handles multiple abstract tokens (Durability, Insight, Material) in one pass. This would reduce per-discipline boilerplate in resolve functions.
- **Enemy card effect resolution**: Mining enemy card effects (`LoseTokens`) now use `TokenType::Durability` in their definitions, but the resolution currently happens in `resolve_ore_play()` via the damage vector — not through the `CardEffectKind::LoseTokens` resolution path. If enemy effects are ever resolved through a shared pipeline, that pipeline will need the same discipline-aware token resolution.
