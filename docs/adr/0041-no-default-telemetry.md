# ADR-0041: No default telemetry

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D41, §5.2 (air-gapped profile)
- **Related:** ADR-0040, ADR-0044, ADR-0103, ADR-0063

## Context

Self-hosted Auvora runs inside customers' security perimeters, often with regulated or air-gapped constraints. An identity system that phones home by default erodes trust, triggers firewall alerts, may breach data-residency commitments, and contradicts the "air-gapped" deployment profile. The project still benefits from knowing which versions and features are in use.

## Decision

1. Self-hosted Auvora MUST NOT make any outbound network call to Auvora-operated endpoints unless an operator explicitly enables it.
2. Usage telemetry is **opt-in** via `telemetry.enabled = true` in `auvora.toml` (env `AUVORA_TELEMETRY__ENABLED=true`). The default is `false`.
3. When enabled, the payload is **anonymous and aggregate only**:
   - random installation ID (UUIDv4, regenerable via `auvora telemetry reset-id`), Auvora version, edition (OSS/EE), OS/arch, Postgres major version, enabled roles;
   - feature-usage booleans (e.g. `saml_sp_enabled`) and bucketed counts (projects: `1`, `2-10`, `11-100`, `>100`; users: power-of-10 buckets).
   - It MUST NOT contain: hostnames, IPs (the collector discards source IP before storage), domains, tenant/project/user IDs or names, emails, issuer URLs, config values, or error messages.
4. Frequency: at most once per 24 h, via HTTPS POST to `https://telemetry.auvora.dev/v1/report`, overridable with `telemetry.endpoint` (e.g. to an internal proxy).
5. `auvora telemetry preview` prints the exact JSON that would be sent. The payload schema is versioned and published in docs.
6. Release update checks are also opt-in (`update_check.enabled`, default `false`). The admin console may show "new version available" only when this is enabled.
7. Auvora Cloud operational metrics are governed by ADR-0040/0063, not by this ADR. Cloud is the operator's own deployment.
8. Third-party calls that features require (HIBP, social IdPs, CAPTCHA, comms providers) are not telemetry. They are listed in the docs' "Outbound connections" page, each with a disable switch, and go through the egress layer (ADR-0103).

## Rationale

Trust is the product. An opt-in, previewable, minimal payload gives useful adoption signals without compromising the air-gapped profile or any residency promise.

## Alternatives considered

- **Opt-out telemetry** (common in OSS): rejected because it breaks air-gapped expectations and enterprise trust.
- **No telemetry at all**: rejected because the project loses all version/feature-adoption signal. Opt-in keeps the option available.
- **Third-party analytics SDK** (Segment, PostHog): rejected because of an extra dependency and data-processor concerns.

## Consequences

- Positive: zero surprise network traffic; easy security review; GDPR-neutral.
- Negative: adoption data will be sparse and biased toward opted-in users.
- Risks: a contributor adds an accidental outbound call. Mitigated by the egress test below.

## Fallback / revisit trigger

None (permanent). Payload fields can be extended only via a new ADR.

## Implementation notes

- P1: `auvora-server` telemetry module (disabled path compiled in, no-op).
- P3: collector service and docs page; console toggle with preview.
- Air-gapped profile docs show that all outbound features are off.

## Verification

- An integration test runs the full server with an egress-denying network namespace and default config and asserts zero outbound connection attempts.
- A unit test asserts the telemetry payload schema contains no field outside the allowlist.
