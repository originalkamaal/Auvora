# ADR-0006: Minimum deployment profile: binary + PostgreSQL

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §1.1 (U2), §1.2 principle 14, §4 D06, §5.2
- **Related:** ADR-0005, ADR-0007, ADR-0008, ADR-0011, ADR-0042, ADR-0043, ADR-0044, ADR-0073

## Context

The v0.2 stack required up to ten external systems, which contradicted "small deployments remain operationally reasonable." The owner accepted a minimum profile of binary + Postgres "without compromising features." That means every feature must work with only Postgres, and external systems may add only scale or integration.

## Decision

1. **The minimum supported production deployment is `auvora` (1+ replicas) + PostgreSQL 16+.** In this profile, every open and EE feature MUST work.
2. Each optional component has a mandatory embedded or Postgres fallback:

   | Optional component | Purpose when present | Fallback in minimum profile |
   |---|---|---|
   | Valkey | Hot cache, rate limits, replay sets | Postgres UNLOGGED tables + advisory locks + in-process caches (ADR-0007) |
   | NATS JetStream / Kafka | Event fan-out, external consumers | Transactional outbox + embedded workers + webhooks (ADR-0008) |
   | Cloud KMS / HSM / Vault Transit | Key custody | Local encrypted keystore in Postgres, KEK from passphrase/file (ADR-0011) |
   | OpenSearch | Large-scale search | Postgres FTS + pg_trgm (ADR-0042) |
   | ClickHouse | Analytics at scale | Postgres rollups (ADR-0043) |
   | Object storage (S3/GCS/Azure) | WORM audit checkpoints, large exports | Postgres-stored checkpoints + local filesystem export (WORM guarantee then becomes the operator's responsibility; documented) |
   | External OpenFGA/SpiceDB/OPA | Existing authz investments | Embedded ReBAC + Cedar (ADR-0015) |
   | SPIRE | SPIFFE attestation | Built-in CA issuing SPIFFE-compatible SVIDs (ADR-0092) |
   | Reverse proxy / gateway | Edge features | Built-in `edge` role (ADR-0010) |

3. The only required Postgres extensions are ones bundled with standard PostgreSQL (`pgcrypto`, `pg_trgm`, `btree_gin`). No third-party extensions are required. Managed Postgres services (RDS, Cloud SQL, Azure Flexible, Neon, Supabase) must work.
4. **Feature-parity tests:** the full E2E and conformance suites run in CI against the minimum profile. Any feature that fails there is a release blocker.
5. A **dev profile** (`auvora dev`) can start an embedded or ephemeral Postgres automatically, so the developer experience needs zero dependencies.

## Rationale

Operational simplicity is a core USP (U2) and a key difference from Keycloak (JVM + DB + cache cluster) and Ory (multiple services). Postgres is mature enough to serve every role at small-to-mid scale.

## Alternatives considered

- **Require Redis for sessions and rate limits:** an extra stateful service for every user. Rejected.
- **SQLite for the minimum profile:** no HA, weak concurrency, and two storage dialects to maintain. Rejected (embedded Postgres is used for dev instead).
- **"Lite" profile with reduced features:** contradicts the owner's no-compromise directive. Rejected.

## Consequences

- Positive: easiest-in-class install; one backup target; air-gap friendly.
- Negative: higher Postgres load; Postgres-specific code paths must be written and tested for every accelerator.
- Risks: Postgres becoming a bottleneck at large scale. Mitigation: accelerators (ADR-0007/0008), partitioning (ADR-0109), and cells (ADR-0019).

## Fallback / revisit trigger

None for the principle. Individual fallbacks are revisited in their own ADRs.

## Implementation notes

- Config detection: if `[cache.valkey] url` is unset, the Postgres-backed implementation of the `EphemeralStore` trait is used. Same pattern for `[events.bus]`, `[keys.provider]` and `[search]`.
- Compose file `deploy/compose/minimal.yaml` (auvora + postgres) is the canonical quickstart.

## Verification

- A CI matrix job `profile=minimal` (auvora + postgres only, outbound network disabled except to the test fixtures) runs the full E2E, OIDF and abuse suites.
- A startup log line lists the active implementations per subsystem (e.g. `ephemeral_store=postgres`).
