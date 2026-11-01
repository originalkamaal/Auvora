# ADR-0009: Control-plane → data-plane config propagation

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D09, §5.1, §5.4
- **Related:** ADR-0005, ADR-0008, ADR-0019, ADR-0033, ADR-0070, ADR-0073

## Context

The data plane must evaluate project config (clients, connections, policies, keys, themes, flow policies, Cedar/ReBAC models) on every request at low latency. It also must never call the control plane synchronously. Admin changes such as "client disabled" or "connection removed" must take effect within a bounded time. v0.2 did not define this mechanism.

## Decision

1. **Postgres is the single source of config.** Every config write increments `project_config_version` (a per-project monotonic `bigint`) in the same transaction and emits `NOTIFY auvora_config, '<project_id>:<version>'`.
2. **Data-plane cache:** each `data`/`edge` instance holds an in-memory, per-project **immutable config snapshot**, a compiled form that includes parsed CEL programs, Cedar policy sets, redirect URI matchers and JWKS.
   - Snapshots load lazily on first request and are swapped atomically (`ArcSwap`).
   - The cache is bounded (`[config_cache] max_projects = 50_000`, LRU eviction of inactive projects).
3. **Invalidation:**
   - **Push:** a dedicated LISTEN connection per instance receives notifications and rebuilds the affected snapshot asynchronously.
   - **Pull safety net:** every 2 s, the instance batch-reads `project_config_version` for cached projects and reloads any that changed. This covers missed notifications and connection drops.
   - **On LISTEN disconnect:** the instance marks all snapshots *suspect*, does a full version check, then resumes.
4. **Propagation SLO:** ≤ 5 s p99 from commit to every instance serving the new version (D09/D33). Metric `auvora_config_propagation_seconds`, measured by comparing commit timestamp to snapshot activation.
5. **Security-critical invalidations** (client disabled, key compromised, connection disabled, user/org suspended) are additionally checked at the point of use where cheap. For example, the token endpoint re-reads client `status` from Postgres or the cache on refresh-token redemption. Their effect doesn't depend only on snapshot propagation.
6. **Cross-cell / multi-region (P9):** the control plane publishes **signed config snapshots** (Ed25519/ECDSA, signed by a platform key) to cells via the bus (ADR-0008). Cells verify the signature and version before activation. Same SLO within a cell, ≤ 30 s cross-cell.
7. **Build failures:** if a new version fails to compile (e.g. an invalid CEL program), the data plane keeps the last good snapshot, raises `config_build_failed`, and the control plane surfaces an error. Validation at write time (control plane) makes this rare.

## Rationale

The shared DB + NOTIFY + version polling design needs no extra infrastructure, gives bounded staleness, and keeps the data plane independent of control-plane availability. Compiled snapshots avoid per-request parsing.

## Alternatives considered

- **Read config from DB on every request:** DB load and latency. Rejected.
- **Push snapshots via a message bus only:** adds a dependency in the minimum profile. Rejected for single-cell use; used cross-cell.
- **TTL-only caches:** unbounded staleness on short TTLs, or DB load on very short ones. Rejected.

## Consequences

- Positive: fast reads; bounded staleness; no data→control RPC.
- Negative: memory per instance scales with active projects.
- Risks: snapshot rebuild storms after bulk changes. Mitigation: debounce of 100 ms per project and a rebuild concurrency limit (`[config_cache] rebuild_concurrency = 8`).

## Fallback / revisit trigger

Propagation SLO breach (> 5 s p99) sustained for 7 days in benchmark or production. A superseding ADR would then adopt signed snapshots via NATS for single-cell scale profiles too.

## Implementation notes

- Crate `auvora-config` (snapshot model, compiler, cache, listener).
- P1 deliverable. Cross-cell signing in P9.
- Config history and rollback rely on entity history (ADR-0071).

## Verification

- Integration test: write config, then assert that all N instances serve the new version within 5 s under load.
- Chaos test: drop the LISTEN connection and assert convergence via the pull loop.
- The benchmark suite records the propagation p99 per release.
