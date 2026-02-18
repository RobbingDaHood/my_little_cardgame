# Audit Findings — Phase 2 (Manual Code Audit)

Date: 2026-02-18

Summary

This document records findings from a focused manual audit of the codebase covering combat, deck, action handling, the in-memory library, and player-state/RNG. Automated fixes already applied: removed unsafe unwraps in production code and added property/concurrency tests.

Scope reviewed
- src/combat/
- src/deck/
- src/action/
- src/library/
- src/player_data.rs, src/player_seed.rs, src/player_tokens.rs
- tests/

High-level findings & recommendations

1) src/combat (severity: medium)
- Findings: combat resolution and enemy_play logic are correct functionally and covered by tests. However resolve::apply_effects holds the `current_combat` async mutex while awaiting other async locks (e.g., `player_data.tokens.lock().await`). Holding multiple async locks across awaits increases risk of deadlock if lock ordering changes.
- Recommendation: shorten lock scope: extract needed mutable references/indices and drop the `current_combat` lock before awaiting other locks; or define a strict lock acquisition ordering and document it.
- Action: documented here; refactor task created in TODOs.

2) src/action (severity: low)
- Findings: PlayCard flow is correct but had a misleading error message referencing the whole action enum instead of the card id (fixed in commit).
- Recommendation: keep error messages precise and add a unit test asserting expected error payloads for invalid plays.
- Action: message fix committed.

3) src/deck (severity: low)
- Findings: Deck and card state transitions appear consistent and are well-covered by tests. Edge-cases around saturating subtraction and zero counts are handled.
- Recommendation: add explicit unit tests for boundary cases (e.g., draw when deck empty, change state when missing from source state).

4) src/library & ActionLog (severity: low/medium)
- Findings: ActionLog is append-only using a std::sync::Mutex and AtomicU64 for seq — concurrency-safe for typical workloads. The library uses a mix of async (rocket futures) and sync mutexes; blocking std::sync::Mutex inside async contexts can block executor threads if heavily contended.
- Recommendation: consider using async-friendly primitives (tokio::sync::Mutex) or parking_lot::Mutex for lower blocking cost; or keep current approach but avoid long-held std locks inside async endpoints. Add stress tests (already added) and monitor contention.

5) player_data, RNG & determinism (severity: low)
- Findings: RNG seeding, snapshot and replay mechanics are present. Tests were added to assert replay determinism and property-based invariants; results pass locally.
- Recommendation: add snapshot/restore unit tests that exercise seed+action-log replay across multiple actions including RngDraw/SetSeed entries.

Other notes
- Many tests use expect/unwrap in test code — this is acceptable in tests but production code must avoid panics (already enforced).
- Coverage: the CI job enforces an 85% lines threshold; local coverage run produced a report but failed the threshold — recommend prioritizing adding unit tests for low-coverage modules or adjusting threshold via team agreement.

Prioritized actionable tasks (short)
1. High: Refactor resolve::apply_effects to avoid holding current_combat across await points (create small PR). (owner: dev, estimate: small)
2. Medium: Replace std::sync::Mutex in ActionLog with an async-aware or lower-latency primitive or document lock usage and avoid long holds. (owner: dev)
3. Medium: Add unit tests for error message payloads and deck edge-cases. (owner: dev)
4. Low: Add snapshot+replay integration test that includes RngDraw entries. (owner: dev)
5. Medium: Run cargo-audit in CI (security workflow added) and triage advisories if present. (owner: dev/infra)

Files changed as part of audit
- Replaced unwraps in production code (committed).
- Added randomized replay test, proptest, and concurrency stress test (committed).
- Fixed PlayCard error message (committed in this step).

Where to find next steps
- Triage tasks are listed here; create GitHub issues for each prioritized task and link to this file (AUDIT_FINDINGS.md) for context.

