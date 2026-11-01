# ADR-0007: Optional Valkey cache with Postgres fallback

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D07, §5.4, v0.2 §18.2
- **Related:** ADR-0006, ADR-0032, ADR-0033, ADR-0073, ADR-0074

## Context

Short-lived state includes authorization codes, PAR request URIs, nonces, DPoP `jti`s, assertion replay sets, rate-limit counters, flow state and session read caches. A fast store helps at scale, but ADR-0006 forbids making it mandatory. Replay protection must fail closed. v0.2 left "Redis or Valkey" undecided.

## Decision

1. **Valkey** (BSD-3) is the supported cache/ephemeral store. Redis OSS ≥ 7.2-compatible servers are accepted on a best-effort basis. Redis versions under SSPL/RSAL licenses aren't tested or recommended.
2. All ephemeral state goes through a trait in `auvora-core`:
   - `EphemeralStore`: `put_if_absent(key, ttl)`, `take(key)` (atomic get-and-delete), `incr_window(key, window)`, `get`/`set` with TTL.
   - Implementations: `ValkeyStore` and `PostgresStore`.
3. **Postgres fallback:**
   - UNLOGGED tables `eph_kv(key text pk, value bytea, expires_at timestamptz)` and `eph_counter(key, window_start, count)`;
   - atomic operations via `INSERT … ON CONFLICT DO NOTHING RETURNING` and `DELETE … RETURNING`;
   - a reaper job deletes expired rows every 30 s in batches;
   - advisory locks only for multi-key operations.
   UNLOGGED data is lost on crash, which is acceptable: it only invalidates in-flight codes and flows. Users retry, and nothing fails open.
4. **Single-use security state** (codes, PAR URIs, nonces, `jti`, SAML assertion IDs, OTP attempts) MUST use `put_if_absent` / `take` semantics. If the store is unavailable, the operation **fails closed** (HTTP 503 with RFC 9457 body), never open.
5. **Read caches** (session lookups, config) are optional accelerators. On cache failure the request reads Postgres. Postgres stays authoritative (ADR-0032).
6. **Valkey topology:** standalone, Sentinel or Cluster. Keys are hash-tagged by project, `{prj_…}:…`, for cluster locality. TLS and AUTH/ACL are required outside dev.
7. **Eviction policy:** Valkey MUST be configured with `maxmemory-policy noeviction` for the ephemeral namespace. Auvora checks this at startup and refuses to use an instance with an evicting policy unless `[cache.valkey] allow_evicting_policy = true` is set, which also forces security state onto the Postgres store.

## Rationale

The trait abstraction gives speed at scale with zero required dependencies. Fail-closed semantics prevent replay windows during outages. The `noeviction` check prevents silent loss of replay state under memory pressure.

## Alternatives considered

- **Redis required:** violates ADR-0006. Rejected.
- **Memcached:** no atomic take or rich structures. Rejected.
- **In-process only:** doesn't work across replicas. Rejected (used only as an L1 cache for config).

## Consequences

- Positive: works everywhere; scales with Valkey; clear failure semantics.
- Negative: two implementations to test; Postgres write load from ephemeral data in the minimum profile.
- Risks: UNLOGGED table bloat. Mitigation: aggressive reaping and autovacuum tuning documented in the Helm/Compose defaults.

## Fallback / revisit trigger

If Postgres ephemeral load exceeds 30% of DB CPU at the documented small-profile capacity (ADR-0105), the docs will recommend Valkey at that tier. Features remain identical either way.

## Implementation notes

- Config: `[cache] backend = "auto" | "postgres" | "valkey"` (default `auto` = Valkey if `[cache.valkey] url` is set). Env `AUVORA_CACHE__VALKEY__URL`.
- Crate: the trait is in `auvora-core`; implementations live in `auvora-store` (Postgres) and `auvora-store::valkey`.
- P1 deliverable.

## Verification

- Contract tests run the same suite against both implementations.
- Chaos tests (toxiproxy) kill Valkey mid-flow and assert that security operations fail closed and read paths degrade to Postgres.
