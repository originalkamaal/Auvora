# ADR-0042: Search: Postgres FTS, optional OpenSearch

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D42, §2.3 (v0.2 review gaps), §13.5
- **Related:** ADR-0006, ADR-0017, ADR-0072, ADR-0109, ADR-0043

## Context

Admins need to search users, organizations, clients, audit events and provisioning logs by partial identifiers, names, attributes and free text. Projects can reach 100 M users (ADR-0105). The minimum profile is binary + Postgres (ADR-0006), so search must work without an external engine, while very large deployments may want richer search.

## Decision

1. **Default engine: PostgreSQL**, built into `auvora-store`:
   - `pg_trgm` GIN indexes on normalized identifiers (email, username, phone, display name) for substring and fuzzy match;
   - `tsvector` generated columns + GIN for free-text over profile display fields and audit `reason`/`action`;
   - B-tree/expression indexes for exact and prefix lookup on external IDs, `sub` and upstream subjects;
   - JSONB GIN (`jsonb_path_ops`) on `public_metadata` for equality filters.
2. **Query API:** one search syntax for Admin API, console and CLI, `GET /v1/projects/{id}/users?q=…&filter=…`, where `filter` uses a SCIM-style filter grammar (shared parser with `auvora-scim`, fuzzed). All searches are tenant/project-scoped through RLS (ADR-0017).
3. **Limits:** results are cursor-paginated (max page 200). Queries time out at `search.statement_timeout_ms = 2000`. A substring query must have ≥ 3 characters to use trigram.
4. **Optional adapter:** `auvora-search-opensearch` (feature flag; OpenSearch 2.x/3.x API), enabled by `search.backend = "opensearch"`. It's fed from the outbox (ADR-0008). Postgres stays authoritative. On adapter unavailability, the backend falls back to Postgres automatically and logs a degraded status.
5. **PII:** the indexes contain only fields tagged searchable in the PII catalog (ADR-0072). `private_metadata` and secrets are never indexed. OpenSearch documents inherit erasure: user deletion triggers delete-by-ID within the D33 propagation bound.
6. Audit search is time-bounded by default (last 30 days unless a range is given) to use partition pruning (ADR-0109).

## Rationale

Postgres trigram + FTS handles tens of millions of rows with proper indexes and keeps the "all features in the minimum profile" promise. OpenSearch is only an accelerator.

## Alternatives considered

- **OpenSearch/Elasticsearch required**: rejected (violates ADR-0006).
- **Meilisearch/Typesense**: rejected because they're weaker for audit-scale data and add a second optional engine.
- **Tantivy embedded index**: rejected because it duplicates state per replica and complicates HA; may be revisited.

## Consequences

- Positive: zero extra infrastructure; consistent with RLS.
- Negative: index write amplification on user tables; trigram index size (~2–3× column size).
- Risk: slow queries at extreme scale. Mitigated by timeouts, the time-bound audit default and the optional adapter.

## Fallback / revisit trigger

If p95 user search > 500 ms at 10 M users per project on reference hardware (ADR-0105) after index tuning, OpenSearch becomes the recommended backend for the Scale profile (still optional).

## Implementation notes

P1 (users, clients), P3 (console search UI), P4 (orgs, audit search), P9 (OpenSearch adapter GA). Config: `search.backend`, `search.statement_timeout_ms`, `search.opensearch.url`.

## Verification

Bench suite query set at 1 M and 10 M users; RLS tests assert search never crosses projects; fuzzing of the filter parser.
