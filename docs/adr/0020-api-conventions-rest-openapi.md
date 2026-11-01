# ADR-0020: API conventions: REST + OpenAPI 3.1

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D20, §1.2 principle 15, §29
- **Related:** ADR-0005, ADR-0018, ADR-0024, ADR-0045, ADR-0070, ADR-0080

## Context

Auvora exposes:
- the Admin API (control plane);
- runtime APIs (Flow API, account APIs, AuthZEN);
- protocol endpoints defined by external specs.

Principle 15 says everything (console, CLI, Terraform, SDKs, MCP) is a client of the same public API. Without conventions, APIs drift, SDK generation breaks, and breaking changes slip through.

## Decision

1. **Style:** REST over HTTPS with JSON (`application/json`, UTF-8). Protocol endpoints (OAuth, OIDC, SAML, SCIM, AuthZEN, SSF) follow their own specs exactly and are exempt from conventions that conflict.
2. **Contract-first:** OpenAPI 3.1 documents in `crates/auvora-api/openapi/` are the source of truth. Handlers are checked against them (generated types + a schema conformance test). The spec is published at `/.well-known/openapi/{admin|runtime}.json`.
3. **Versioning:**
   - URL major version `/v1/...`.
   - Within a major version, changes are **additive only**: new endpoints, new optional fields, new enum values. Clients must tolerate unknown fields and enum values (documented).
   - Breaking changes need `/v2` alongside `/v1`, with `/v1` supported ≥ 24 months after `/v2` GA.
   - Deprecations use the `Deprecation` and `Sunset` headers (RFC 9745 / RFC 8594).
4. **Resource paths:**
   - plural nouns, prefixed IDs (ADR-0018);
   - nesting max 2 levels (`/v1/projects/{prj}/users/{usr}`);
   - actions as sub-resources with POST (`/users/{id}/suspend`).
5. **Errors:** RFC 9457 Problem Details (`application/problem+json`), with `type` URIs under `https://auvora.dev/problems/<slug>`, `title`, `status`, `detail`, `instance`, plus `code` (stable machine code), `request_id` and `errors[]` for validation (JSON Pointer to the field).
6. **Pagination:** cursor-based. Parameters `limit` (default 50, max 200) and `cursor`. Response `{ "data": [...], "next_cursor": "...|null" }`. Offset pagination is not offered.
7. **Filtering and sorting:** `filter` with a documented subset of CEL-like predicates (`email == "x"`, `created_at > "…"`). `sort=field,-field`. Every field allowed in a filter or sort is index-backed and declared in the OpenAPI `x-filterable` extension.
8. **Idempotency:** all non-idempotent POSTs accept `Idempotency-Key` (≤ 255 chars). Keys are stored 24 h per (project, actor, key) with request hash. Mismatched reuse returns 422.
9. **Concurrency:** resources return `ETag` (entity version). `PATCH`/`PUT`/`DELETE` accept `If-Match`, and a mismatch returns 412. Mutable config endpoints require `If-Match` in the console and Terraform provider.
10. **PATCH:** JSON Merge Patch (RFC 7396) by default. JSON Patch (RFC 6902) is accepted where arrays need precise edits (declared per endpoint).
11. **Auth for Admin API:** OAuth 2.0 bearer tokens (preferably DPoP-bound) issued by the platform's own project (dogfood). Scopes follow `admin:<resource>:<read|write>`. API keys (ADR-0093) are allowed for automation.
12. **Rate-limit headers:** `RateLimit-Policy`/`RateLimit` (IETF draft structured fields) + `Retry-After` (ADR-0074).
13. **Time and formats:** RFC 3339 UTC timestamps with milliseconds; durations in ISO 8601; money/counts as integers; JSON field names in `snake_case`.
14. **Internal RPC:** gRPC (tonic) is allowed only between role-split instances when an ADR introduces such a split (ADR-0005). It is never public.

## Rationale

REST + OpenAPI maximizes ecosystem compatibility (generators, Terraform, gateways). Cursor pagination, ETags and idempotency are required for correctness at scale and for automation.

## Alternatives considered

- **GraphQL for Admin API:** authorization complexity, caching issues, poorer Terraform fit. Rejected.
- **gRPC public API:** poor browser and Terraform ergonomics. Rejected.
- **Date-based versioning (Stripe):** heavy machinery for a self-hosted product with mixed versions. Rejected.

## Consequences

- Positive: predictable, generatable, automatable.
- Negative: some verbosity; additive-only discipline constrains redesigns.
- Risks: accidental breaking changes. Mitigation: oasdiff in CI.

## Fallback / revisit trigger

None.

## Implementation notes

- Crate `auvora-api` with `utoipa` or a generated-types approach (decided in P0 spike; either way the OpenAPI file is canonical).
- P0: style guide `docs/specs/api-style.md`, lint (Spectral ruleset `auvora.spectral.yaml`).

## Verification

- CI: Spectral lint, oasdiff breaking-change check against the last release (fails on breaking changes within `/v1`), and a schema conformance test (responses validated against OpenAPI in integration tests).
- Every Admin API operation has Terraform and CLI coverage (parity check, ADR-0070).
