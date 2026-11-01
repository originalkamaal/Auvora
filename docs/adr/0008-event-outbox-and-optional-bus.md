# ADR-0008: Transactional outbox and optional event bus

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D08, §6.2, §5.4
- **Related:** ADR-0006, ADR-0039, ADR-0080, ADR-0095, ADR-0096, ADR-0109

## Context

Auvora emits domain and audit events for webhooks, SIEM, SSF, analytics, metering, the SCIM client, cache invalidation and customer integrations. Events must never be lost or published for rolled-back transactions. v0.2 offered "Kafka or NATS," but both are heavy requirements for small deployments.

## Decision

1. **Transactional outbox:** every state change writes its domain event(s) into `outbox_events` in the **same Postgres transaction** as the state change. Audit records are written in the same transaction too (ADR-0039).
2. **Embedded dispatcher** (`worker` role):
   - Workers claim batches with `SELECT … FOR UPDATE SKIP LOCKED` ordered by `(partition_key, seq)`.
   - Batch size defaults to 500; poll interval to 200 ms, plus a `LISTEN outbox_new` wake-up.
   - Workers dispatch to registered sinks and mark rows delivered per sink.
3. **Delivery semantics:** at-least-once per sink. Ordering is guaranteed per `partition_key` (default: `project_id` + aggregate ID). Consumers deduplicate by event ID (CloudEvents `id`, ADR-0080).
4. **Built-in sinks (all embedded):**
   - webhooks (ADR-0080)
   - SSF transmitter (ADR-0096)
   - SIEM (syslog/CEF, OTLP logs, Splunk HEC, Datadog, Sentinel, Chronicle)
   - metering/analytics rollups (ADR-0043)
   - cache-invalidation NOTIFY
   - SCIM client
   - internal job triggers
5. **Optional bus:** **NATS JetStream** is the one supported internal bus for multi-cell and high-fan-out deployments. When configured, the dispatcher publishes to subjects `auvora.<cell>.<project>.<event_type>` and the other sinks consume from JetStream.
6. **External streaming sinks** (export only, never required): Kafka, AWS Kinesis, EventBridge, GCP Pub/Sub, Azure Event Hubs.
7. **Retention:** delivered outbox rows are pruned after 7 days (`[events] outbox_retention = "7d"`). Undeliverable rows go to `outbox_dead_letter` after the sink's retry policy is exhausted. They can be replayed via the Admin API and CLI.
8. Event payloads exclude secrets (no passwords, OTPs, tokens or private keys). PII is limited to the event's schema (ADR-0072).

## Rationale

The outbox gives exactly-once *recording* and at-least-once *delivery* using only Postgres. That satisfies ADR-0006. NATS is lighter to operate than Kafka for an internal bus. Kafka remains supported where enterprises already have it, as a sink.

## Alternatives considered

- **Kafka as the core bus:** heavy, JVM/KRaft operations. Rejected as a requirement; kept as a sink.
- **Logical decoding / CDC (Debezium):** couples events to table layout and needs extra infrastructure. Rejected.
- **Direct publish after commit:** loses events on crash. Rejected.

## Consequences

- Positive: no lost events; one dependency; bus-agnostic consumers.
- Negative: outbox write amplification; Postgres polling cost.
- Risks: outbox backlog under bursts. Mitigation: partitioned outbox table (ADR-0109), dispatcher autoscaling via the `worker` role replica count, and a lag metric with alerts (`auvora_outbox_lag_seconds` > 30 s).

## Fallback / revisit trigger

If outbox lag p99 exceeds 30 s at the target throughput (ADR-0105) despite worker scaling, the docs will mandate NATS for that deployment tier. No feature changes.

## Implementation notes

- Crate `auvora-events`. Config: `[events.bus] kind = "none" | "nats"` (default `none`), `[events.bus.nats] url`, `[[events.sinks]]` blocks.
- Event schema registry lives in `crates/auvora-events/schemas/*.json` (ADR-0080).
- P1: outbox, dispatcher, NOTIFY sink, audit. P3: webhooks. P4: SIEM. P5: SSF. P9: NATS multi-cell.

## Verification

- Property test: random transaction commit/rollback sequences. Every committed change has exactly one outbox row; no rolled-back change has any.
- Crash test: kill the worker mid-batch. All events are delivered at least once and per-partition order holds.
