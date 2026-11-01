# ADR-0093: API keys (last resort)

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §2.3 (long-lived API keys row), §7.2 (Credential `api_key`), §10 (service account credentials), §9.2 (introspection)
- **Related:** ADR-0092, ADR-0018, ADR-0028, ADR-0029, ADR-0074, ADR-0101

## Context

Short-lived tokens via workload federation or `private_key_jwt` are preferred. However, many real integrations need static credentials: legacy tools, webhooks from SaaS products, simple scripts, and customers' own end-user API keys (developers asking Auvora to manage API keys for *their* users, as Unkey and Zuplo do). Refusing them would compromise features. Unmanaged, they're a top breach vector.

## Decision

1. **Two use cases, one implementation:**
   - (a) **Platform keys:** credentials for Auvora service accounts and agents to call Auvora or obtain tokens.
   - (b) **Managed keys:** keys issued to the customer's users or orgs for the customer's own APIs, verified via Auvora (introspection or SDK).
2. **Format:** `auv_{env}_{type}_{base62(random 32 bytes)}_{crc32-checksum}`, for example `auv_live_sk_…`.
   - `env` ∈ {`live`, `test`}; `type` ∈ {`sk` (secret), `rk` (restricted)}.
   - The checksum lets secret scanners validate keys offline.
   - Customers may set a custom prefix for managed keys (≤ 8 chars, `[a-z0-9]`).
3. **Storage:** only an HMAC-SHA-256 (keyed by a per-project `api_key_hmac` key class, an extension of ADR-0082) of the secret is stored, plus the first 8 characters for display. Keys are shown once at creation and are never retrievable.
4. **Mandatory attributes:**
   - Owner principal; scopes (required, no default "all").
   - Expiry: default 90 days, max 365 days. A non-expiring key requires the project policy `api_keys.allow_no_expiry = true` and an `Admin` role, and is flagged in posture reports.
   - Optional: IP/CIDR allowlist, rate limit, allowed audiences, metadata.
5. **Verification:** constant-time lookup by HMAC. Results are cached for ≤ 60 s with revocation-aware invalidation (ADR-0009 notify). `last_used_at` and `last_used_ip` are updated asynchronously (batched). Managed-key verification is available via:
   - introspection (`token_type_hint=api_key`)
   - an AuthZEN request context
   - SDK middleware with a local cache
6. **Exchange preferred:** platform keys can be exchanged for a short-lived access token (client-credentials-style token exchange), so downstream calls use JWTs and the key travels once.
7. **Rotation:** `rotate` creates a new key and keeps the old one valid for a grace period (default 24 h, max 30 d), then revokes it. Expiry warnings go out at 14, 7 and 1 days (events + email).
8. **Leak response:** Auvora registers its prefix format with GitHub Secret Scanning partner programme and others. Reports of leaked keys trigger automatic revocation (configurable: `revoke` | `notify`) plus an audit and SSF event.
9. **Governance:** included in the credential inventory, stale-key detection (unused 90 d), and access reviews (EE, ADR-0101).
10. **Configuration:** `[projects.<id>.api_keys] default_ttl`, `max_ttl`, `allow_no_expiry`, `custom_prefix`, `leak_action`, `rotation_grace`.

## Rationale

API keys are supported because the need is real, but with the safest known design: prefixed, checksummed, hashed, scoped, expiring, rotatable and scanner-integrated. Offering managed keys also covers API-key-management products, which supports the superset goal.

## Alternatives considered

- **No API keys.** Rejected: blocks legitimate integrations; users would invent worse workarounds.
- **Reversibly encrypted keys.** Rejected: unnecessary. Hashing is enough.

## Consequences

- Positive: secure static credentials with full lifecycle; a new product surface (managed keys).
- Negative: verification lookups on the hot path. Mitigated by caching.

## Fallback / revisit trigger

None.

## Implementation notes

- Crates: `auvora-authn` (`api_keys`), `auvora-oauth` (introspection, exchange), `auvora-workload` (service-account binding).
- Platform keys ship in P3. Managed keys and leak-scanning integration ship in P5.

## Verification

- A checksum and format test against secret-scanner regex.
- A timing-safe lookup test.
- Expiry and rotation tests.
- A revocation propagation test (≤ 5 s).
