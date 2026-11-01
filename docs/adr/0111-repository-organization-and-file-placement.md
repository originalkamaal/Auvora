# ADR-0111: Repository organization and file placement

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §5.5, §29
- **Related:** ADR-0000, ADR-0023, ADR-0048, ADR-0049, ADR-0107, ADR-0112

## Context

Auvora keeps **everything** (server, frontends, all SDKs, mobile app, integrations, infrastructure, conformance, benchmarks, product docs, specs, ADRs, public docs site) in a single repository for many years. Without strict placement rules, a monorepo of this size degrades into duplicated docs, orphaned files and directories nobody owns. The owner requires the repository, **including documentation**, to stay organized, with every new folder and file placed deliberately.

## Decision

### 1. Top-level layout (closed set)

```
auvora/
├── crates/          Rust server crates (AGPL)              ├── fuzz/          cargo-fuzz targets + corpus
├── ee/              Commercial code (ee/crates, ee/LICENSE) ├── bench/         benchmark suites + reference infra
├── apps/            Deployable frontends                    ├── conformance/   protocol/interop harnesses
├── packages/        JS/TS npm packages                      ├── tests/         cross-crate/system E2E tests only
├── sdks/            Non-JS SDKs, one dir per language       ├── examples/      reference/dogfood apps
├── mobile/          Auvora Authenticator (ios/, android/)   ├── scripts/       repo tooling (check-*, release, codegen)
├── integrations/    Terraform provider, MCP server, plugins ├── docs/          all documentation (see §2)
├── deploy/          helm/, compose/, terraform-modules/, observability/, reference-proxies/
└── root files:      README.md, LICENSE, CONTRIBUTING.md, SECURITY.md, CODE_OF_CONDUCT.md, CLAUDE.md,
                     CODEOWNERS (or .github/), Cargo.toml, Cargo.lock, pnpm-workspace.yaml, package.json,
                     pnpm-lock.yaml, rust-toolchain.toml, justfile, .gitignore, .editorconfig, dotfiles for tools
```

- Adding, renaming or removing a **top-level directory** or **root file** not listed above requires amending this ADR (via a superseding ADR) in the same commit.
- `.temp/` is the only scratch area. It is gitignored, and **tracked files never reference it**.

### 2. Documentation layout

```
docs/
├── README.md        Map of all documentation (must list every subdirectory)
├── product/         plan.md (master plan), archive/ (superseded plans), reviews/ (dated reviews)
├── adr/             Architecture Decision Records (ADR-0000 process) + README.md index
├── specs/           Technical specifications, one subsystem per file or subfolder (e.g. specs/hooks/v1/)
├── threat-models/   One threat model per feature (_template.md + platform.md)
├── security/        Security policies, crypto/PQ migration guides, review reports
├── roadmap/         STATUS.md (phase status), phase exit criteria tracking
├── releases/        Release notes and phase exit reports
├── runbooks/        Operational runbooks (incident, key compromise, DR)
└── site/            Public documentation site (Astro Starlight, ADR-0048)
```

- **One home per type of content.** Decisions → `adr/`; *how something works* → `specs/`; *how to use it* → `site/`; *how to operate it* → `runbooks/`; *what and when* → `product/` and `roadmap/`. Content is never duplicated across homes; link instead.
- Creating a new `docs/` subdirectory requires updating `docs/README.md` and this ADR.
- Superseded documents move to an `archive/` folder next to their replacement; they aren't deleted and aren't edited.

### 3. Naming and placement rules (all files)

1. Directory and file names are `kebab-case`, except where an ecosystem mandates otherwise: Rust modules `snake_case`, Swift/Kotlin types `PascalCase`, conventional files `README.md`, `LICENSE`, `CHANGELOG.md`.
2. Dated documents (reviews, reports, incident write-ups) are prefixed with `YYYY-MM-DD-`.
3. Numbered documents (ADRs) use zero-padded `NNNN-`.
4. **Every directory that holds content has a `README.md`** stating its purpose, ownership and what belongs there. Leaf code folders inside a crate or package are exempt.
5. **Indexes are updated in the same commit** as the content they index: `docs/README.md`, `docs/adr/README.md`, `crates/README.md`, `packages/README.md`, `sdks/README.md`.
6. Tests: Rust unit tests in-module (`#[cfg(test)]`); crate integration tests in `crates/<crate>/tests/`; cross-crate/system tests in root `tests/`; frontend tests colocated (`*.test.ts`); Playwright E2E in `tests/e2e/`.
7. Generated files live in a directory named `generated/` (or are marked with a `@generated` header). They are committed only where ADR-0023 §6 requires it.
8. Fixtures and test data live in `testdata/` next to the tests that use them.
9. No catch-all folders (`misc/`, `utils/` at top level, `stuff/`, `new/`, `old/`). Shared Rust code goes into a named crate; shared TS code into a named package.
10. Secrets, credentials, real customer data and large binaries (> 1 MB) are never committed. Large assets use Git LFS only by ADR.

### 4. Ownership

`CODEOWNERS` covers every top-level directory and every `docs/` subdirectory. Unowned paths fail CI.

## Rationale

A closed top-level set and one home per content type keep a decade-long, everything-in-one monorepo navigable for humans and AI assistants alike. Mechanical rules (naming, READMEs, indexes) can be checked by CI instead of relying on memory.

## Alternatives considered

- **Free-form layout guided by convention only:** drifts within months in a repo this size. Rejected.
- **Docs in a separate repository or wiki:** splits the source of truth, against the owner's single-repo directive. Rejected.
- **Monolithic `docs/` without type separation:** mixes decisions, specs and user guides, which leads to duplication. Rejected.

## Consequences

- Positive: predictable placement; reviewers can reject misplaced files objectively; easy onboarding.
- Negative: small overhead (README + index updates) for every new directory.
- Risks and mitigations: rule drift is mitigated by CI checks and the `CLAUDE.md` contributor rules, which summarize this ADR for every session.

## Fallback / revisit trigger

Revisit if more than 3 PRs in a quarter need an exception to §1 or §2. In that case, write a superseding ADR with the revised layout.

## Implementation notes

- P0: create the top-level skeleton with README.md files, `docs/README.md`, CODEOWNERS and `.editorconfig`.
- `scripts/check-repo-layout` (P0) enforces §1 (closed set), §3.1 (naming), §3.4 (READMEs), §3.5 (index completeness) and §4 (CODEOWNERS coverage).
- `CLAUDE.md` at the repo root summarizes these rules for AI-assisted sessions.

## Verification

CI job `check-repo-layout` blocks merge on violations. PR template checkbox: "Files placed per ADR-0111; indexes updated."
