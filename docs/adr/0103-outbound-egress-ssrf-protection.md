# ADR-0103: Outbound egress and SSRF protection

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §26 (SSRF abuse case), §2.3 of review, §5.2 (air-gapped profile)
- **Related:** ADR-0038, ADR-0080, ADR-0052, ADR-0087, ADR-0096, ADR-0099, ADR-0006, ADR-0049

## Context

Auvora fetches many tenant-controlled URLs:
- OIDC discovery and JWKS, SAML metadata;
- webhooks and sync hooks, Wasm `http-fetch`;
- DCR `jwks_uri`, `logo_uri` and `request_uri`, and Client ID Metadata Documents;
- SSF endpoints, SCIM client targets, proofing vendors, social providers.

In a multi-tenant service, any of these is an SSRF vector into cloud metadata services (169.254.169.254), internal networks, or Auvora's own admin endpoints.

## Decision

1. **All outbound HTTP(S) MUST go through `auvora-core::egress::Client`.** Constructing `reqwest`/`hyper` clients directly is forbidden outside that module. This is enforced by a `clippy` disallowed-methods rule and a CI grep check.
2. **Default policy (`[egress] mode = "public-only"`):**
   - Resolve DNS once. Reject if **any** resolved address is loopback, private (RFC 1918, ULA fc00::/7), link-local (169.254/16, fe80::/10), CGNAT (100.64/10), multicast, unspecified, IPv4-mapped/embedded forms of those, or in `[egress] deny_cidrs`.
   - **Connect to the validated IP** (pinning) to defeat DNS rebinding. TLS SNI and Host are kept.
   - Schemes `https` only. `http` is allowed only for loopback in dev projects (`environment = "dev"`).
   - Ports: 443 and 8443 by default; extendable via `allowed_ports`.
   - Redirects are not followed by default. Where a spec requires following them (discovery), at most 3 hops are allowed, each re-validated.
   - Limits: connect timeout 3 s, total 10 s (sync hooks use their own lower budget, ADR-0038), response size 1 MiB (metadata 5 MiB), no decompression bombs (ratio cap 20:1).
   - Request headers are stripped of Auvora internal headers. A `User-Agent: Auvora/<version>` is added.
3. **Modes:**
   - `public-only` (default).
   - `allowlist`: only hosts/CIDRs in `allow_hosts`/`allow_cidrs`. Used for regulated and air-gapped profiles.
   - `proxy`: all egress via `[egress] proxy_url` (corporate proxy). The deny rules still apply to the target host.
   - `off`: no outbound at all. Features that need egress fail with a clear error and the console shows them as unavailable.
4. **Per-tenant overrides:** a self-hosted operator may allow specific private CIDRs per project (e.g. an on-prem IdP): `[egress.projects.<id>] allow_cidrs = [...]`. Auvora Cloud never allows private ranges.
5. **Isolation:** in Cloud, worker egress runs from dedicated NAT addresses that customers can allowlist. Data-plane pods have no route to the cloud metadata service (an IMDS hop limit of 1 plus a network policy).
6. **Observability:** every blocked request emits `egress.blocked` (tenant, purpose, reason). Metrics are labelled by `purpose`.

## Rationale

One guarded client is the only reliable way to prevent SSRF across dozens of features. IP pinning closes DNS rebinding. The modes support the air-gapped and corporate-proxy profiles without a feature fork.

## Alternatives considered

- **Per-feature validation:** inconsistent, and one missed feature is enough for an exploit.
- **Rely on a network egress proxy only:** not available in the Postgres-only/small profile, and can't distinguish tenants.

## Consequences

- Positive: SSRF closed by construction; air-gapped support.
- Negative: on-prem IdPs need explicit operator configuration.
- Risk: parser differentials in URL handling. Mitigations: parse with the `url` crate once and pass the structured value to the client; fuzz the validator.

## Fallback / revisit trigger

None. This is a permanent security invariant. Bypasses are treated as vulnerabilities (ADR-0108).

## Implementation notes

- P0 (module skeleton + lint rule).
- P1: used by the first upstream fetches (breach check, ACME).
- Config namespace `[egress]`; env `AUVORA_EGRESS__MODE`.

## Verification

- An SSRF regression suite covering metadata IPs, decimal/octal/hex IP encodings, IPv6-mapped addresses, DNS rebinding (a test DNS server), redirect to private, and oversize/decompression bombs.
- The CI lint forbids direct HTTP clients.
- Fuzzing of URL validation for 24 h.
