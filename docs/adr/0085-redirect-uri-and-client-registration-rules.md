# ADR-0085: Redirect URI and client registration rules

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §9.4, §26 (open redirect, wildcard, loopback, consent phishing), §13.5
- **Related:** ADR-0052, ADR-0053, ADR-0084, ADR-0103, ADR-0070

## Context

Redirect URI manipulation is the most common OAuth vulnerability class (RFC 9700 §4.1). Developers want wildcard and preview-deployment URLs; security requires exact matching. Third-party clients enable consent phishing.

## Decision

1. **Matching:** exact string comparison after parsing, with no normalization beyond lowercasing the scheme and host. The only exception is the RFC 8252 loopback rule: `http://127.0.0.1:{any}/path` and `http://[::1]:{any}/path` match any port for `native` clients. `localhost` is not allowed (DNS resolution risk); use the IP literals.
2. **Allowed schemes:**
   - `https` (required in prod projects for web and SPA).
   - `http` only for loopback, or in projects marked `environment = dev`.
   - Private-use schemes for native apps must be reverse-DNS (`com.example.app:/cb`) per RFC 8252.
   - `javascript:`, `data:`, `file:` and `vbscript:` are rejected.
3. **No fragments.** No userinfo. Query strings are allowed but must match exactly.
4. **Wildcards:** allowed **only** in `dev` projects, for subdomain labels (`https://*.preview.example.com/cb`), single label, never the registrable domain, never in path. The console shows a persistent banner. Promotion to a prod project (ADR-0070) fails while wildcards exist. For preview deployments in prod-like environments, an Admin API endpoint `POST /v1/clients/{id}/redirect-uris:ephemeral` adds a URI with a TTL (max 7 days) and audit.
5. **Limits:** max 100 redirect URIs per client; max 2,048 chars each. Same rules apply to `post_logout_redirect_uris`, `frontchannel_logout_uri` and `backchannel_logout_uri` (back-channel must be https, SSRF-guarded via ADR-0103).
6. **Registration metadata validation:**
   - `logo_uri`, `policy_uri`, `tos_uri`, `client_uri` and `jwks_uri` must be https.
   - Fetched URIs (`jwks_uri`, `logo_uri`, `sector_identifier_uri`, `request_uri`) go through the egress guard.
   - Logos are proxied and re-encoded (PNG/WebP, ≤ 256 KiB), never hot-linked.
   - `sector_identifier_uri` contents are validated for pairwise subjects.
7. **Third-party client trust:** clients are `first_party` (set only by project admins) or `third_party`.
   - Third-party consent screens show the publisher's verification state: `unverified` (warning banner), `domain_verified` (DNS TXT on `client_uri` domain) or `publisher_verified` (manual or EE workflow).
   - Org admins can restrict third-party apps to an allowlist (`org.third_party_apps = allowlist|verified_only|any`).
8. **Error behaviour:** an invalid or unregistered `redirect_uri` is never redirected to. Auvora renders an error page in its own origin (no open redirect).
9. **Configuration:** `[projects.<id>.oauth.redirects] allow_dev_wildcards`, `ephemeral_max_ttl`; enforced through profiles (ADR-0084).

## Rationale

Exact matching is the RFC 9700 recommendation. Ephemeral URIs and dev-only wildcards cover the real developer need (preview deployments) without weakening production.

## Alternatives considered

- **Pattern/regex matching** (as in some products). Rejected: a history of bypasses, plus ReDoS.
- **Allowing `localhost`.** Rejected: resolution and port-hijacking risks.

## Consequences

- Positive: the open redirect and code-theft class is closed.
- Negative: preview-deployment workflows need an API call or CLI (`auvora clients redirect add --ttl 3d`).

## Fallback / revisit trigger

None for the matching rules. Publisher verification criteria may evolve via a new ADR.

## Implementation notes

- Crate: `auvora-oauth` (`redirect` module, a fuzzed URI parser using the `url` crate plus strict checks).
- Ships in P1. Publisher verification ships in P5.

## Verification

- A fuzz target for redirect matching.
- A negative test corpus from known bypasses (OAuth bypass cheat-sheets, double-encoding, IDN, backslash, `@`, trailing dots, case tricks).
- The OIDF suite's redirect tests.
