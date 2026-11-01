# ADR-0098: Identity proofing connectors

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §8.1 (Identity proofing), §7.2 (User IAL), §24 (NIST 800-63-4), §27 P5/P10
- **Related:** ADR-0050, ADR-0038, ADR-0067, ADR-0072, ADR-0079, ADR-0110, ADR-0103

## Context

Regulated CIAM (fintech, health, government, marketplaces) needs identity assurance level 2 or higher (NIST IAL2): verified documents, liveness and record checks. Auvora won't build document verification itself. Instead it orchestrates IDV vendors and wallet credentials, and records the resulting assurance on the principal so that policies can require it.

## Decision

1. **Connector framework** in `auvora-authn` (module `proofing`). A connector implements the `ProofingProvider` trait: `start(session) → redirect | sdk_token`, `callback/webhook → ProofingResult`, `fetch_result(id)`.
2. **Built-in connectors**, maintained as declarative manifests plus code where needed:
   - Onfido, Persona, Veriff, Jumio.
   - India: DigiLocker and offline Aadhaar XML/QR, via a licensed partner connector only. Auvora never stores Aadhaar numbers.
   - EUDI Wallet / OID4VP (ADR-0110).
   - A generic **HTTP connector** that uses a sync hook (ADR-0038).
3. **Flow integration:** `proofing` is a step type in the Flow API (ADR-0067), usable in registration, step-up, recovery (ADR-0078) and admin-triggered re-proofing.
4. **Result model:**
   - `ProofingResult {provider, reference_id, ial (1|2|3), checks[], verified_claims (OIDC Identity Assurance format), evidence_summary, expires_at}`.
   - `verified_claims` is emitted in ID tokens/UserInfo only when requested via the `claims` parameter and allowed by policy.
5. **Data minimization:**
   - Auvora stores the result, the verified attributes and the vendor reference. It doesn't store document images or biometrics.
   - Vendor API keys are stored with envelope encryption (ADR-0072).
   - Verified attributes are tagged `pii: sensitive`.
6. **Assurance:** the principal records `ial`, `ial_verified_at` and `ial_expires_at` (default 3 years). Policies use CEL: `principal.ial >= 2`.
7. **Vendor communication** goes through the egress guard (ADR-0103). Webhooks from vendors are verified by signature, and a replay window of 5 min applies.
8. **Config:** `[proofing.providers.<name>]` with credentials reference, `ial_mapping`, `allowed_countries`.

## Rationale

Orchestration gives full coverage of the need without Auvora becoming a KYC vendor. Using the OIDC Identity Assurance claim format avoids a proprietary claim shape.

## Alternatives considered

- **Build document and liveness verification in-house:** requires specialist ML and a compliance burden, and falls outside IAM.
- **Hooks only:** works, but every customer would rebuild the same integrations. The manifests reduce that effort.

## Consequences

- Positive: IAL2 flows are available out of the box, and the EU wallet path is ready.
- Negative: vendor APIs drift. The fixture-based contract tests in CI catch that.
- Risk: storing sensitive verified attributes. Mitigations: tagging, DSAR coverage (ADR-0079) and per-attribute retention.

## Fallback / revisit trigger

A connector whose vendor API breaks for more than 30 days without a maintainer is marked `deprecated` and moved to the generic HTTP connector pattern.

## Implementation notes

- P5: framework + Onfido, Persona + generic HTTP.
- P8: Veriff, Jumio, DigiLocker partner.
- P10: EUDI wallet via ADR-0110.

## Verification

- Recorded-fixture tests per connector.
- Negative tests: forged webhooks, replay, an expired result.
- A DB snapshot test confirms no document or biometric blobs are stored.
