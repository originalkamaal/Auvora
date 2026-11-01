# ADR-0051: OIDF certification targets

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D51, §9, §27 (P1–P5, P10), §1.1 U5
- **Related:** ADR-0013, ADR-0106, ADR-0060, ADR-0084, ADR-0064

## Context

Pillar U5 ("provable security") and ADR-0013 (own OAuth/OIDC engine) require independent evidence of protocol correctness. The OpenID Foundation conformance suite and certification program provide it.

## Decision

1. **Conformance suite in CI from P1.** The OIDF conformance suite (self-hosted container in `conformance/oidf/`) runs against `auvora dev`:
   - per PR touching `auvora-oauth`, `auvora-jose`, `auvora-sessions`: the relevant test plans (≈10 min budget);
   - nightly: the full matrix of all targeted plans and variants (client auth methods, response modes, signing algorithms).
   - Any failure or warning regression blocks merge (warnings must be triaged and allowlisted with justification in `conformance/oidf/allowlist.toml`).
2. **Certification targets and phase:**

| Profile | Passing in CI by | Certified by |
|---|---|---|
| OP Basic | P1 | GA 1.0 (P5) |
| OP Config | P1 | GA 1.0 |
| OP Dynamic | P3 | GA 1.0 |
| OP Form Post | P2 | GA 1.0 |
| RP-Initiated Logout OP | P1 | GA 1.0 |
| Front-Channel Logout OP | P2 | GA 1.0 |
| Back-Channel Logout OP | P2 | GA 1.0 |
| Session Management OP | P2 | GA 1.0 |
| FAPI 2.0 Security Profile (DPoP + mTLS variants, private_key_jwt + mTLS client auth) | P5 | P10 (post-GA, first priority) |
| FAPI 2.0 Message Signing | P5 | P10 |
| FAPI-CIBA | P5 | P10 |
| OIDC RP certification for `auvora-federation` (Basic, Config) | P4 | P7 |
| Shared Signals (SSF transmitter/receiver) interop | P5 | when OIDF certification is available |
| OpenID4VCI/VP | P10 | P10 |

   **Implicit and Hybrid OP profiles are deliberately not targeted** (ADR-0066). The suite is configured to skip the `id_token`/`token` response types, and docs explain why.
3. **Re-certification:** certifications are renewed on every major release, and whenever the OIDF suite releases a breaking version, within 90 days.
4. **Publication:** results (suite version, plan, pass/warn list, log links) are published per release in docs under "Conformance" (ADR-0106). The certification marks are shown on the website.
5. **Budget:** OIDF membership + certification fees are budgeted before GA. The project applies for OIDF membership in P3.

## Rationale

Continuous conformance prevents regressions. Certification is a credibility requirement for enterprise and fintech buyers.

## Alternatives considered

- **Certify only at GA**: rejected because late discovery is costly.
- **Certify Implicit/Hybrid for completeness**: rejected because it conflicts with RFC 9700 and ADR-0066.

## Consequences

- Positive: externally verifiable correctness; marketing proof for U5.
- Negative: CI time; fees.

## Fallback / revisit trigger

If the OIDF suite can't run in CI (licensing/infrastructure change), run it on a scheduled self-hosted runner instead. Certification targets remain.

## Implementation notes

P0 harness container + config generator; P1 plans wired; `conformance/oidf/plans/*.json` generated from the client policy profiles (ADR-0084).

## Verification

The CI job `conformance-oidf` is required; the release checklist requires a current conformance report attached.
