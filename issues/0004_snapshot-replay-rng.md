# 0004 - Snapshot+Replay test for RNG draws

Summary:
Add an integration test that records a sequence of actions including SetSeed and RngDraw entries, then replays the action log against a fresh GameState to assert deterministic results (token balances and RNG state snapshots).

Tasks:
- Extend the action log with RngDraw and RngSnapshot events where RNG is used.
- Add test that performs a sequence, serializes the action log, rebuilds state using replay_from_log, and asserts equality on relevant state fields.
