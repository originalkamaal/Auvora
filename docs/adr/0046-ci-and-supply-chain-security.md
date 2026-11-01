# ADR-0046: CI and supply-chain security

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D46, §26, §29
- **Related:** ADR-0047, ADR-0045, ADR-0106, ADR-0107, ADR-0108, ADR-0023

## Context

An IAM server is a high-value supply-chain target: a compromised dependency or build pipeline compromises every downstream login. The plan requires signed artifacts, SBOMs, provenance and reproducible builds.

## Decision

1. **Platform:** GitHub repository + GitHub Actions. Actions are pinned by **full commit SHA**, updated by Renovate. Workflow `permissions:` default to `contents: read`, with per-job elevation. OIDC is used for cloud credentials (no static secrets in CI).
2. **Required checks on every PR:** `cargo fmt`, `clippy` (ADR-0047), `cargo test` (unit + integration via Testcontainers), `cargo deny` (licenses, bans, advisories, sources), `cargo audit`, `cargo vet`, `pnpm lint/typecheck/test`, `pnpm audit --prod`, OpenAPI breaking-change check (`oasdiff`), ADR lint (ADR-0000), secret scanning (gitleaks), CodeQL (JS/TS) and Semgrep rules for Rust.
3. **Dependency policy:**
   - New Rust crates need a `cargo vet` audit or an import from trusted audit sets (Mozilla, Google, Bytecode Alliance, ZcashFoundation).
   - Allowed licenses: Apache-2.0, MIT, BSD-2/3, ISC, Zlib, Unicode-3.0, MPL-2.0 (file-level), CC0. GPL/AGPL dependencies are disallowed in SDKs; allowed in the server only after owner approval.
   - Git dependencies are disallowed in releases.
   - npm uses `pnpm` with lockfile, `minimumReleaseAge` ≥ 3 days for updates, no install scripts except an allowlist.
4. **Build integrity:**
   - Release builds run on GitHub-hosted runners in a dedicated `release` environment with required reviewers.
   - **SLSA Build L3** provenance via `slsa-github-generator`.
   - Artifacts (binaries, OCI images, Helm charts, npm packages) are signed with **Sigstore cosign** (keyless) and npm provenance.
   - **CycloneDX SBOM** for every binary, image and package, attached to the release and the image as an attestation.
   - **Reproducible builds:** `--locked`, pinned toolchain via `rust-toolchain.toml`, `SOURCE_DATE_EPOCH`, path remapping. A weekly job rebuilds the last release and diffs hashes (Tier 1 target: bit-for-bit for the static musl binary).
5. **Repository protection:** branch protection on `main`, signed commits for maintainers, CODEOWNERS on `crates/auvora-{crypto,jose,oauth,saml,authz,sessions,authn}` and `.github/` with 2 reviewers required for the `security` label, no force-push, linear history.
6. **Container images:** distroless/static base, non-root UID 65532, read-only root FS compatible, scanned with Trivy/Grype in CI (block on fixable critical).
7. **Fuzzing in CI:** cargo-fuzz targets run 10 min per PR touching parsers, and 24 h nightly per target via ClusterFuzzLite/OSS-Fuzz application (ADR-0106).

## Rationale

These controls cover dependency, build and distribution threats end-to-end with free tooling compatible with the open-source model.

## Alternatives considered

- **Self-hosted GitLab/Buildkite**: rejected because of extra operations for no security gain at this stage.
- **GPG-signed artifacts**: rejected because key management is worse than Sigstore keyless with transparency logs.

## Consequences

- Positive: verifiable artifacts; strong posture for enterprise review and SOC 2.
- Negative: slower dependency onboarding (vet audits); CI minutes.

## Fallback / revisit trigger

If GitHub Actions is compromised or unavailable for > 72 h, or SLSA L3 becomes unattainable on hosted runners, move release builds to an isolated self-hosted builder (new ADR).

## Implementation notes

P0 deliverable (all of it before any feature code). Files: `.github/workflows/{ci,release,nightly-fuzz,repro}.yml`, `deny.toml`, `supply-chain/` (cargo-vet), `renovate.json`.

## Verification

Required status checks in branch protection; `cosign verify` and `slsa-verifier` steps in the post-release job; weekly reproducibility job report.
