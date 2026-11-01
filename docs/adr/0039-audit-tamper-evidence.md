# ADR-0039: Audit tamper evidence

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D39, §6.2, §18
- **Related:** ADR-0008, ADR-0011, ADR-0072, ADR-0080, ADR-0095, ADR-0109, ADR-0059

## Context

Audit records are evidence for compliance, incident response and Supervised Support Access transparency. They must be complete (no lost events), tamper-evident (insider or operator modification is detectable), privacy-safe (no secrets) and exportable to SIEMs. They must work Postgres-only.

## Decision

1. **Write path:** audit events are inserted in the **same transaction** as the state change they describe (outbox pattern, ADR-0008). No state change commits without its audit record.
2. **Storage:**
   - Table `audit_events`, append-only, partitioned monthly (ADR-0109).
   - The application DB role has `INSERT`/`SELECT` only; `UPDATE`/`DELETE` are revoked, and a trigger raises on any attempt.
   - Retention purge is done by dropping partitions via the separate `auvora_retention` role.
3. **Hash chain:**
   - Each event has `seq` (per-project monotonic, assigned by a serialized sequence allocator) and `hash = SHA-256(prev_hash || canonical_json(event))`.
   - Canonical JSON follows RFC 8785 (JCS).
   - Chains are per project, plus one tenant/platform chain for control-plane events.
4. **Checkpoints:**
   - Every 10 min (`audit.checkpoint_interval`), the worker signs `{project_id, seq, hash, time}` with the project's `audit_signing` key (ADR-0082).
   - Checkpoints go into `audit_checkpoints` and, when configured, to **WORM object storage** (S3 Object Lock compliance mode, GCS bucket lock, Azure immutable blob; `[audit.worm]`).
   - `auvora audit verify --project <id> [--from --to]` recomputes the chain and validates signatures against checkpoints.
5. **Event schema:** `id, time, seq, hash, tenant_id, project_id, org_id, category (security|admin|activity|system), action, outcome, reason, actor {type, id, act_chain}, subject {type, id}, client_id, session_id, ip?, user_agent?, geo?, trace_id, request_id, target_before_hash?, target_after_hash?, metadata`.
   - IP/UA/geo storage is policy-controlled (`audit.capture_network = true` default).
6. **Never logged:** passwords, OTP values, private keys, bearer/refresh tokens, client secrets, raw recovery codes, Token Vault contents. A schema-level denylist and a test scanner enforce this.
7. **Erasure interplay (canonical scheme):** PII never enters the hashed event body. It is stored in the sidecar table `audit_event_pii` (keyed by event ID), encrypted with the subject's per-principal DEK (ADR-0072). The hash chain covers `HMAC-SHA256(project_audit_pii_key, canonical_pii)`, never plaintext. On purge (ADR-0054) the per-principal DEK is destroyed (crypto-shredding) and the sidecar rows are deleted; the event keeps the opaque `principal_id`, and chain verification still succeeds because the HMAC is retained. This is the single erasure scheme for audit and history; ADR-0039 is authoritative.
8. **Access:**
   - Tenant- and project-scoped via Admin API roles (Auditor role, ADR-0055).
   - Org admins see org-scoped events.
   - Users see their own security activity.
   - Every audit read by the platform role is itself audited.
9. **Streaming/export:**
   - SIEM sinks via outbox workers: syslog/CEF (RFC 5424 over TLS), OTLP logs, Splunk HEC, Datadog, Microsoft Sentinel, Google Chronicle, S3/GCS/Azure files (JSONL) and webhooks.
   - Exports include `seq`/`hash` so external systems can verify continuity.
10. **Retention:** 90 days in the open edition. EE is configurable up to 10 years plus WORM (ADR-0062).

## Rationale

The transactional write gives completeness. Hash chain + signed checkpoints + WORM give tamper evidence without a ledger database. PII separation reconciles immutability with GDPR erasure.

## Alternatives considered

- **Separate ledger DB (QLDB-style)**: extra dependency. Rejected.
- **Async-only audit**: events can be lost. Rejected.
- **Hashing over PII plaintext**: makes erasure break verification. Rejected.

## Consequences

- Positive: verifiable integrity; erasure-compatible; Postgres-only.
- Negative: the per-project sequence allocation serializes audit inserts per project. Mitigated by short critical sections; benchmarked target ≥ 5,000 events/s per project.
- Risks and mitigations: a DB superuser can still rewrite rows, but this is detectable via WORM checkpoints, documented as the trust boundary.

## Fallback / revisit trigger

If per-project audit insert throughput limits login throughput (< 5,000 events/s per project), switch to batched chain computation by the worker (events inserted unchained, chained within ≤ 5 s), with a superseding ADR.

## Implementation notes

P1 transactional audit + chain; P4 checkpoints, WORM, SIEM sinks; P9 compliance packs. Crate `auvora-audit`.

## Verification

Tests: tamper a row → `verify` fails; erase a user → `verify` passes; secret-denylist scanner; DB role permission tests; throughput benchmark.
