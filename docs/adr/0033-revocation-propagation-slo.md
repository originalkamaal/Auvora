# ADR-0033: Revocation propagation SLO

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D33, D09, §5.4, §21, §25
- **Related:** ADR-0007, ADR-0009, ADR-0019, ADR-0029, ADR-0030, ADR-0032, ADR-0073, ADR-0096

## Context

v0.2 listed "maximum tolerated revocation propagation delay" as undefined. Security guarantees such as "user suspended", "client disabled" or "session revoked" need a measurable bound in each token model.

## Decision

1. **Bounds (in-region, p99):**

   | Artifact | Bound |
   |---|---|
   | SSO session revocation (cookie/session checks, refresh) | **≤ 5 s** |
   | Introspection result for opaque tokens or revoked JWT `jti`/session | **≤ 5 s** |
   | Config changes affecting authorization (client disabled, user suspended, key revoked) on the data plane | **≤ 5 s** (ADR-0009) |
   | Self-contained JWT access tokens at resource servers that validate locally | **≤ token lifetime** (default 10 min, ADR-0029), or **≤ 5 s** for resource servers subscribed to SSF/CAEP or using introspection |
   | Cross-cell (P9) for global objects | ≤ 30 s |

2. **Mechanism:**
   - The write commits to Postgres.
   - `NOTIFY auvora_invalidate` carries `{project_id, kind, id, version}`.
   - Every data-plane node evicts its in-process cache entry and deletes the Valkey key (if present).
   - Cache entries carry a version and a hard max TTL of 5 s for revocable objects, so a missed notification still converges within the bound.
3. **Hot revocation list:** a per-project in-memory set of revoked session IDs and token `jti`s, valid until their natural expiry, populated by NOTIFY and bootstrapped from Postgres at start. It is used by introspection, the userinfo endpoint and refresh.
4. **Push to relying parties:** revocation events are emitted as SSF/CAEP `session-revoked` / `credential-change` and RISC `account-disabled` events (ADR-0096), and as webhooks (ADR-0080). Back-channel logout tokens are sent for session revocations.
5. **Fail behaviour:** if a node loses its LISTEN connection, it switches to **no caching** for revocable objects (direct DB reads) until it reconnects and replays from the `invalidation_log` table (retained 10 min). Fail closed: never serve stale "valid" beyond 5 s.
6. **Measurement:** a synthetic probe (`auvora-worker` job `revocation_probe`) revokes test sessions every minute and measures time to rejection on all nodes. It exports `auvora_revocation_propagation_seconds` (histogram).

## Rationale

Short JWTs plus a push channel give a revocation story that is scalable and fast for those who need it. NOTIFY gives a sub-second typical path with no extra dependency, and the TTL cap guarantees the bound even on missed messages.

## Alternatives considered

- **Polling only**: higher DB load and slower. Rejected.
- **Mandatory introspection for all APIs**: kills scalability. Rejected as default; available per resource server.
- **Valkey pub/sub required**: violates ADR-0006. Rejected; it's an optional accelerator.

## Consequences

- Positive: measurable, testable revocation guarantees.
- Negative: the 5 s TTL cap reduces cache hit ratio for revocable objects. This is acceptable at the target load (ADR-0105).
- Risks and mitigations: a NOTIFY payload limit (8,000 bytes) is handled with batched IDs and a fallback "flush project" message.

## Fallback / revisit trigger

If the measured p99 exceeds 5 s for 3 consecutive days in Cloud, or Postgres NOTIFY throughput becomes a bottleneck (> 5k notifications/s sustained), make the NATS invalidation channel the preferred path for scale profiles (still optional).

## Implementation notes

P1 mechanism and probe; P5 SSF push. Crates `auvora-config`, `auvora-sessions`, `auvora-events`.

## Verification

Integration test with 3 nodes asserting rejection ≤ 5 s after revoke; chaos test dropping the LISTEN connection; probe SLO alert in Cloud.
