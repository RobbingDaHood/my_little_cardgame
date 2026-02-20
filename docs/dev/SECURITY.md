# Security & Audit

This repository includes a GitHub Actions workflow to run `cargo audit` weekly (see .github/workflows/security.yml).

To run a local security scan:

```bash
# install cargo-audit if not installed
cargo install --locked cargo-audit
# run the audit and review advisories
cargo audit
```

If advisories are reported, create issues to upgrade or patch vulnerable crates. The CI security workflow is configured to run `cargo audit` and should be used as the authoritative source for gating PRs.
