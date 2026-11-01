# ADR-0024: SDK strategy and tiers

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D24, §13.2, §20.3, §27 (P3, P5, P10)
- **Related:** ADR-0020, ADR-0022, ADR-0023, ADR-0067, ADR-0069, ADR-0091, ADR-0102

## Context

The v0.2 plan listed 10+ SDKs with no prioritization, and SDK maintenance often costs more than the server. Auvora needs broad coverage (no feature compromise) while keeping SDKs correct, thin and secure. Plan principle: "SDKs must not hide protocol behavior or weaken security defaults."

## Decision

1. **Three SDK categories:**

   | Category | Purpose | Production |
   |---|---|---|
   | Admin SDKs | Call the Admin API (`/v1` control plane) | **Generated** from OpenAPI 3.1, with a thin hand-written ergonomic layer (pagination iterators, retries, idempotency keys) |
   | Runtime/client SDKs | Drive the Flow API and OAuth/OIDC flows from apps (browser, mobile, desktop, CLI) | **Hand-written**, thin, protocol-transparent |
   | Resource-server SDKs | Validate tokens (JWKS cache, DPoP, mTLS binding, `act`/support restrictions) and call AuthZEN | **Hand-written** middleware per framework |

2. **Mobile:** native Swift (SPM) and Kotlin (Maven Central), hand-written. React Native and Flutter wrap the native SDKs through platform bridges. There's no shared Rust core via UniFFI.
3. **Tiers:**
   - **Tier 1 (P3):** `@auvora/js` (browser + Node core), `@auvora/react`, `@auvora/nextjs`, `@auvora/node` (Express/Fastify/Nest middleware), `@auvora/elements`, Go (`net/http`, gin, echo, chi), Python (FastAPI, Django, Flask).
   - **Tier 2 (P5):** Swift, Kotlin, React Native, Flutter, Java (Spring Security/Boot starter), .NET (ASP.NET Core), Rust (axum/tower middleware), `@auvora/mcp` (TS, Python, Go, Rust).
   - **Tier 3 (P10):** PHP (Laravel, Symfony), Ruby (Rails), Elixir, Vue/Angular/Svelte wrappers beyond `@auvora/elements`, Unity/game engines on demand.
4. **Support commitments per tier:**
   - Tier 1: same-day support for new server features, full E2E in main CI.
   - Tier 2: support within 1 minor release.
   - Tier 3: best effort, community co-maintainers allowed.
5. **Security defaults every SDK MUST implement:**
   - PKCE S256 always.
   - `state` + `nonce` validation.
   - `iss` response parameter validation (RFC 9207).
   - Strict `aud`/`iss`/`typ` checks.
   - No tokens in URLs or localStorage by default. The browser SDK uses in-memory tokens plus a refresh via DPoP-bound refresh token, or the BFF pattern (`@auvora/nextjs`, `@auvora/node`).
   - Native SDKs store secrets in Keychain/Keystore only.
6. **Versioning:** SemVer per SDK. SDKs declare the minimum server version they support. A compatibility matrix is published in the docs.
7. **Conformance:** every Tier 1/2 SDK runs a shared **SDK conformance suite** (a black-box scenario set against `auvora dev`).

## Rationale

Generated admin SDKs give coverage at near-zero cost. Hand-written runtime SDKs avoid leaky, insecure generated auth code. Native mobile without a Rust core keeps platform idioms and avoids FFI build complexity on iOS/Android.

## Alternatives considered

- **All SDKs generated**: poor ergonomics and unsafe defaults for auth flows. Rejected.
- **Rust core + UniFFI bindings**: complex builds, binary size, debugging pain for mobile developers. Rejected.
- **Only OIDC-generic libraries (AppAuth etc.)**: no Flow API or headless support. Kept only as a documented interop option.

## Consequences

- Positive: broad coverage with controlled cost; consistent security defaults.
- Negative: hand-written runtime SDKs per language need maintainers.
- Risks and mitigations: SDK drift is mitigated by the shared conformance suite and the compatibility matrix.

## Fallback / revisit trigger

If a Tier 2/3 SDK misses two consecutive minor releases of support, demote it a tier and announce it. Introducing a shared native core requires a new ADR.

## Implementation notes

- Repos per ADR-0023.
- Codegen tool: `openapi-generator` or language-specific generators chosen per language in P3. Generated code is committed.
- The SDK conformance suite lives in `conformance/sdk/`.

## Verification

SDK conformance suite in each SDK repo's CI; security-defaults checklist in each SDK's release template; docs compatibility matrix generated from CI results.
