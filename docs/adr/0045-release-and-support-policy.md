# ADR-0045: Release and support policy

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D45, §27
- **Related:** ADR-0025, ADR-0064, ADR-0108, ADR-0020, ADR-0046, ADR-0062

## Context

Identity is critical infrastructure. Operators need predictable releases, safe upgrades and fast security fixes.

## Decision

1. **Versioning:** SemVer for the server, SDKs, the Terraform provider and Helm charts independently. Pre-1.0 (`0.x`) minor releases MAY include breaking changes, which are listed in an "Upgrade notes" section. From 1.0, breaking changes only happen in majors.
2. **Cadence:** a minor release monthly (target: first Tuesday). Patch releases as needed. Majors at most yearly.
3. **Security fixes:** critical (CVSS ≥ 9.0) ≤ 7 days from triage, high ≤ 30 days, medium ≤ 90 days. Fixes are backported to all supported lines (ADR-0108).
4. **Support windows:**
   - OSS: the latest minor plus the previous minor (N, N-1) receive security patches.
   - EE: **LTS** release every 6 months, supported 12 months (security + critical bug fixes).
   - Auvora Cloud runs latest at all times.
5. **Upgrades:** zero-downtime rolling upgrade from N-1 → N for minors within a major (ADR-0025). Skipping minors requires sequential upgrades, and `auvora upgrade plan` prints the required path. LTS → next LTS is supported directly.
6. **Compatibility contracts:** API `/v1` additive-only (ADR-0020); token formats, cookie names, JWKS URLs and discovery paths are stable; config keys are deprecated with ≥ 2 minor releases of warnings before removal.
7. **Channels:** `stable`, `rc` (≥ 7 days before stable), `nightly`. Release artifacts are signed (ADR-0046).
8. **Deprecation:** announced in the changelog, the console banner and `/v1` response header `Deprecation` + `Sunset` (RFC 9745/8594).
9. **Changelog:** generated from conventional commits, with a curated "Highlights / Security / Breaking / Upgrade notes" section.

## Rationale

The monthly cadence balances velocity with operator fatigue. N-1 rolling upgrades are essential for HA identity. EE LTS serves enterprises that upgrade slowly.

## Alternatives considered

- **Continuous releases only**: rejected because enterprises need predictable versions.
- **Quarterly releases**: rejected as too slow for security-adjacent improvements during build-out.

## Consequences

- Positive: predictable for operators.
- Negative: backport effort for N-1 and LTS lines.

## Fallback / revisit trigger

If backports consume > 25% of engineering time for 2 consecutive quarters, reduce OSS support to N only (security fixes) via a new ADR.

## Implementation notes

P0: release workflow, release-please or equivalent, signing. P1: `auvora upgrade plan`. LTS starts at GA 1.0 (P5).

## Verification

The upgrade test suite (ADR-0106) runs N-1 → N under load with zero failed requests. A release checklist in CI blocks publication without signatures, SBOM or changelog.
