# ADR-0106: Testing and conformance strategy

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §5.6 (Testing), §24, §26, §27 (standing gates, exit criteria), §29; v0.2 §24
- **Related:** ADR-0047, ADR-0051, ADR-0060, ADR-0105, ADR-0107, ADR-0049, ADR-0017, ADR-0025

## Context

An identity platform is only as trustworthy as its negative-path testing. v0.2 listed test layers but didn't decide tooling, ownership, thresholds or when conformance runs. Auvora's "provable security" pillar needs conformance results published per release.

## Decision

1. **Test layers, tools and gates:**

| Layer | Tooling | Gate |
|---|---|---|
| Unit | `cargo test`, `vitest` | Every PR |
| Property | `proptest` (Rust), `fast-check` (TS) | Every PR; required for parsers, token validation, normalization, RLS |
| Fuzz | `cargo-fuzz` (libFuzzer), corpora in `fuzz/corpus` | PR smoke (60 s/target); nightly 1 h/target; ≥ 24 h per new parser before release (72 h for SAML, SCIM filter) |
| Mutation | `cargo-mutants` on `auvora-oauth`, `-jose`, `-saml`, `-authz`, `-crypto` | Weekly; surviving-mutant budget ≤ 5% |
| Integration (DB) | Testcontainers Postgres 16/17 (+ Valkey, NATS variants) | Every PR |
| RLS isolation | A generated test calls every repository method as project A against project B data | Every PR; 0 leaks |
| API contract | OpenAPI schema validation + `oasdiff` breaking-change check | Every PR |
| E2E browser | Playwright (Chromium, WebKit, Firefox), axe-core | Every PR (smoke); nightly (full) |
| Mobile | Device farm (latest 2 major iOS/Android) | Nightly from P5 |
| Load/perf | k6 + criterion (ADR-0105) | Per ADR-0105 |
| Failure/chaos | toxiproxy, Postgres failover, Valkey loss, clock skew ±5 min, NATS partition | Nightly |
| Upgrade | Install N-1, load data, rolling upgrade to N under traffic; 0 failed requests | Every release candidate |
| Backup/restore | `pg_dump` + PITR restore; keys restorable via KeyProvider | Weekly |
| Security regression | One test per abuse case in plan §26; tagged `abuse::<case>` | Every PR |

2. **Conformance (in CI from P1):**
   - The OpenID Foundation conformance suite runs as a container in `conformance/oidf`. Plans are added per phase (P1: Basic, Config; P2: Logout; P3: Dynamic, Form Post; post-GA: FAPI 2.0, CIBA).
   - Every plan is required to pass 100% (warnings are reviewed and documented).
   - SAML: an interop harness against Keycloak, SimpleSAMLphp and Shibboleth containers, plus a negative suite (XSW variants, comments in NameID, replay, unsigned/partially signed responses, XXE, entity expansion).
   - SCIM: Okta and Entra validator runs (recorded), plus our own RFC 7644 suite.
   - WebAuthn: a FIDO conformance tool run before P2 exit, plus a browser/authenticator matrix.
   - SSF, AuthZEN, LDAP, RADIUS: interop harnesses per ADR-0096 and ADR-0100.
3. **External interop:** the targets in ADR-0060 run nightly where accounts allow. The others run per release candidate.
4. **Negative-test rule:** every protocol feature merges with negative, malformed, replay, expiry and key-rotation tests (v0.2 §24.3). Reviewers block PRs that lack them.
5. **Coverage:** ≥ 90% line coverage on protocol and crypto crates; ≥ 75% on others. The coverage gate is per crate (`cargo-llvm-cov`).
6. **Publication:** each release publishes `docs/conformance/<version>.md` with suite versions, results and known deviations.
7. **Test data:** synthetic only. Production data is never used in tests. Seeded via `auvora dev seed`.

## Rationale

This turns the "no premature feature claims" principle into enforceable CI gates. Running conformance early catches spec drift when it's cheap to fix.

## Alternatives considered

- **Conformance only before certification (v0.2 Phase 10):** defects would be found far too late.
- **One global coverage threshold:** too loose for crypto, too strict for UI glue.

## Consequences

- Positive: high confidence and public evidence.
- Negative: CI time and cost. Mitigations: tiering (PR smoke vs nightly), caching, and self-hosted runners for fuzzing.

## Fallback / revisit trigger

If PR CI median exceeds 20 min, move more suites to merge-queue or nightly tiers. Security-regression and RLS suites never leave the PR tier.

## Implementation notes

P0 creates the harness skeletons (OIDF container, Testcontainers, Playwright, fuzz targets, bench). Owned via CODEOWNERS in `conformance/` and `fuzz/`.

## Verification

CI configuration review per phase. The standing gate in plan §27 checks the published conformance report exists for each release.
