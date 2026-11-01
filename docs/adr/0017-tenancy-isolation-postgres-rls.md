# ADR-0017: Tenancy isolation with Postgres RLS

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D17, §6.1, §1.2 principle 17, §26
- **Related:** ADR-0018, ADR-0019, ADR-0025, ADR-0026, ADR-0055, ADR-0109

## Context

Auvora is multi-tenant at two levels: Tenant → Project (ADR-0026). Cross-tenant data access is the most damaging possible bug. Projects must be cheap (≥ 100,000 per cell, principle 17), which rules out heavyweight per-tenant objects. Regulated customers may require physical separation.

## Decision

1. **Shared schema, row-level tenancy.** Every tenant-owned table has `tenant_id uuid NOT NULL` and `project_id uuid NOT NULL` (except tenant-level tables, which have only `tenant_id`). Composite primary keys or indexes lead with `project_id`.
2. **Postgres Row-Level Security is mandatory** on every tenant-owned table:
   - `ALTER TABLE … ENABLE ROW LEVEL SECURITY; ALTER TABLE … FORCE ROW LEVEL SECURITY;`
   - Policy: `USING (project_id = current_setting('auvora.project_id')::uuid)` with matching `WITH CHECK`. Tenant-level tables use `auvora.tenant_id`.
3. **Database roles:**
   - `auvora_owner`: owns the schema; used only by migrations.
   - `auvora_app`: the normal runtime role, subject to RLS, no `BYPASSRLS`.
   - `auvora_platform`: `BYPASSRLS`, used only by explicitly marked cross-tenant operations (platform admin, metering rollups, retention jobs). Every use is audited with the reason, and the code path is restricted to `auvora-store::platform` behind a type-level capability (`PlatformScope` token, constructible only in specific modules).
4. **Transaction context:** every DB transaction starts with `SET LOCAL auvora.project_id = …` (and `auvora.tenant_id`), done by `auvora-store`'s `TenantTx` type. Repository APIs accept only `TenantTx`, never a raw pool connection. If the setting is missing, `current_setting` errors, so a missing context fails closed.
5. **Connection pooling:** pooling is transaction-level and compatible with PgBouncer transaction mode, because `SET LOCAL` is transaction-scoped.
6. **Defense in depth:** queries still include `project_id` predicates explicitly (for index use and as a second guard). RLS is the backstop.
7. **Dedicated isolation tiers:**
   - `shared` (default);
   - `dedicated-database`: a project or tenant gets its own Postgres database in the same cell, with the same schema and routing by project→DSN map (EE);
   - `dedicated-deployment`: its own cell (ADR-0019, EE).
   Code paths are identical in all tiers.
8. **Caches and ephemeral stores** namespace keys by `project_id` (ADR-0007). In-memory snapshots are per project (ADR-0009).

## Rationale

RLS gives a database-enforced guarantee independent of application bugs. Shared schema makes projects cheap and migrations single-pass. Dedicated tiers satisfy regulated buyers without separate code.

## Alternatives considered

- **Schema-per-tenant:** migration fan-out and catalog bloat at 100k projects. Rejected.
- **Database-per-tenant as default:** costly and operationally heavy. Rejected (offered as the EE tier).
- **App-only filtering:** a single missed predicate leaks data. Rejected.

## Consequences

- Positive: strong isolation; cheap projects; uniform code.
- Negative: RLS adds planner overhead (small) and complicates cross-tenant reporting (which uses `auvora_platform`).
- Risks: RLS policy omitted on a new table. Mitigation: the CI check below.

## Fallback / revisit trigger

None.

## Implementation notes

- P0: `TenantTx`, roles, migration lint. P1: all tables. EE dedicated-database routing in P9.
- Config: `[database] url` (app role), `[database] migration_url` (owner role), `[database] platform_url` (platform role, optional; platform jobs are disabled if unset).

## Verification

- A CI migration lint queries `pg_class` after migrations and fails if any table in `auvora` schemas lacks `relrowsecurity AND relforcerowsecurity`, unless it's on the explicit global-table allowlist (`docs/specs/global-tables.md`).
- A property-based cross-tenant test generates two projects with random data, exercises every repository method under project A, and asserts that no project B row is returned or modified.
- `PlatformScope` usage is reviewed by CODEOWNERS.
