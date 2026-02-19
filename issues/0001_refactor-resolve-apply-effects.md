# 0001 - Refactor resolve::apply_effects (follow-up)

Summary:
The apply_effects function was refactored to snapshot operations and avoid holding the combat lock across await points. Follow-up work: code review, targeted unit tests for race conditions, and documentation of the lock acquisition ordering.

Tasks:
- Review the refactor in src/combat/resolve.rs and verify behavior under concurrent plays.
- Add unit/integration tests that simulate concurrent player and enemy plays to validate no regressions.
- Document lock ordering and rationale in AUDIT_FINDINGS.md or module-level comments.
