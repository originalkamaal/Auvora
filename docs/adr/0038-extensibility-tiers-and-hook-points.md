# ADR-0038: Extensibility tiers and hook points

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D38, §14, principle 8
- **Related:** ADR-0016, ADR-0067, ADR-0080, ADR-0081, ADR-0103, ADR-0058, ADR-0098

## Context

Extensibility is why Auth0 (Actions), Keycloak (SPIs), Zitadel (Actions/targets) and Ory (webhooks) win deals. Auvora must offer the union of these, from no-code to deep customization, without letting extensions weaken core security (principle 8).

## Decision

1. **Five tiers:**

   | Tier | Mechanism | Execution | Phase |
   |---|---|---|---|
   | 1 | Webhooks | Async, after commit, via outbox (ADR-0080) | P3 |
   | 2 | Sync HTTP hooks | In-flow HTTP call to a customer endpoint | P3 |
   | 3 | CEL expressions | In-process, bounded (ADR-0016) | P2 |
   | 4 | Wasm actions | In-process wasmtime sandbox (ADR-0081) | P8 |
   | 5 | Rust modules | Compile-time trait impls in custom builds | P3 |

2. **Hook points** (sync, tiers 2/3/4):
   - `pre-user-create`, `post-user-create`
   - `pre-login`, `post-authentication`
   - `pre-token-issue`, `pre-userinfo`, `pre-saml-assertion`
   - `password-verify` (lazy migration)
   - `mfa-required` (decision)
   - `pre-org-join`, `post-org-join`
   - `scim-pre-apply`
   - `pre-support-session`, `pre-agent-delegation`
   - `credential-exchange` (workload/token exchange claim checks)
   - `identity-proofing-result` (ADR-0098)

   `post-*` sync hooks may only annotate (metadata, claims) or trigger async work.
3. **Hook contract:**
   - Request: a JSON envelope `{hook, version, project_id, context, subject, client, request: {ip, ua, ...}}` signed with HMAC-SHA-256 (`Auvora-Signature` header, timestamped, 5 min tolerance), optional JWS.
   - Response: `{decision: "continue" | "deny" | "require", patch: {...}, require: {aal?, factors?, challenge?}, message?}`. Unknown fields are rejected.
   - **Allowed effects:** add or modify mapped claims within the token budget; set metadata; deny; require stronger auth; supply a migrated password verdict (`password-verify` only).
   - **Forbidden effects** (enforced by the engine regardless of response): skipping credential verification, lowering the AAL, changing `sub`/`iss`/`aud`/`act`, issuing tokens, modifying signing, suppressing audit, granting scopes not allowed to the client.
4. **Execution limits:**
   - Sync HTTP: timeout `hooks.timeout_ms` (default 2,000, max 10,000), with 1 retry on connect error only.
   - Circuit breaker per hook (opens after 5 consecutive failures within 60 s).
   - **Fail mode per hook:** `fail_open` or `fail_closed`. The default is `fail_closed` for hooks that can deny (`pre-*`, `mfa-required`) and `fail_open` for enrichment (`pre-token-issue`). It is shown explicitly in the console.
   - All outbound calls go through the egress guard (ADR-0103).
5. **Ordering:** multiple handlers per hook point run in configured order. A `deny` short-circuits. Patches merge in order, last write wins, and conflicts are audited.
6. **Rust module trait (tier 5):** `auvora_hooks::Module` with registration for authenticators, connectors (directories, IdV, SMS), claim providers and storage adapters. It's available only in custom builds (`--features custom-modules`), not loaded dynamically. Modules run in-process and are trusted code, documented as such.
7. **Observability:** every hook invocation records duration, outcome and an error class (with no payload bodies unless `hooks.debug_capture = true` for dev projects only). The console has a hook test console with sample payloads.
8. **Versioning:** hook payload schemas are versioned (`v1`). Old versions are supported for 12 months after a new version.

## Rationale

Offering all five tiers meets the needs behind Auth0 Actions, Keycloak SPIs and Ory webhooks. The engine-enforced effect allowlist keeps the invariant that extensions can only add data, deny, or require more.

## Alternatives considered

- **Arbitrary JS (Auth0-style Node runtime)**: needs a JS runtime, a weaker sandbox, and Node in the minimum profile. Rejected; Wasm covers the need.
- **Dynamic plugin loading (.so)**: unsafe ABI, supply-chain risk. Rejected.
- **Webhooks only**: can't influence flows. Rejected.

## Consequences

- Positive: full customization spectrum; security invariants preserved.
- Negative: sync hooks add latency to flows. This is surfaced in analytics and bounded by timeouts.
- Risks and mitigations: a slow customer endpoint degrading logins is handled by the circuit breaker plus fail mode.

## Fallback / revisit trigger

None for the tiers. New hook points are added via minor releases without an ADR, as long as they follow the contract above.

## Implementation notes

Crate `auvora-hooks`; JSON Schemas in `docs/specs/hooks/v1/`. Config per project: `hooks[]` with `point`, `type`, `url|expression|module`, `fail_mode`, `timeout_ms`, `order`.

## Verification

Engine tests proving forbidden effects are ignored or rejected; timeout and circuit-breaker tests; signature verification tests; fuzzing the hook response parser.
