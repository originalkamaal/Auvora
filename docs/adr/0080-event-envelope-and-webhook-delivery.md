# ADR-0080: Event envelope and webhook delivery

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §6.2, §14 (tier 1 webhooks), §24 (CloudEvents), §29 (spec-first event catalog)
- **Related:** ADR-0008, ADR-0038, ADR-0039, ADR-0096, ADR-0103, ADR-0109

## Context

Events feed webhooks, buses (NATS/Kafka), SIEM and SSF. Without one envelope, a versioned catalog and well-defined delivery semantics, integrations break on every release. Competitors' webhooks are often unsigned, unordered or not replayable.

## Decision

1. **Envelope: CloudEvents 1.0** (JSON structured mode). Required attributes:
   - `id` (UUIDv7), `source` (`auvora://{cell}/{project_id}`), `type` (`auvora.<domain>.<entity>.<verb>.v<N>`), `time`, `subject` (external ID), `datacontenttype=application/json`.
   - Extensions: `tenantid`, `projectid`, `orgid`, `actorid`, `actorchain`, `traceparent`, `sequence` (per-partition monotonic).
2. **Catalog:** every event type has a JSON Schema in `crates/auvora-events/catalog/`, published in the docs and in OpenAPI webhooks. Versioning is additive within `vN`. A breaking change creates `vN+1`, and both versions are emitted for one minor-release deprecation window (minimum 6 months).
3. **No secrets in events.** No passwords, OTPs, tokens or keys appear. PII appears only if the subscription opts in (`include_pii = true`), and that opt-in is audited.
4. **Source of truth:** the transactional outbox (ADR-0008). An event exists only if its transaction commits.
5. **Webhook delivery semantics:**
   - At-least-once. Consumers dedupe by `id`.
   - **Ordering:** per subscription and per `subject` (ordering key), achieved by serial delivery per key. Different keys are delivered in parallel.
   - Retries use exponential backoff with jitter: 10 s, 30 s, 2 m, 10 m, 30 m, 1 h, 3 h, 6 h, then every 12 h up to 72 h total. After that the event goes to the **dead-letter queue**, which is retained for 30 days.
   - **Auto-disable** after 72 h of continuous failure, with a notification to project admins.
   - Timeout is 10 s. Any 2xx response is success. A 410 response disables the subscription.
   - **Replay:** by event ID, time range or DLQ, via API, CLI and console.
6. **Signing:**
   - Default: HMAC-SHA256 over `webhook-id.webhook-timestamp.body` (Standard Webhooks spec headers `webhook-id`, `webhook-timestamp`, `webhook-signature`).
   - Optional: a JWS detached signature with the project's webhook-signing key (ADR-0082), verifiable through JWKS.
   - Timestamp tolerance is 5 min. Secrets can be rotated with two active secrets during overlap.
7. **Egress** goes through the SSRF-guarded egress layer (ADR-0103). Endpoints must be HTTPS in prod projects.
8. **Filtering:** subscriptions select event types (with wildcards), plus an optional CEL filter (ADR-0016) on envelope and data.
9. **Bus and sink delivery** (NATS, Kafka, Kinesis, Pub/Sub, EventBridge, SIEM) uses the same envelope. SSF uses SET format instead (ADR-0096).
10. **Limits:** 100 subscriptions per project (configurable); payload ≤ 256 KiB (larger payloads become a reference plus a fetch URL).

## Rationale

CloudEvents and Standard Webhooks are widely supported, and there's no reason to invent a format. Ordering per subject avoids the "updated before created" class of consumer bugs without serializing all traffic.

## Alternatives considered

- **Custom envelope.** Rejected: interop and tooling costs.
- **Global ordering.** Rejected: throughput.
- **Exactly-once delivery.** Rejected: impossible over HTTP. Idempotency by `id` is the contract.

## Consequences

- Positive: reliable, replayable, verifiable integrations.
- Negative: per-key serialization can delay events on a hot subject; this is acceptable.
- Risks: DLQ growth. Mitigated by retention and auto-disable.

## Fallback / revisit trigger

If webhook delivery lag exceeds 30 s p99 at target load, introduce a NATS-backed dispatcher in the scale profile. The semantics stay the same.

## Implementation notes

- Crates: `auvora-events` (outbox reader, dispatcher, signer, catalog), `auvora-api` (subscriptions, replay).
- Ships in P3. The catalog starts in P1 for audit/outbox.

## Verification

- A schema-compat CI check (no breaking change without a version bump).
- Chaos tests with toxiproxy: retries, ordering per key, DLQ, replay.
- Signature test vectors are published.
