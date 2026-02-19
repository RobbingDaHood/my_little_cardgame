# 0006 - cargo-audit triage

Summary:
A scheduled CI job runs cargo-audit weekly. A local run produced no advisories. Record this result and mark security workflow as configured.

Tasks:
- Verify CI runs cargo-audit and reports findings in PRs.
- If advisories are reported in CI, create PRs to update vulnerable dependencies or add mitigations.
