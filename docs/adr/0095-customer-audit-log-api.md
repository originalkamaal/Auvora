# ADR-0095: Customer Audit Log API

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §2.1 (WorkOS row), §6.2, §27 P4
- **Related:** ADR-0039, ADR-0080, ADR-0088, ADR-0089, ADR-0109, ADR-0017

## Context

B2B SaaS developers are routinely asked by their enterprise customers for an audit log of what happened *inside the developer's application*: who exported a report, who changed a setting. WorkOS sells this as a product. Auvora already runs a tamper-evident, tenant-aware audit store (ADR-0039) with org scoping (ADR-0088). Letting developers write their own application events into it closes a superset-matrix gap at little cost, and gives enterprise customers one place to see identity events and application events together.

## Decision

1. Auvora exposes a **Customer Audit Log API** in `auvora-audit`, served by the `data` role:
   - `POST /v1/audit-logs/events` (single) and `POST /v1/audit-logs/events:batch` (≤ 500 events, ≤ 1 MiB).
   - `GET /v1/audit-logs/events` with cursor pagination and filters: `organization_id`, `actor.id`, `action`, `target.type`, `target.id`, time range.
   - `POST /v1/audit-logs/exports` for asynchronous CSV/NDJSON exports, delivered as a signed download URL valid for 24 h.
2. **Authentication:** service-account tokens with scope `audit_logs:write` or `audit_logs:read`. The resource server is `auvora:audit`. Org admins read their org's events through the Org Admin Portal (ADR-0089) with `audit_logs:read` scoped to their org.
3. **Event schema (validated):**
   - `action`: matches `^[a-z0-9_]+(\.[a-z0-9_]+){1,4}$`. Every action must be registered in a per-project **action schema** that declares allowed `target` types and `metadata` JSON Schema. Unregistered actions are rejected with 422 in `strict` mode; `permissive` mode is the default in dev projects only.
   - `occurred_at`: accepted if within ±24 h of server time, otherwise 422.
   - `actor` {type, id, name?}, `targets[]` (≤ 10), `context` {ip?, user_agent?, location?}, `metadata` (≤ 16 KiB).
   - `organization_id` is required for org-visible events.
   - Optional `idempotency_key`; deduplication window 24 h.
4. **Storage:** events go to the same partitioned, append-only audit store as Auvora's own events, but are tagged `source=customer` and kept in a separate hash chain per project+source (ADR-0039). Customer events can never be mixed into, or forge, Auvora system events: `source` is set server-side.
5. **Retention:** follows ADR-0109 (OSS default 90 days; EE configurable up to 7 years; WORM export via ADR-0039).
6. **Streaming:** customer events can be streamed to the org's own SIEM sinks configured in the Org Admin Portal (Splunk HEC, Datadog, S3, generic webhook). Delivery semantics follow ADR-0080.
7. **Limits:** default 100 events/s per project. Rate limiting follows ADR-0074; quotas are configurable per tenant plan in Cloud.
8. **Config:** `[audit.customer_api] enabled = true`, `strict_actions = true`, `max_batch = 500`.

## Rationale

It reuses audit infrastructure that has to exist anyway (embedded-first; works on the Postgres-only profile). It provides WorkOS-equivalent value without a second product to operate, and it gives enterprises one place for identity and application audit.

## Alternatives considered

- **Out of scope; developers use their own logging:** leaves a superset gap; enterprises would have to stitch logs together.
- **A separate store (ClickHouse) by default:** violates ADR-0006. ClickHouse remains an optional analytics sink (ADR-0043).
- **Free-form events without schemas:** causes query and export inconsistency, and makes PII leaks likely.

## Consequences

- Positive: a sellable B2B feature; a single SIEM integration point for enterprise customers.
- Negative: storage volume grows with customer traffic. Partitioning and retention (ADR-0109) and per-plan quotas mitigate it.
- Risk: customers write PII or secrets into metadata. Mitigations: schema validation, a documented PII guidance, and an optional `metadata` redaction rule set (key-name deny-list) applied before storage.

## Fallback / revisit trigger

If customer-event write volume exceeds 20% of the audit table's write IOPS in any Cloud cell for 7 consecutive days, move customer events to their own partitioned table set or database within the cell. The API stays unchanged.

## Implementation notes

- Phase P4. Crates: `auvora-audit`, `auvora-api`, `apps/org-portal` (viewer).
- SDK helpers come in tier-1 SDKs (`auvora.auditLogs.create()`).
- The action schema is managed through the Admin API, Terraform and config-as-code (ADR-0070).

## Verification

- Contract tests on the schema and limits.
- A property test confirms customer events can't set `source=system` or join the system hash chain.
- An RLS test confirms cross-project reads are denied.
- Load test: 100 events/s per project sustained, p99 write < 30 ms.
