# ADR-0071: Entity history (identity time travel)

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §2.2 (item 4), §6.1, §2.1 (Zitadel row)
- **Related:** ADR-0039, ADR-0070, ADR-0072, ADR-0109, ADR-0015

## Context

Zitadel's event-sourced design gives full history. Auditors and incident responders ask "what did this user/role/client look like, and who had what access, at time T?". Full event sourcing complicates queries and migrations. Auvora wants the benefits without making event sourcing the system of record.

## Decision

1. **State-based storage remains authoritative.** Every mutable identity or config table `X` has a companion `X_history` table, written in the **same transaction** by repository code (not triggers, so that actor context is mandatory).
2. **History row:** `entity_id`, `version` (monotonic per entity), `valid_from`, `valid_to`, `op` (`create|update|delete|restore`), full row snapshot (JSONB, secrets excluded or replaced by hash/fingerprint), `actor` (principal + `act` chain), `txid`, `request_id`, `reason` (optional).
3. **Covered entities:** principals, identifiers (hashed values for lookup), credential *metadata* (never secret material), memberships, groups, roles, role assignments, ReBAC tuples (as tuple history), apps, clients, resource servers, connections, policies (flow/access/Cedar/ReBAC models), themes, templates, hooks, keys (metadata), org settings, project settings.
4. **APIs:**
   - `GET /v1/admin/{entity}/{id}/history`
   - `GET /v1/admin/{entity}/{id}?as_of=<RFC3339>`
   - `GET /v1/admin/diff?from=…&to=…`
   - `POST …/restore {version}`
   - **Authz as-of:** `POST /v1/authz/evaluation` with `as_of` evaluates RBAC/ReBAC/Cedar using historical state (P6).
5. **Retention:**
   - Config history is kept indefinitely by default.
   - Principal history follows identity retention (ADR-0054): on hard delete, PII fields in history are purged (crypto-shredded via per-principal DEK, ADR-0072), and the structure/pseudonymous ID is kept.
6. **Optimistic concurrency** uses `version` (ETag in the API, ADR-0020).
7. Domain events (ADR-0080) are emitted from the same transaction via the outbox. History is *not* rebuilt from events.

## Rationale

This captures roughly 90% of event-sourcing value (audit, diffs, as-of queries, rollback) while keeping simple relational queries and migrations.

## Alternatives considered

- **Full event sourcing** (Zitadel): complex projections and migrations.
- **Postgres temporal triggers**: lose actor context; hidden behaviour.
- **Audit log only**: can't reconstruct state.

## Consequences

- Positive: time travel, rollback, forensic answers.
- Negative: write amplification (~2×) and storage growth. Mitigation: partitioning (ADR-0109), JSONB compression (TOAST lz4).
- Risks: secrets copied into history. Mitigation: a `#[history(exclude)]` derive attribute plus a test.

## Fallback / revisit trigger

If history write overhead exceeds 15% of p99 latency on write paths, move history writes to the outbox-driven async writer for high-volume entities (sessions are already excluded).

## Implementation notes

- Exclusions: sessions, flows, tokens, rate-limit counters (high churn; covered by audit).
- `auvora-store` provides a `Versioned<T>` repository trait.
- P1 for core entities; P6 for authz as-of.

## Verification

- A schema test: every table tagged `versioned` has a `_history` twin.
- Property test: replaying history versions reproduces the current row.
- A secret-exclusion test.
