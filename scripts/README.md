# scripts/: Repository tooling

Check scripts run locally and in CI (`check-adrs`, `check-repo-layout`, `check-commits`), git hooks, and release/codegen helpers.

- **License:** AGPL-3.0
- **Governing decisions:** [ADR-0000](../docs/adr/0000-record-architecture-decisions.md), [ADR-0111](../docs/adr/0111-repository-organization-and-file-placement.md), [ADR-0112](../docs/adr/0112-commit-policy.md)
- **Placement rules:** [ADR-0111](../docs/adr/0111-repository-organization-and-file-placement.md)

## Contents

| Path | Purpose |
|---|---|
| [`check-adrs`](check-adrs) | ADR headings, status, index completeness, reference resolution (ADR-0000) |
| [`check-repo-layout`](check-repo-layout) | Closed top-level set, READMEs, kebab-case docs, docs index, no `.temp/` paths (ADR-0111) |
| [`check-commit-message`](check-commit-message) | Validates one commit message (ADR-0112) |
| [`check-commits`](check-commits) | Validates every commit message in a range; used by CI (ADR-0112) |
| [`git-hooks/`](git-hooks/) | Git hooks installed by `just setup` (`commit-msg`) |
