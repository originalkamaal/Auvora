# ADR-0019: Cell architecture for multi-region

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D19, §5.3, §25, §27 (P0 design, P9 build)
- **Related:** ADR-0005, ADR-0008, ADR-0009, ADR-0017, ADR-0018, ADR-0063, ADR-0090, ADR-0105

## Context

Auvora Cloud and large self-hosted operators need multiple regions, data residency, blast-radius containment and regional failover. Active-active multi-master writes across regions would require solving identifier uniqueness, revocation propagation and conflict resolution for security-critical data. v0.2 warned against promising this. The decisions that make multi-region possible (IDs, routing, tenancy) must be made now, even though the build happens in P9.

## Decision

1. **Cell** = a complete Auvora deployment (all roles, its own PostgreSQL HA cluster, optional Valkey/NATS, its own KeyProvider config) in one region, spanning ≥ 3 availability zones.
2. **Home cell:** every Tenant has a default home cell. Every **Project** has an immutable `home_cell`, chosen at creation (defaulting to the tenant's) and changeable only via the **cell migration tool** (P9: online copy + cutover with a ≤ 60 s write freeze).
3. **Data residency:** all identity data, PII, audit and keys of a project stay in its home cell. Only the following leave the cell:
   - aggregated, non-PII metrics;
   - routing directory entries;
   - billing usage counters;
   - signed config snapshots for the global console views (metadata only).
4. **Global router:** a minimal stateless service (or DNS-based routing) that maps custom domain / `{project}.{base}` host / issuer → cell endpoint.
   - It holds a replicated directory (`host → cell`), cached with a TTL of 60 s.
   - It is purely a forwarder or redirector, with no auth logic.
   - In single-cell deployments the router is unnecessary and is disabled.
5. **Global control:** the tenant directory, billing, cross-cell admin aggregation and the cell registry. It has no identity data and is not on the authentication path. If it is unavailable, logins keep working.
6. **No active-active writes.** Within a cell, Postgres has a single primary. Cross-region DR is an asynchronous standby cell in a paired region:
   - RPO ≤ 1 min, RTO ≤ 1 h (ADR-0105);
   - failover is an operator-initiated or automated runbook, with DNS/router repointing;
   - signing keys are available in the DR region through KeyProvider replication (multi-region KMS keys, or a wrapped key backup).
7. **Issuer stability:** `iss` URLs don't encode the cell, so cell migration and DR are transparent to relying parties.
8. **Cross-cell config** uses signed snapshots (ADR-0009). Cross-cell events use NATS (ADR-0008).
9. **Self-hosted multi-region** uses the same model (EE).

## Rationale

Cells give residency, blast-radius isolation and linear scaling without distributed-consensus risk on security data. This pattern is proven by large SaaS identity providers. Deciding IDs and routing now (ADR-0018) avoids a later re-architecture.

## Alternatives considered

- **Global active-active database** (CockroachDB/Spanner-like): new dependency, latency, and hard uniqueness/revocation semantics. Rejected.
- **Single global region:** fails residency and resilience. Rejected.
- **Read replicas in other regions serving the data plane:** stale-read risk for revocation. Rejected, except for explicitly read-only, cacheable endpoints (JWKS, discovery) served from the edge/CDN.

## Consequences

- Positive: residency by design; predictable failure domains; simple consistency.
- Negative: a project can't be served from two regions simultaneously for writes. Global users of one project see latency from the home region (mitigated by CDN for static and metadata endpoints).
- Risks: cell migration complexity. Mitigation: built and tested in P9 with drills.

## Fallback / revisit trigger

If a design-partner requirement for multi-region *write* availability within one project is validated (≥ 3 paying customers), a superseding ADR evaluates per-project active/active with conflict-free semantics for a defined subset of entities.

## Implementation notes

- P0: cell-aware routing design doc (`docs/specs/cells.md`) and a `home_cell` column on tenants/projects (value `default` in single-cell installs).
- P9: router, global control, migration tool, DR automation.
- Config: `[cell] id = "eu-central-1a"`, `[cell] region`, `[router] directory_url`.

## Verification

- P9 exit: regional failover drill within RTO/RPO; chaos suite (cell isolation, router outage, global-control outage while logins continue).
- A residency test asserts that no PII fields appear in cross-cell payloads (schema PII tags, ADR-0072).
