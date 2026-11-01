# ADR-0013: Own OAuth/OIDC engine in Rust

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D13, §9, §24, §27 (P1, P5)
- **Related:** ADR-0012, ADR-0029, ADR-0030, ADR-0031, ADR-0051, ADR-0052, ADR-0066, ADR-0084, ADR-0085, ADR-0086, ADR-0106

## Context

Auvora must be a complete, certified OAuth 2.x Authorization Server and OpenID Provider covering every grant and extension in plan §9 (PAR, JAR, JARM, RAR, DPoP, mTLS, CIBA, token exchange, UMA, DCR, CIMD, RFC 9728, FAPI 2.0, logout specs). The Rust ecosystem has no mature, certified, feature-complete OP library. Existing crates are client-side (`openidconnect`, `oauth2`) or minimal servers. v0.2 left the library choice open (ADR 4 there).

## Decision

1. Auvora implements its own AS/OP in the crate **`auvora-oauth`**, built on `auvora-jose` (ADR-0012) and `auvora-core`.
2. **Architecture:**
   - **Protocol layer** (pure, no I/O): request parsing, validation, and the decision state machine per endpoint. Inputs are typed requests plus a config snapshot (ADR-0009). Outputs are typed responses or errors. 100% deterministic and fuzzable.
   - **Service layer:** storage (codes, grants, tokens), key access (ADR-0011), session/flow integration (ADR-0067), and events.
   - **HTTP layer:** axum handlers in `auvora-api`, a thin mapping layer.
3. **Spec-driven development:**
   - Every MUST/SHOULD in implemented specs is tracked in `crates/auvora-oauth/spec-matrix.toml`, with a test ID per requirement.
   - Unimplemented SHOULDs need a documented justification.
   - CI fails if a requirement row lacks a test.
4. **Validation strictness defaults:**
   - exact redirect matching (ADR-0085);
   - PKCE S256 required for all clients;
   - `iss` in authorization responses (RFC 9207);
   - `alg` pinned per key;
   - `typ` checks;
   - max request object size 64 KiB;
   - clock skew ±60 s (configurable to ±300 s max);
   - authorization codes single-use, TTL 60 s;
   - PAR `request_uri` TTL 60 s.
5. **Excluded grants:** Implicit and ROPC are never implemented (ADR-0066).
6. **Conformance:** the OpenID Foundation conformance suite runs in CI from P1, with profiles expanding per phase (ADR-0051, ADR-0106). Target certifications are listed in ADR-0051.
7. **Feature placement:** P1 (core), P2 (device flow, form_post, front/back-channel logout, consent), P5 (DPoP, mTLS, PAR, JAR, JARM, RAR, token exchange, assertion grants, CIBA, DCR/DCRM, CIMD, RFC 9728, client policy profiles, encrypted ID tokens, pairwise), P6 (UMA 2.0), P10 (FAPI 2.0 certification, OID4VCI pre-authorized code).

## Rationale

Owning the engine is the only way to reach full feature coverage (U1) and high performance in Rust. A pure protocol layer makes fuzzing, property testing and formal requirement tracking practical. Certification provides independent evidence.

## Alternatives considered

- **Wrap an existing OP (e.g. Ory Hydra) as a sidecar:** a second runtime (Go), split data model, limited control over extensions. Rejected.
- **Use `oxide-auth`:** limited OIDC/extension coverage and maintenance concerns. Rejected.
- **Fork a Rust OP crate:** none is mature enough to justify it. Rejected.

## Consequences

- Positive: full control, performance, coherent model with flows, agents and support access.
- Negative: large implementation and maintenance effort; Auvora carries the security responsibility.
- Risks: subtle protocol bugs. Mitigation: pure protocol layer + fuzzing + property tests + OIDF suite + external protocol review before GA (ADR-0106).

## Fallback / revisit trigger

None. This is a core competency.

## Implementation notes

- Module layout: `auvora-oauth::{authorize, token, par, introspect, revoke, userinfo, discovery, jwks, logout, device, ciba, exchange, dcr, uma, profiles}`.
- A client library, `auvora-oauth-client` (used for federation, ADR-0087), shares the JOSE and validation code.

## Verification

- OIDF conformance suite (CI).
- Spec-matrix completeness check.
- cargo-fuzz targets for every parser (authorization request, token request, request object, DPoP proof, client assertion).
- Mutation testing with cargo-mutants, required score ≥ 80% (ADR-0106).
