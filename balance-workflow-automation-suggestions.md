# Vision & Roadmap Suggestions — Balance Workflow Automation

Based on learnings from the herbalism balance tuning session (PR #72) and the subsequent meta-improvement analysis.

## Vision Suggestions

### Developer Experience as a First-Class Concern
The balance tuning workflow revealed that **tooling quality directly determines tuning velocity**. Sessions that should take 30 minutes can stretch to 3+ hours when the developer (human or AI) must manually discover API JSON formats, debug field path mismatches, and iterate without fast feedback loops. Consider adding a vision statement about developer experience parity — tools for authoring and tuning game content should be as polished as the runtime itself.

### Self-Documenting APIs for AI Collaboration
The API format inspector (`tests/balance/api_inspect.rs`) was created specifically because AI agents lose context about JSON field nesting between sessions. This pattern — executable documentation that dumps real formats — could be generalized. Vision: every API surface should have a companion "format dump" test that serves as ground truth for both human developers and AI agents.

## Roadmap Suggestions

### Short Term
1. **Expand API inspectors to all disciplines** — Currently only herbalism has an `api_inspect_*` test. Adding mining, woodcutting, fishing, and combat inspectors would prevent the same 40% time-sink pattern in future tuning sessions.
2. **Add inspector for scouting flow** — The scouting → encounter-pick → encounter-start flow has its own JSON format nuances. An inspector that walks through this multi-step flow would prevent bugs in encounter selection logic.

### Medium Term
3. **Parameterized balance simulation configs** — Currently simulation sample sizes (games × encounters) are hard-coded. Support runtime overrides (e.g., `BALANCE_GAMES=3 BALANCE_ENCOUNTERS=20 scripts/balance-sim.sh herbalism`) to enable fast iteration without code changes.
4. **Balance regression dashboard** — Store simulation results from each run in a structured format (JSON file or CSV). Over time this creates a history that shows how config changes affected each strategy's performance.
5. **Copilot skill for parallel balance tuning** — The `parallel-balance-tuning` skill exists but could integrate with the new scripts. Define a workflow where multiple worktrees tune different disciplines simultaneously, with a final cross-discipline validation step.

### Long Term
6. **Auto-discovery of API formats** — Instead of hand-written inspector tests per discipline, generate format dumps from the OpenAPI schema. This would automatically stay in sync as the API evolves.
7. **Balance CI gate** — Run `make balance-check` in CI (perhaps on a schedule rather than every push, given the ~7 min runtime). Alert when any strategy drifts outside its tier targets.
