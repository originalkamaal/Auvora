# ADR-0073: State stores, consistency and fail modes

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §5.4, §4 D06/D07/D09/D33, v0.2 §18.2
- **Related:** ADR-0006, ADR-0007, ADR-0008, ADR-0009, ADR-0033, ADR-0074, ADR-0015

## Context

Each kind of state needs an explicit store, consistency guarantee and failure behaviour. Security-critical state must never fail open silently. The Postgres-only profile must provide every feature.

## Decision

`StateStore` traits in `auvora-store` have two backends: **Postgres** (always available) and **Valkey** (optional accelerator).

| State | Authoritative store | With Valkey | Consistency | On store failure |
|---|---|---|---|---|
| Identities, credentials, config, grants | Postgres | Read-through cache (config/keys only) | Strong on primary | Fail closed (503) |
| Config in data plane | In-memory, versioned | — | ≤ 5 s p99 (ADR-0009) | Serve last-known-good up to `config.max_staleness` (default 5 min), then fail closed for writes to tokens referencing changed objects |
| Signing keys (public) / JWKS | Postgres + memory | — | ≤ 5 s | Serve cached; never stop verifying |
| Sessions | Postgres | Read-through cache, write-through | Revocation ≤ 5 s p99 (ADR-0033) | Fail closed for session validation older than cache TTL (`sessions.cache_ttl` default 5 s) |
| Flow state | Postgres (UNLOGGED option) | Primary when present | Strong per flow | Fail closed (user restarts flow) |
| Auth codes, PAR request_uris, device codes, CIBA auth_req_ids | Postgres UNLOGGED / Valkey | Valkey primary | Single-use, atomic consume | **Fail closed** |
| Replay caches (nonce, DPoP `jti`, client assertion `jti`, SAML assertion IDs, magic link/OTP use) | Postgres UNLOGGED / Valkey | Valkey primary | Atomic insert-if-absent within the validity window | **Fail closed** |
| Refresh token families | Postgres | — | Strong | Fail closed |
| Rate-limit counters | Postgres UNLOGGED / Valkey | Valkey primary | Approximate | Credential endpoints: **fail closed** (429). Read-only/public endpoints (JWKS, discovery): **fail open** with a local in-process limiter |
| ReBAC tuples | Postgres | Check cache | Consistency tokens (at-least-as-fresh) | Fail closed (deny) |
| Audit + outbox | Postgres (same tx) | — | Committed with the change | The business transaction fails if audit can't be written |
| Webhook delivery state | Postgres | — | At-least-once | Retry/backoff (ADR-0080) |

**Rules:**
1. UNLOGGED tables are used only for state whose loss on crash is safe, meaning loss causes denial, not acceptance. After a crash, replay caches are rebuilt empty, and every artifact issued before the crash is invalidated by an **epoch** counter stored in a logged table (`state_epoch`). Artifacts embed the epoch, and a mismatch means reject.
2. Valkey usage requires `maxmemory-policy noeviction` for the replay/code keyspaces. Auvora checks this at boot and refuses to start otherwise (`valkey.allow_eviction=false`).
3. Valkey failover: on connection loss, single-use and replay operations switch to Postgres automatically only if `state.fallback_to_postgres=true` (default true). Otherwise they fail closed.
4. No sticky sessions are required for correctness.
5. Every fail-open path is enumerated in this table. Adding one requires a superseding ADR.

## Rationale

This makes the failure semantics reviewable and testable, and satisfies v0.2's "replay protection must not silently fail open".

## Alternatives considered

- **Valkey required**: violates ADR-0006.
- **Everything in logged Postgres tables**: WAL pressure for high-churn ephemeral state. UNLOGGED + epoch gives safety at lower cost.

## Consequences

- Positive: explicit guarantees; the Postgres-only profile is complete.
- Negative: two backends to maintain. Mitigation: a shared trait conformance test suite.

## Fallback / revisit trigger

If Postgres UNLOGGED throughput limits single-use ops below ADR-0105 targets in the Small profile, recommend Valkey for that profile. Features stay intact.

## Implementation notes

- `auvora-store::state` traits: `SingleUseStore`, `ReplayCache`, `CounterStore`, `SessionStore`, `FlowStore`.
- P1.

## Verification

- A shared conformance test suite runs against both backends.
- Chaos tests (toxiproxy): kill Valkey and Postgres mid-flow, and assert no acceptance of replayed artifacts.
- A crash-restart test for epoch invalidation.
