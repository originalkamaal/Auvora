# ADR-0052: Dynamic client registration and CIMD

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D52, §9.2, §9.3 (`mcp` profile), §11.2
- **Related:** ADR-0013, ADR-0084, ADR-0085, ADR-0091, ADR-0103, ADR-0053, ADR-0051

## Context

OIDC Dynamic certification, open ecosystems and especially MCP clients need clients to register without an admin. Open registration is also a consent-phishing and spam vector. Client ID Metadata Documents (CIMD, IETF OAuth draft) let a client use an HTTPS URL as its `client_id`, which the AS fetches. This avoids registration-state explosion for MCP.

## Decision

1. **Supported:** RFC 7591 (DCR) and RFC 7592 (DCRM: read/update/delete with registration access token), and Client ID Metadata Documents.
2. **Default: off.** Configured per project via `oauth.dcr.mode`:
   - `disabled` (default);
   - `initial_access_token`: requires an IAT minted by admins (scoped: allowed profiles, scopes, max clients, expiry);
   - `software_statement`: requires a signed software statement (RFC 7591 §2.3) from a trusted issuer in `oauth.dcr.trusted_software_issuers`;
   - `open`: anyone. Allowed only in combination with policy restrictions (below), and the console shows a warning.
3. **CIMD:** `oauth.cimd.enabled` (default `false`; `true` when the `mcp` profile is enabled on any resource server). Rules:
   - `client_id` MUST be an `https` URL with a path. The metadata is fetched through the egress layer (ADR-0103) with a 5 s timeout, a 64 KiB size limit, no redirects to private ranges, and `client_id` equality checked against the document.
   - Cache honors HTTP cache headers, clamped to 5 min–24 h.
   - Domains can be allowlisted or denylisted per project.
   - `jwks_uri`/`jwks` are supported for `private_key_jwt`; otherwise the client is public (PKCE mandatory).
   - `redirect_uris` from the document follow ADR-0085 rules.
4. **Policy for dynamically registered/CIMD clients** (CEL, `oauth.dcr.policy`): allowed client profiles (default: `baseline`, `spa`, `native`, `mcp`), allowed scopes/resources (default: none beyond `openid`; resource servers opt in), grant types (code + refresh only by default; client_credentials never via open DCR), token lifetimes, a max number of clients per IAT/IP/day (default 100 per IAT, 10 per IP/day for `open`).
5. **Trust markers:** dynamically registered and CIMD clients are **third-party** (consent always shown, ADR-0053), labelled "unverified" on the consent screen unless the publisher's domain is verified (DNS TXT or CIMD URL host matches a verified domain in the project).
6. **Lifecycle:** unused DCR clients expire after `oauth.dcr.inactive_expiry_days = 90`. Admins can review, suspend or delete them. All registrations are audited with the source IP and IAT ID.
7. **Metadata stored** is validated against the profile: `logo_uri`/`policy_uri`/`tos_uri` must be `https`; logos are fetched via egress and re-hosted (never hot-linked) after image validation.

## Rationale

This delivers OIDF Dynamic + MCP compatibility (superset goal) with safe defaults that neutralize registration abuse and consent phishing.

## Alternatives considered

- **Open DCR by default** (some MCP guidance): rejected as an abuse vector.
- **CIMD only, no DCR**: rejected because OIDF Dynamic and existing ecosystems require DCR.

## Consequences

- Positive: frictionless MCP onboarding when enabled; certification coverage.
- Negative: egress fetches in the authorization path for CIMD (mitigated by caching).

## Fallback / revisit trigger

When the CIMD draft becomes an RFC or changes materially, update the implementation behind the `cimd` feature flag. A breaking change triggers a new ADR.

## Implementation notes

P3 DCR/DCRM (for OIDF Dynamic); P5 CIMD + `mcp` profile. Crate `auvora-oauth`. Endpoints: `/oauth2/register`, `/oauth2/register/{client_id}`.

## Verification

OIDF Dynamic plan; abuse tests (SSRF via CIMD/logo/jwks URIs, registration flood, scope escalation via update); MCP client interop (ADR-0060).
