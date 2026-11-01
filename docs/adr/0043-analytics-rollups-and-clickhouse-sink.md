# ADR-0043: Analytics: rollups, optional ClickHouse

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D43, §2.1 (analytics gap), §13.5, §22
- **Related:** ADR-0006, ADR-0008, ADR-0039, ADR-0040, ADR-0109, ADR-0063

## Context

Customers need product analytics: login funnels, MAU/DAU, method mix (passkey vs password), failure reasons, MFA coverage, risk events, agent and M2M token volume. Auvora Cloud needs MAU/M2M metering for billing. Raw events can reach billions of rows. The minimum profile is Postgres-only.

## Decision

1. **Source:** analytics derive from domain events in the outbox (ADR-0008), never from scraping audit tables at query time.
2. **Default: Postgres rollups** maintained by the `worker` role:
   - tables `analytics_rollup_{hourly,daily}` keyed by `(project_id, org_id NULL, app_id NULL, metric, dimension_hash, bucket_start)`;
   - metrics include `logins_success`, `logins_failure{reason}`, `signups`, `mfa_challenges{method}`, `active_users` (HyperLogLog via the `postgresql-hll` extension when available, else exact distinct per day with a 10 M cap per project-day), `tokens_issued{grant}`, `risk_actions{action}`, `agent_delegations`;
   - hourly rollups are kept 30 days and daily rollups 25 months (configurable `analytics.retention.*`).
3. **MAU for metering** uses an exact per-month distinct table `usage_active_principals(project_id, month, principal_id)` (one row per principal per month). Exactness is required for billing, and this table is never sampled.
4. **Optional sink:** `analytics.sink = "clickhouse"` streams events to ClickHouse via the outbox worker for ad-hoc and high-cardinality analysis. Postgres rollups keep running so that console dashboards never depend on ClickHouse.
5. **Privacy:** rollups hold no direct PII. Dimensions are limited to an allowlist (method, reason, country code, app, org, client). IPs are never stored in analytics. Erasure doesn't need to touch rollups (aggregates), except for the metering table, which keeps only the pseudonymous principal ID.
6. **API:** `GET /v1/projects/{id}/analytics/{metric}` + console dashboards; exportable as CSV and to a Prometheus-style endpoint for per-tenant metrics on Cloud.

## Rationale

Rollups keep dashboards fast and bounded in the minimum profile. ClickHouse serves analysts at scale without becoming a dependency.

## Alternatives considered

- **Query raw audit**: rejected because it's slow and couples analytics to audit retention.
- **TimescaleDB requirement**: rejected because it's an extension dependency not available on all managed Postgres offerings. It may be used opportunistically.
- **ClickHouse required**: rejected (ADR-0006).

## Consequences

- Positive: predictable dashboard latency; exact billing data.
- Negative: worker load. New metrics need rollup migrations.
- Risk: the HLL extension is missing on managed Postgres. Mitigated by the exact-count fallback with a cap.

## Fallback / revisit trigger

Rollup job lag > 15 min p95 at reference scale → move rollup computation to an incremental, partition-local strategy, or recommend the ClickHouse sink for the Scale profile.

## Implementation notes

P1 metering table, P3 dashboards, P5 agent/M2M metrics, P9 ClickHouse sink GA. Crates: `auvora-events` (consumer), `auvora-store`.

## Verification

Property tests compare rollups to recomputation from raw events in the test DB. The billing reconciliation job asserts that MAU equals distinct authenticated principals.
