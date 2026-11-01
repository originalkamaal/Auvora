# ADR-0090: Custom domains and TLS

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §13.3, §5.3 (global router), §7.2 (Project issuer), §4 D10 (built-in TLS + ACME), §27 P3
- **Related:** ADR-0010, ADR-0012, ADR-0019, ADR-0026, ADR-0068, ADR-0088, ADR-0089

## Context

Login on the customer's own domain (`login.example.com`) is required for brand trust, first-party cookies (third-party cookie deprecation), passkey RP ID alignment and phishing resistance. Because no edge gateway is bundled (ADR-0010), the binary must handle TLS and certificates itself.

## Decision

1. **Domains per project:**
   - The default domain is `{project-slug}.{base_domain}`.
   - Up to 20 custom domains per project (configurable).
   - One domain is the **primary issuer** (`iss`). Others serve UI and flows and may optionally be alternate issuers only in `multi_issuer` mode (off by default, since it affects token validation).
   - Org-specific domains (ADR-0088) are allowed and map to the same project with an org hint.
2. **Verification:** a DNS CNAME to `{project-slug}.{base_domain}` (or an A/AAAA record for self-hosted) **plus** a TXT record `_auvora-challenge.{domain}`. Verification is re-checked every 24 h. A domain that fails verification for 7 days is disabled (with notifications at day 1 and day 5).
3. **Certificates:**
   - Built-in ACME client (RFC 8555): Let's Encrypt by default, ZeroSSL fallback, any ACME directory configurable.
   - HTTP-01 and TLS-ALPN-01 challenges; DNS-01 via provider plugins (Cloudflare, Route 53, Google Cloud DNS, Azure DNS) for wildcard or private cases.
   - Certificates are stored encrypted in Postgres (ADR-0072). Renewal at 30 days before expiry. Multi-node coordination via an advisory lock.
   - ECDSA P-256 certs by default (RSA optional).
   - Bring-your-own certificate is supported (upload or file path; reload on change).
4. **TLS settings:**
   - TLS 1.2+ (1.3 preferred), hybrid PQ key exchange (ADR-0083).
   - HSTS (`max-age=63072000; includeSubDomains` opt-in for preload).
   - OCSP stapling, SNI-based certificate selection.
5. **Routing:**
   - The edge role maps `Host` → project (and → cell in Cloud via the global router, ADR-0019) through an in-memory domain table, invalidated via the config propagation mechanism (ADR-0009).
   - An unknown host gets 421, with no default project fallback.
6. **Proxy deployments:** when TLS terminates at an external proxy, `trusted_proxies` CIDRs plus `X-Forwarded-*` / `Forwarded` handling apply. ACME is then disabled per domain, or HTTP-01 is passed through.
7. **Passkeys:** the RP ID defaults to the registrable domain of the primary custom domain. Related Origin Requests (`/.well-known/webauthn`) are auto-published for additional domains.
8. **Cookies:** `__Host-` cookies per domain (ADR-0032). Cross-domain SSO between custom domains goes through protocol redirects, never shared cookies.
9. **Configuration:** `[tls] acme_directory`, `acme_email`, `dns_providers.*`; `[edge] base_domain`, `trusted_proxies`; per-project domains through the API/Terraform.

## Rationale

Built-in ACME keeps the single-binary promise (ADR-0006) and matches the managed-domain experiences of Auth0, Clerk and Logto for self-hosted users too.

## Alternatives considered

- **Require an external proxy or cert-manager.** Rejected: dependency sprawl. Supported, but optional.
- **Wildcard certificate for all tenants only.** Rejected: doesn't cover custom domains.

## Consequences

- Positive: zero-config HTTPS on custom domains.
- Negative: ACME rate limits at scale in Cloud. Mitigated by a ZeroSSL fallback and a commercial CA option.

## Fallback / revisit trigger

If ACME rate limits are hit in Cloud, add a commercial ACME CA via configuration (no architecture change).

## Implementation notes

- Crates: `auvora-server` (edge role), `auvora-crypto` (cert storage), ACME via a vetted crate (e.g. `instant-acme`) behind a trait.
- Ships in P3. Default-domain TLS ships in P1.

## Verification

- An integration test with a Pebble ACME test server.
- A routing test where an unknown host returns 421.
- A renewal chaos test across nodes.
