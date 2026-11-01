# ADR-0029: Access token format and lifetime

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D29, §9.2, §9.4
- **Related:** ADR-0013, ADR-0030, ADR-0031, ADR-0033, ADR-0084, ADR-0086, ADR-0015

## Context

Resource servers need fast local validation (JWT) but sometimes central revocation or dynamic data (opaque + introspection). Tokens must be audience-restricted to prevent substitution across APIs, and minimal to avoid leaking data.

## Decision

1. **Default format:** JWT per RFC 9068.
   - Header `typ: at+jwt`, `kid` required.
   - Claims: `iss`, `sub`, `aud` (single resource server identifier), `exp`, `iat`, `nbf`, `jti`, `client_id`, `scope`, plus `auth_time`/`acr`/`amr` when user-bound, `cnf` when sender-constrained, `act` when delegated (ADR-0086), `org_id` when org-scoped (ADR-0088), and `authorization_details` when RAR is used.
2. **Opaque option** per resource server: `token_format = "jwt" | "opaque"`.
   - Opaque tokens are 256-bit random values, prefixed `auv_at_`, stored hashed (SHA-256), and validated via introspection (RFC 7662) or JWT introspection responses (RFC 9701).
3. **Audience:**
   - Every access token targets **one** resource server, selected via the `resource` parameter (RFC 8707) or the client's default resource server.
   - Requests spanning multiple unrelated resource servers receive an error (`invalid_target`). Clients obtain separate tokens via the refresh token.
   - `aud` arrays are not issued, except for the built-in `userinfo` audience combined with one resource server.
4. **Lifetime:** default **600 s** (10 min). Configurable per resource server between **60 s and 86,400 s**. The console warns above 3,600 s. M2M (client_credentials) default is 3,600 s.
5. **Claims minimization:**
   - Profile/PII claims are not included unless mapped explicitly per resource server (CEL mappers, ADR-0016).
   - Total JWT size budget is **4 KB** (`token.max_size_bytes`); issuance fails closed with a configuration error event if exceeded.
   - Role/permission claims follow ADR-0015 (embedded if under budget, otherwise queried).
6. **Sender constraint:** DPoP (`cnf.jkt`) or mTLS (`cnf.x5t#S256`) when the client profile requires it or the client presents proof (ADR-0084).
7. **Never in URLs:** access tokens are never placed in query strings or fragments (no implicit grant, ADR-0066).
8. **Signing:** per ADR-0031. JWE-encrypted access tokens are optional per resource server (`token_encryption`) for confidentiality-sensitive APIs.
9. **Validation requirements published to resource servers:** `iss` exact match, `aud` contains own identifier, `typ = at+jwt`, `alg` from the allowlist, `exp`/`nbf` with ±60 s skew, `cnf` verification when present. All Auvora SDKs enforce these (ADR-0024).

## Rationale

JWT gives scalable local validation. Short lifetimes bound the revocation gap (ADR-0033). Per-resource opaque tokens satisfy APIs needing instant revocation. Single audience prevents token substitution.

## Alternatives considered

- **Opaque by default**: forces introspection load on every API call. Rejected as default.
- **1-hour default lifetime**: too long a revocation gap. Rejected.
- **Multi-audience tokens**: substitution risk. Rejected.

## Consequences

- Positive: secure, scalable defaults; RFC 9068 interoperability.
- Negative: clients needing multiple APIs must refresh per audience.
- Risks and mitigations: claim bloat is prevented by the size budget with fail-closed issuance and a console warning.

## Fallback / revisit trigger

None. Defaults are revisited only if RFC 9700 or its successor changes guidance.

## Implementation notes

P1: JWT AT + RFC 8707 + introspection. P5: DPoP/mTLS binding, JWE, RFC 9701. Crate: `auvora-oauth`, `auvora-jose`. Resource-server config: `token_format`, `access_token_ttl`, `token_encryption`, `claim_mappers`.

## Verification

OIDF conformance; negative tests (wrong `aud`, wrong `typ`, `alg` confusion, expired, `nbf` future); size budget test; SDK conformance validation tests.
