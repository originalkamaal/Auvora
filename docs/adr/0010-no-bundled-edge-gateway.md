# ADR-0010: No bundled edge gateway

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D10, §5.1, v0.2 §3.2 ("Envoy or Kong")
- **Related:** ADR-0005, ADR-0006, ADR-0037, ADR-0074, ADR-0090, ADR-0100

## Context

v0.2 listed Envoy or Kong as the API edge. Bundling a third-party gateway adds a runtime (Kong: OpenResty/Lua; Envoy: C++ with xDS config) and contradicts ADR-0006. Auvora still needs edge capabilities: TLS, custom domains, ACME, routing, rate limits, bot checks and request size limits.

## Decision

1. Auvora **does not bundle or require** any reverse proxy or API gateway.
2. The built-in **`edge` role** provides:
   - TLS via rustls (aws-lc-rs, ADR-0012), TLS 1.2+ (1.3 preferred), with PQ hybrid key exchange enabled (ADR-0083);
   - ACME certificate issuance and renewal for platform and custom domains (ADR-0090);
   - routing of Host → project (custom domain or `{project}.{base_domain}`) and path → role/handler;
   - rate limiting (ADR-0074) and bot-protection checks (ADR-0037);
   - request limits: header size 16 KiB, body 1 MiB default (SCIM bulk: 10 MiB, configurable);
   - slowloris protection and connection limits;
   - security headers (HSTS, CSP for hosted UI per ADR-0068);
   - HTTP/1.1 and HTTP/2; HTTP/3 (QUIC) optional behind a flag from P5.
3. **Running behind an existing proxy or CDN** is fully supported:
   - Trusted proxy CIDRs: `[edge] trusted_proxies = [...]`. Only these may set `X-Forwarded-For`/`Forwarded`/`X-Forwarded-Proto`.
   - mTLS client certificate forwarding (RFC 8705 deployments) via a configured header, accepted only from trusted proxies, with a format option (`xfcc` Envoy, `ssl-client-cert` nginx RFC 9440 `Client-Cert`).
   - TLS termination at the proxy is allowed. The ACME and cert features are then disabled per domain.
4. **Reference configs** are maintained and tested in CI under `deploy/reference-proxies/`: nginx, Envoy, Traefik, Caddy, HAProxy, Cloudflare, AWS ALB, GCP LB, and Kubernetes Gateway API / Ingress-NGINX.
5. The **identity-aware proxy** for protecting *other* apps is a separate role (`proxy`, ADR-0100) and is unrelated to the edge.

## Rationale

Most deployments already have a proxy, load balancer or CDN, so mandating one adds cost without value. Built-in edge features cover the minimum profile and keep security-relevant logic (rate limits per identifier, bot checks) in Auvora, where identity context exists.

## Alternatives considered

- **Bundle Envoy:** extra runtime and xDS complexity. Rejected.
- **Bundle Kong:** Lua/OpenResty plus a DB, and plugin licensing concerns. Rejected.
- **No edge features at all:** forces a proxy, which breaks ADR-0006. Rejected.

## Consequences

- Positive: fewer moving parts; identity-aware rate limiting; easy integration with any infrastructure.
- Negative: Auvora maintains TLS/ACME/HTTP edge code.
- Risks: header spoofing when proxies are misconfigured. Mitigation: deny forwarded headers by default; startup warnings when listening on a public interface without trusted proxies set.

## Fallback / revisit trigger

None.

## Implementation notes

- Crates: `auvora-server` (listeners, routing), `auvora-api` (handlers). Config `[edge]`, `[server.listeners.edge] bind = "0.0.0.0:443"`.
- P1: TLS, routing, limits, trusted proxies. P3: ACME + custom domains. P5: HTTP/3 flag.

## Verification

- CI runs E2E behind each reference proxy config (nightly matrix).
- Tests assert that forwarded headers from untrusted sources are ignored.
- TLS config is checked against a testssl.sh baseline in CI.
