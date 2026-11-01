# Contributing to Auvora

Thank you for your interest. This guide summarizes the rules contributors must follow. The authoritative versions are the linked ADRs.

## Before you start

- Read the plan: [`docs/product/plan.md`](docs/product/plan.md).
- Read the decisions in [`docs/adr/`](docs/adr/README.md). Changes must not contradict an Accepted ADR. To change a decision, propose a superseding ADR ([ADR-0000](docs/adr/0000-record-architecture-decisions.md)).
- A Contributor License Agreement is required before your first contribution can be merged ([ADR-0004](docs/adr/0004-licensing-agpl-apache-commercial-cla.md)). The CLA process will be published before external contributions open.

## Development setup

Tool versions are pinned in `.mise.toml` and `rust-toolchain.toml`.

```sh
mise install        # installs the pinned Rust, Node, pnpm and just
just setup          # installs git hooks (commit-msg policy check)
just check          # everything CI runs: fmt, lint, tests, repository checks
```

## Where files go ([ADR-0111](docs/adr/0111-repository-organization-and-file-placement.md))

- The top-level directories form a closed set. Each kind of documentation has one home: decisions in `docs/adr/`, specs in `docs/specs/`, user docs in `docs/site/`, runbooks in `docs/runbooks/`.
- Names are `kebab-case`. Every content directory has a `README.md`. Update indexes in the same commit.

## Commits ([ADR-0112](docs/adr/0112-commit-policy.md))

- Make one logical change per commit. Every commit must build and pass checks; `main` is never broken.
- Use the Conventional Commits format: `type(scope): imperative summary` (at most 72 characters), plus a body explaining *why* and referencing ADRs.
- No attribution trailers (co-author lines, "Generated with …", tool signatures). The `commit-msg` hook and CI enforce this.

## Code

- Rust standards: [ADR-0047](docs/adr/0047-rust-coding-standards.md). No `unsafe` outside allowlisted FFI crates; no panics on request paths; pedantic clippy.
- Security-sensitive changes need a threat-model note ([ADR-0049](docs/adr/0049-threat-modelling-method.md)) and review by a security code owner.
- Definition of Done: [ADR-0107](docs/adr/0107-engineering-process-and-definition-of-done.md).
