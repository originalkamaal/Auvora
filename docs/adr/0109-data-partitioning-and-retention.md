# ADR-0109: Data partitioning and retention

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §6.1, §6.2, §3.2 (EE retention), §5.4, §25 (scale)
- **Related:** ADR-0017, ADR-0039, ADR-0054, ADR-0071, ADR-0079, ADR-0095, ADR-0043, ADR-0019, ADR-0025

## Context

High-volume tables (audit, domain events/outbox, sessions, token metadata, risk evaluations, webhook deliveries, provisioning logs, entity history) would grow without bound and degrade Postgres. Retention also has legal meaning: GDPR minimization, audit evidence and legal hold. The Postgres-only profile must handle 100 M users per project by design (ADR-0105).

## Decision

1. **Partitioning:** native Postgres declarative **range partitioning by time**. Partitions are created ahead (7 days ahead) and dropped by the worker role. No `pg_partman` dependency (embedded-first). Hash partitioning isn't used; tenancy is enforced by RLS plus `(project_id, …)` leading index columns (ADR-0017).

| Table family | Partition | Default retention (OSS) | EE max | Notes |
|---|---|---|---|---|
| `audit_events` (system + customer) | daily | 90 days | 7 years | Hash-chain checkpoints exported to WORM before drop (ADR-0039) |
| `outbox_events` | daily | 7 days after delivered | 30 days | Undelivered rows are never dropped; they alert instead |
| `webhook_deliveries` | daily | 30 days | 90 days | Payload bodies kept 7 days, then metadata only |
| `risk_evaluations` | daily | 30 days | 180 days | Aggregated into rollups before drop (ADR-0043) |
| `sessions` (ended) | monthly | 30 days after end | 1 year | Active sessions live in the non-partitioned `sessions_active` |
| `token_metadata` (refresh families, revocations) | monthly | Expiry + 30 days | Expiry + 1 year | Needed for reuse detection |
| `provisioning_logs`, `sync_runs` | daily | 30 days | 1 year | |
| `*_history` (entity history, ADR-0071) | monthly | 1 year | 10 years | Identity time travel horizon = retention |
| `security_events` (subset of audit: auth failures, lockouts, admin actions) | daily | 1 year | 10 years | Separate from activity events (Q76) |
| `analytics_rollups` | monthly | 2 years | 10 years | Aggregates only; no PII |

2. **Configuration:** `[retention.<family>] days = N` per installation, overridable per project within EE limits. OSS caps at the defaults above, except `security_events`, which is configurable up to 1 year in OSS.
3. **Legal hold:** a hold on a project, org or principal pauses drops for affected rows. Partitions containing held rows are kept (detached and moved to an archive schema) until released. Holds are audited and require the `Owner` or `Auditor` role (ADR-0055).
4. **Erasure interplay (ADR-0054, ADR-0079):** principal erasure follows the canonical scheme in ADR-0039 §7 (PII in the `audit_event_pii` sidecar under a per-principal DEK, chain over an HMAC; purge = crypto-shred + sidecar delete). Retained rows keep the opaque `principal_id`; history rows are tombstoned per ADR-0054.
5. **Archival:** before a partition is dropped, families marked `archive = true` (audit by default in EE) are exported as compressed NDJSON + manifest + signature to object storage (S3/GCS/Azure/filesystem).
6. **Autovacuum and indexes:** per-partition indexes; BRIN on time columns; `fillfactor` tuned for append-only tables. Vacuum settings are shipped as recommendations in the Helm chart and docs.

## Rationale

Time partitioning makes retention an O(1) partition drop instead of mass deletes. Native partitioning keeps the minimum profile dependency-free. The ADR-0039 sidecar + crypto-shredding scheme reconciles tamper evidence with the right to erasure.

## Alternatives considered

- **pg_partman / TimescaleDB:** extra extensions; managed Postgres support varies.
- **DELETE-based retention:** bloat and vacuum pressure at scale.
- **Moving audit to object storage by default:** queryability suffers. It's used for archives only.

## Consequences

- Positive: predictable storage; GDPR-aligned defaults; scalable audit.
- Negative: partition management code must be robust. The worker alerts if future partitions are missing (< 3 days ahead).
- Risk: an accidental drop of held data. Mitigations: the hold check is done in the same transaction as the detach, and archives are verified before the drop.

## Fallback / revisit trigger

If a single cell's audit volume exceeds 5 TB hot, move the audit family to a dedicated Postgres instance per cell (same schema). Users see no API change.

## Implementation notes

- P1: audit, outbox, sessions partitioning.
- P3: webhook deliveries.
- P4: provisioning logs, legal hold.
- P5: risk.
- P9: archive tooling for cells.

## Verification

- Retention integration tests with time travel (clock injection).
- Hash-chain verification after erasure.
- A legal-hold test confirms no drop happens.
- A partition pre-creation alert test.
