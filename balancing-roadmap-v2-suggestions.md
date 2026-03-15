# Vision & Roadmap Improvement Suggestions

Observations from updating the Balancing Track (B1–B8) to use worktree-isolated iterative simulation.

## vision.md suggestions

1. **Update the "Tuning pipeline and instrumentation" section** (lines 370-378) to mention the worktree-isolation pattern: each discipline gets its own git worktree/branch for safe config modification and parallel runner execution. The current text mentions "multiple server instances on different ports" but doesn't describe the worktree-based workflow that enables isolated config experimentation.

2. **Add "worktree-based experimentation" to the balancing section**: The layered balancing approach could mention that config mutations are tested in isolated git worktrees, committed for audit trail, and compared against baselines — reinforcing the deterministic/reproducible design principle.

3. **Expand "Operational controls & feedback"** (lines 376-378): The current text mentions "A/B and sandbox playtests" generically. The worktree-per-mutation pattern is effectively A/B testing — each mutation is an isolated experiment with committed results. This could be described more concretely.

## roadmap.md suggestions

1. **Consider adding a "Balancing prerequisites" note**: B2 (general config bypass) depends on Step 14 (Configuration externalization) being complete. The roadmap could make this dependency explicit with a cross-reference.

2. **Add a "Strategy bots" sub-step to B3**: The runner description mentions strategies (random, greedy, conservative) but could benefit from a more detailed strategy definition section — perhaps as a sub-step B3.1 — defining what each strategy means per discipline. This was detailed in the old B5 and is still valuable context.

3. **Consider adding a "Continuous balance regression" future step**: The old B6 had a `make balance-check` target concept that runs quick simulations and asserts win rates stay within documented targets. This was removed in the rewrite but remains a valuable idea for preventing balance regressions. It could be added as B9 (future) or as a note under B6.

4. **Document the worktree naming convention**: The roadmap mentions branch names like `balance/mining-runner` and `balance/combat-mutation-1` but could benefit from a documented naming convention section to keep branches organized as the number of experiments grows.

5. **Cross-reference with `scripts/worktree-manage.sh`**: The existing worktree management script could be extended or referenced in B3 for automating worktree creation/cleanup during balancing runs.
