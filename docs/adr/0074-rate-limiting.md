# ADR-0074: Rate limiting

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §8.2, §5.1 (edge role), §26, review §2.3
- **Related:** ADR-0010, ADR-0028, ADR-0035, ADR-0037, ADR-0073, ADR-0097, ADR-0036

## Context

The identity edge faces credential stuffing, brute force, SMS pumping, email bombing, enumeration and noisy tenants. No gateway is bundled (ADR-0010), so Auvora must rate limit natively, from P1.

## Decision

1. **Algorithm:**
   - GCRA (generic cell rate algorithm; token-bucket equivalent) for request limits.
   - Sliding-window counters for failure-based limits (e.g. failed logins per identifier).
2. **Dimensions** (combined; any one can trigger):
   - Per IP and per /64 IPv6 prefix.
   - Per identifier (normalized email/phone/username; blind-indexed).
   - Per client_id.
   - Per project.
   - Per tenant.
   - Per destination (email address / phone number / phone prefix + country for comms).
   - Per principal for authenticated APIs.
3. **Default policy** (overridable per project within platform maxima):

| Endpoint class | Default |
|---|---|
| Password/OTP verify | 5 failures / identifier / 15 min → progressive delay (1, 2, 4, 8 s…) → soft lock 15 min with notification; 100 attempts / IP / 5 min |
| OTP/magic-link send | 1 per 30 s, 5 per hour per destination; 10/min per IP; SMS per-country velocity caps |
| Registration | 10 / IP / hour (risk-adjusted) |
| Token endpoint | 50 req/s per client (burst 100) |
| Admin API | 20 req/s per principal (burst 40) |
| Flow API | 30 req/min per flow; 60 req/min per IP |
| JWKS/discovery | Not limited per client; global protection only |

4. **Tenant fairness:**
   - Per-project and per-tenant global quotas (Cloud plans) prevent noisy neighbours.
   - Argon2 hashing has its own semaphore (ADR-0028).
5. **Responses:**
   - `429` with `Retry-After` and RFC 9457 body (`type: …/rate-limited`). Credential endpoints return uniform responses that don't confirm identifier existence (ADR-0035).
   - `RateLimit-Policy` / `RateLimit` headers (IETF draft) on APIs, not on credential endpoints.
6. **Storage and fail modes:** per ADR-0073. Credential endpoints fail closed; public metadata endpoints fail open with a local limiter.
7. **Client IP:**
   - Taken from the socket by default.
   - `X-Forwarded-For`/`Forwarded` is honoured only from `edge.trusted_proxies` CIDRs.
   - Cloudflare/Fastly/Akamai presets are available.
8. **Allowlists:** per-project allowlist of CIDRs or client IDs (e.g. for load tests), audited.
9. Limits emit metrics and `auvora.ratelimit.triggered.v1` events (sampled) for the risk engine (ADR-0097).

## Rationale

Native, identity-aware limits (per identifier and per destination) can't be expressed by generic gateways.

## Alternatives considered

- **Delegating to Envoy/Kong**: not identity-aware; adds a dependency.
- **Hard permanent lockout**: enables DoS against users.

## Consequences

- Positive: protection from day one in all profiles.
- Negative: tuning burden. Mitigation: sane defaults plus dashboards.

## Fallback / revisit trigger

If GCRA in Postgres can't sustain ADR-0105 throughput, Valkey becomes the recommended store for Scale profiles. The algorithm is unchanged.

## Implementation notes

- `auvora-risk::ratelimit` as tower middleware, plus in-handler checks for identifier-keyed limits.
- Config: `[ratelimit]` in `auvora.toml`, plus per-project overrides via the Admin API.
- P1.

## Verification

- k6 abuse scenarios (stuffing, SMS pumping, email bombing) in CI nightly.
- Unit tests of GCRA math.
- A trusted-proxy spoofing test.
