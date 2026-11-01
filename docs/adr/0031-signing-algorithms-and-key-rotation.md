# ADR-0031: Signing algorithms and key rotation

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D31, §9.4, §23
- **Related:** ADR-0011, ADR-0012, ADR-0029, ADR-0082, ADR-0083, ADR-0096

## Context

Token signatures anchor trust for every relying party. Algorithm confusion, stale JWKS caches during rotation, and shared keys across tenants are classic failure modes. Interoperability needs RS256, while modern defaults prefer ECDSA or EdDSA.

## Decision

1. **Supported JWS algorithms:**
   - ES256 (**default**), ES384, ES512 (verify only),
   - RS256, RS384, RS512, PS256, PS384, PS512,
   - EdDSA (Ed25519).
   - `HS*` only for client_secret_jwt client authentication (verification of client assertions), **never** for Auvora-issued tokens.
   - `none` is never accepted or issued.
2. **Per-project keys.** Each project has its own key set; keys are never shared across projects. Per-usage keys follow ADR-0082 (token signing, SAML signing, etc.).
3. **Per-client algorithm selection:** `id_token_signed_response_alg`, `authorization_signed_response_alg` etc. are honored if the algorithm is in the project allowlist (`signing.allowed_algs`, default `[ES256, RS256, PS256, EdDSA]`). The project must hold an active key for each enabled algorithm. RS256 keys are 3072-bit by default (2048 accepted for imports).
4. **`alg` pinning:** each key's JWK carries `alg` and `use`. Verifiers in Auvora and its SDKs select the key by `kid` and require the token `alg` to equal the key's `alg`.
5. **Automatic rotation:**
   - Every **90 days** (`signing.rotation_days`, bounds 7–365).
   - The new key is published in JWKS as `pending` at least **48 h** before activation (`signing.prepublish_hours`), so relying-party caches pick it up.
   - The previous key stays published as `retiring` (verify-only) for max token lifetime + 48 h (minimum 7 days).
   - At most 2 signing keys per algorithm are verifiable at a time, plus pending.
6. **Emergency rotation:** `auvora keys rotate --emergency --project <id>` (and the console/API equivalent, requiring AAL3 + approval if configured).
   - Immediately activates a new key and removes the compromised key from JWKS.
   - Revokes refresh tokens and sessions per policy (`keys.emergency_revoke = "sessions" | "tokens_only" | "none"`).
   - Emits an SSF/CAEP event and an audit `key.compromised`.
7. **JWKS endpoint:**
   - `Cache-Control: public, max-age=300, stale-while-revalidate=60`. ETag supported.
   - SDKs refetch on unknown `kid`, rate-limited to once per 30 s.
8. **ML-DSA** support is added behind a flag once IANA JOSE registrations are final (ADR-0083).

## Rationale

ES256 has small, fast signatures and broad support. RS256 remains for legacy relying parties. Pre-publication and overlap remove rotation race conditions. Per-project keys bound the blast radius of any compromise.

## Alternatives considered

- **RS256 default**: larger tokens, slower signing. Rejected as default.
- **Global installation key**: cross-tenant blast radius. Rejected.
- **Manual rotation only**: keys get stale. Rejected.

## Consequences

- Positive: safe rotation; tenant-isolated blast radius; algorithm agility.
- Negative: many keys at large scale (100k projects × algorithms). Signing keys are generated lazily per enabled algorithm, and KMS cost is handled via wrapped local keys (ADR-0011).
- Risks and mitigations: relying parties with broken JWKS caching are covered by documented `prepublish_hours` and a console "JWKS consumer check."

## Fallback / revisit trigger

Revisit when ML-DSA JOSE registrations finalize (ADR-0083), or if any supported algorithm is deprecated by NIST/IETF.

## Implementation notes

P1: ES256 + RS256, rotation, JWKS. P5: PS*/EdDSA/JWE. Crates `auvora-crypto`, `auvora-jose`, worker job `key_rotation`.

## Verification

Tests for algorithm confusion (RS→HS, `none`, `kid` injection), rotation race tests (tokens signed at T-1 s verify across rotation), JWKS cache-header tests, OIDF key-rotation tests.
