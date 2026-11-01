# Tenancy isolation and row-level security

- **Status:** Draft
- **Date:** 2026-11-01
- **Governing ADRs:** ADR-0017 (primary), ADR-0019, ADR-0025, ADR-0026, ADR-0027, ADR-0039, ADR-0055, ADR-0057, ADR-0073, ADR-0109
- **Plan refs:** plan.md §6.1, §7
- **Companion spec:** [`domain-model.md`](domain-model.md)
- **P0 exit criterion served:** "RLS isolation property tests pass" ([`../roadmap/STATUS.md`](../roadmap/STATUS.md))

Isolation has three layers, each sufficient on its own for reads (defense in depth):

1. Rust types that make an unscoped query unrepresentable.
2. Explicit `project_id` predicates in every query.
3. PostgreSQL row-level security (RLS), forced on every tenant-owned table.

---

## 1. Table classes

| Class | Columns | RLS policy key | Examples |
|---|---|---|---|
| **Project-owned** | `tenant_id`, `project_id` | `project_id` **and** `tenant_id` | `principals`, `identifiers`, `oauth_clients`, `sessions_active`, `audit_events` |
| **Tenant-owned** | `tenant_id` | `tenant_id` | `tenants` (`id` = tenant), `projects`, `environment_groups` |
| **Cell-local platform** (schema `auvora`) | none | none (platform scope only) | `installation_settings`, `platform_keys`, `state_epoch`, `platform_audit_events`, `schema_migrations` |
| **Global directory** (schema `auvora_global`) | per table | none; reachable **only** through the `GlobalDirectory` trait | the closed list in [`cells.md`](cells.md) §4.2: `cell_registry`, `tenant_directory`, `project_directory`, `domain_directory`, `usage_counters`, `console_snapshots` |

- Every table in schemas `auvora` and `auvora_ee` MUST be project-owned or tenant-owned, unless it is on the **cell-local platform allowlist** (§5). Every table in `auvora_global` MUST be in the cells.md §4.2 list (cells.md invariant I-5).
- *Decision (spec-level):* the allowlist lives in §5 of this spec, in a machine-readable block. ADR-0017 links here.
- `projects` is tenant-owned, so the control plane can list a tenant's projects. Data-plane transactions set both keys, so reading their own project row works.

---

## 2. Database roles

| Role | Login | `BYPASSRLS` | Privileges | Used by | Config |
|---|---|---|---|---|---|
| `auvora_owner` | yes | no | Owns schemas, tables, functions | `auvora migrate` only | `[database] migration_url` |
| `auvora_app` | yes | **no** | `SELECT, INSERT, UPDATE, DELETE` on tenant tables; `INSERT, SELECT` only on `audit_events`, `audit_checkpoints`; `EXECUTE` on allowlisted functions (§5) | All runtime roles | `[database] url` |
| `auvora_platform` | yes | **yes** | As `auvora_app`, plus global tables; member of `auvora_retention` | Code holding a `PlatformScope` only | `[database] platform_url` (optional; platform jobs are disabled when unset) |
| `auvora_retention` | **no** | n/a | `EXECUTE` on `auvora.drop_expired_partitions(...)` and `auvora.detach_held_partitions(...)` | Retention jobs, via `auvora_platform` membership | — |

**Rules:**

1. `FORCE ROW LEVEL SECURITY` makes even the table owner subject to policies. Superusers still bypass RLS. **Boot check (MUST):** the server refuses to start if the `[database] url` role has `rolsuper` or `rolbypassrls`, or owns any table in `auvora`. *Decision (spec-level).*
2. `auvora_retention` is a fourth role, introduced by ADR-0039 §2 and not listed in ADR-0017. *Decision (spec-level):* it is a NOLOGIN role reached only through `auvora_platform` membership. Partition drops run in `SECURITY DEFINER` functions owned by `auvora_owner`, so no runtime role needs `DROP` rights.
3. `UPDATE` and `DELETE` on `audit_events` are revoked from every role. A trigger raises an error on any attempt (ADR-0039 §2).
4. Data backfills in migrations run per project, with the context set (§3). A cross-project backfill runs as a platform job with a `PlatformScope` and an audit reason.

---

## 3. Per-transaction context

### 3.1 SQL side

```sql
-- Helper functions: STABLE, so the planner evaluates them once per statement.
CREATE FUNCTION auvora.ctx_project_id() RETURNS uuid
  LANGUAGE sql STABLE PARALLEL SAFE
  AS $$ SELECT current_setting('auvora.project_id')::uuid $$;

CREATE FUNCTION auvora.ctx_tenant_id() RETURNS uuid
  LANGUAGE sql STABLE PARALLEL SAFE
  AS $$ SELECT current_setting('auvora.tenant_id')::uuid $$;
```

**Fail-closed semantics:**
- If the setting was never set in the session, `current_setting(...)` raises an error.
- If it was set earlier, `SET LOCAL` has reverted it to `''` at transaction end, and `''::uuid` raises an error.
- Either way, a missing context is an **error, never an empty or unfiltered result**.

### 3.2 Rust side (`auvora-store`)

```rust
/// Who the transaction acts for. Built only from a resolved request context:
/// host route → project (data plane) or an authorized admin target (control plane).
pub struct TenantScope { tenant_id: Uuid, project_id: Option<Uuid> }

/// The only handle repositories accept. Holds the open transaction and its scope.
pub struct TenantTx<'c> { tx: sqlx::Transaction<'c, Postgres>, scope: TenantScope }

impl Store {
    pub async fn begin(&self, scope: &TenantScope) -> Result<TenantTx<'_>, StoreError>;
}
```

`Store::begin` MUST execute, as the first statement of the transaction:

```sql
SELECT set_config('auvora.tenant_id', $1, true),
       set_config('auvora.project_id', $2, true);   -- $2 = '' for tenant-level scopes
```

**Rules:**
- Values are bound parameters. *Decision (spec-level):* use `set_config(…, is_local => true)`, never string-built `SET LOCAL`, so no SQL injection is possible.
- `set_config(…, false)` and plain `SET auvora.*` are forbidden. A CI grep check enforces this.
- Repository functions take `&mut TenantTx`. The `sqlx::PgPool` is private to `auvora-store` and is never handed out.
- Repository SQL still includes `WHERE project_id = $1`, with the value taken from `TenantTx.scope`, not from caller input (ADR-0017 §6).
- **Pooling:** compatible with PgBouncer transaction mode, because all context is transaction-local (ADR-0017 §5).

### 3.3 Platform scope

```rust
/// Capability token for RLS-bypassing work. Every acquisition is audited.
pub struct PlatformScope { reason: PlatformReason, actor: ActorRef, _sealed: () }

#[non_exhaustive]
pub enum PlatformReason {
    RetentionPurge, MeteringRollup, CellMigration, ConfigSnapshotPublish,
    HostRouteMaintenance, KeyRotationSweep, JobDispatch, MigrationBackfill { migration: &'static str },
    PlatformAdminRead { ticket: String },   // ADR-0057: Cloud staff only via support access
}

impl PlatformScope {
    /// Writes a `platform_audit_events` row (reason, actor, trace) in the same transaction.
    pub async fn acquire(reason: PlatformReason, actor: ActorRef, store: &PlatformStore) -> Result<PlatformTx<'_>, StoreError>;
}
```

**Rules:**
- `PlatformTx` connects with `platform_url`. Only functions in `auvora_store::platform::*` accept it.
- *Decision (spec-level):* `acquire` is a public function, but `scripts/check-platform-scope` (P0) fails CI if it is called outside the allowlisted modules (`auvora-store::platform`, retention, metering and cell-migration workers). CODEOWNERS review is required for those modules (ADR-0017 §3).
- A platform read of tenant data writes an audit event into the **affected project's** chain as well, so the tenant can see it (ADR-0039 §8).

---

## 4. RLS policy templates

Each table gets exactly one policy, named `tenant_isolation`, created by a helper. Hand-written policies are not allowed.

```sql
CREATE FUNCTION auvora.enable_tenant_rls(tbl regclass, level text) RETURNS void
LANGUAGE plpgsql AS $$
BEGIN
  EXECUTE format('ALTER TABLE %s ENABLE ROW LEVEL SECURITY', tbl);
  EXECUTE format('ALTER TABLE %s FORCE ROW LEVEL SECURITY', tbl);
  IF level = 'project' THEN
    EXECUTE format($p$CREATE POLICY tenant_isolation ON %s
      USING      (project_id = (SELECT auvora.ctx_project_id())
              AND tenant_id  = (SELECT auvora.ctx_tenant_id()))
      WITH CHECK (project_id = (SELECT auvora.ctx_project_id())
              AND tenant_id  = (SELECT auvora.ctx_tenant_id()))$p$, tbl);
  ELSIF level = 'tenant' THEN
    EXECUTE format($p$CREATE POLICY tenant_isolation ON %s
      USING      (tenant_id = (SELECT auvora.ctx_tenant_id()))
      WITH CHECK (tenant_id = (SELECT auvora.ctx_tenant_id()))$p$, tbl);
  ELSE
    RAISE EXCEPTION 'unknown tenancy level %', level;
  END IF;
  EXECUTE format('GRANT SELECT, INSERT, UPDATE, DELETE ON %s TO auvora_app', tbl);
END $$;
```

- *Decision (spec-level):* project policies check **both** `project_id` and `tenant_id`. It costs nothing and guards against a mis-scoped project ID from another tenant.
- `(SELECT …)` wrappers make the planner evaluate the context once per statement (initplan).
- `tenants` uses `id = (SELECT auvora.ctx_tenant_id())` and is created with a dedicated one-off policy. It is the only hand-written exception, and it is listed in the lint's expected-expression table.

### 4.1 Declaring RLS in migrations

Every migration that creates a table MUST call the helper in the same file:

```sql
CREATE TABLE auvora.identifiers (
  id uuid PRIMARY KEY,
  tenant_id uuid NOT NULL,
  project_id uuid NOT NULL,
  ...,
  UNIQUE (project_id, type, value_normalized_bidx),
  FOREIGN KEY (project_id, principal_id) REFERENCES auvora.principals (project_id, id)
);
SELECT auvora.enable_tenant_rls('auvora.identifiers', 'project');
```

The following object types have their own rules:

| Object | Rule |
|---|---|
| Partitioned tables | Call the helper on the **parent** only. Privileges on individual partitions are revoked from `auvora_app`, so partitions can't be queried directly. Partition-creation jobs run as owner (via a `SECURITY DEFINER` function). |
| UNLOGGED tables (ADR-0073) | Same helper. UNLOGGED doesn't change RLS. |
| Views | MUST be `WITH (security_invoker = true)`. Otherwise a view runs as its owner and bypasses RLS. |
| Functions | `SECURITY DEFINER` is forbidden except for the §5 allowlist. Allowlisted functions MUST `SET search_path = pg_catalog, auvora`. |
| Materialized views | Forbidden on tenant data (they have no RLS). Rollups are plain tables with RLS (ADR-0043). |
| Unique and composite keys | MUST lead with `project_id` (or `tenant_id` for tenant tables). |

### 4.2 Catalog lint (CI, after `auvora migrate` on an empty database)

The lint checks the following. Any violation fails CI:

- **L1:** every relation in `auvora` and `auvora_ee` with `relkind IN ('r','p')` that isn't allowlisted has `relrowsecurity AND relforcerowsecurity`.
- **L2:** each such table has exactly one policy, `tenant_isolation`, whose expression matches the template for its level.
- **L3:** each project-level table has `tenant_id uuid NOT NULL` and `project_id uuid NOT NULL`. Each tenant-level table has `tenant_id uuid NOT NULL`.
- **L4:** every unique index and PK on a project table, other than the `id` PK, leads with `project_id`.
- **L5:** every view has `security_invoker=true`. No materialized views exist.
- **L6:** `SECURITY DEFINER` functions are exactly the allowlist.
- **L7:** `auvora_app` has no privileges on global tables or on individual partitions.
- **L8:** every column carries a `pii=` comment (ADR-0072). This lint is shared with the domain-model spec.

---

## 5. Cell-local platform-table and definer-function allowlist

This block is machine-readable. The lint parses the fenced block.

```toml
[global_tables]
"auvora.schema_migrations"      = "migration ledger; owner only"
"auvora.installation_settings"  = "installation config; platform only"
"auvora.platform_keys"          = "cell/platform keys (ADR-0082 §2); platform only"
"auvora.state_epoch"            = "crash epoch (ADR-0073); SELECT granted to auvora_app"
"auvora.platform_audit_events"  = "platform audit chain (ADR-0039 §3); INSERT via platform scope"

[security_definer_functions]
"auvora_global.resolve_host(text)"      = "data plane resolves Host header before any scope exists, from domain_directory + project_directory (cells.md §4.2); returns (tenant_id, project_id, home_cell, state) only"
"auvora.drop_expired_partitions(text, timestamptz)" = "retention (ADR-0109); EXECUTE to auvora_retention"
"auvora.detach_held_partitions(text)"   = "legal hold (ADR-0109 §3); EXECUTE to auvora_retention"
"auvora.create_partitions(text, int)"   = "partition pre-creation; EXECUTE to auvora_app (worker)"
"auvora.enable_tenant_rls(regclass, text)" = "migration helper; EXECUTE to auvora_owner only"
```

---

## 6. Scope derivation (where `TenantScope` comes from)

| Entry point | Derivation |
|---|---|
| Data plane (Flow API, OAuth, SAML, SCIM, gateways) | The `Host` header or issuer path is resolved via `auvora.resolve_host()` to `(tenant_id, project_id)`, with a result cached per instance ≤ 60 s. **Request parameters never select the project.** |
| Token-authenticated APIs (AuthZEN, introspection, Admin API) | Project = the access token's `iss` → project, cross-checked against the resolved host. A mismatch returns 401. |
| Control plane (console, Admin API) | 1. Authenticate the admin in `prj_system` (its own transaction and scope). 2. Authorize the target tenant/project with the ADR-0055 role check. 3. Open a **new** `TenantTx` for the target. One transaction never spans two projects. |
| Background jobs | The job row carries `(tenant_id, project_id)`. The dispatcher claims due jobs across projects with `PlatformReason::JobDispatch` (`FOR UPDATE SKIP LOCKED`, reading only routing columns) and then executes each job body in a `TenantTx` for the job's project. *Decision (spec-level).* |
| Cross-project features (Auvora project connection, ADR-0027 §3) | Standard OIDC over HTTP between projects. They never share a transaction. |

Dedicated tiers (ADR-0017 §7): `TenantScope` also resolves to a DSN through the `project → database` map. Code is identical across tiers.

---

## 7. Failure modes

| Failure | Effect | Mitigation |
|---|---|---|
| Context not set (bug) | Query errors with `invalid input syntax for type uuid` or `unrecognized configuration parameter` | Fail closed. `auvora_store_context_missing_total` metric and a P1 alert. Property P4 (§8). |
| Wrong project in context (bug) | The query sees another project's data | Scope comes only from host/issuer resolution (§6). Explicit predicates use the same scope (consistency). P1/P2 tests. Composite FKs block cross-links. |
| App connected as superuser or owner | RLS is bypassed | Boot check refuses to start (§2 rule 1) |
| Context leaks across pooled connections | — | Impossible: `is_local => true` only. The CI grep bans session-level `set_config`. Pool-reuse property P9. |
| View, function or materialized view bypass | Silent cross-tenant read | Lint rules L5 and L6 |
| Platform credentials leaked | Full read | Separate optional secret, used only by worker/platform deployments. Every use writes a platform audit event. Cloud: network policy restricts `platform_url` to worker pods. |
| Cache key collision across projects | Data served to the wrong project | All cache and ephemeral keys are prefixed `p:{project_id}:` (ADR-0017 §8). Property P10 covers the in-memory caches. |
| Partition accessed directly | RLS on the parent is bypassed | Partition privileges revoked from `auvora_app` (lint L7) |
| Migration adds a table without RLS | Unprotected table | Lint L1 blocks the merge |

---

## 8. Isolation property-test plan (P0 exit criterion)

**Location:** `crates/auvora-store/tests/rls_isolation.rs`.

**Tools:**
- `testcontainers` with a PostgreSQL module and `proptest`.
- Default container: Postgres 16. Nightly matrix: 16, 17, 18.

**Setup per test run:**
1. Start a Postgres container and create the four roles (§2).
2. Run all migrations as `auvora_owner`.
3. Open pools as `auvora_app` and `auvora_platform`.

**Generators:**
- 2 tenants (`A`, `B`). Tenant `A` has projects `A1` and `A2`; tenant `B` has project `B1`.
- Random entity graphs per project, created **through repository methods**: principals of every kind, identifiers, orgs with nesting, memberships, clients and sessions.
- **Deliberate collisions:** identical normalized emails, org slugs, client names and identical UUID bodies in different projects. Each must succeed independently.

**Properties:**

| # | Property | Assertion |
|---|---|---|
| P1 | Read isolation | For every registered repository read method, under scope `A1`, every returned row has `project_id = A1` |
| P2 | Write isolation | Update or delete by the ID of a `B1` row under scope `A1` returns `NotFound` / 0 rows. A checksum of `B1` (computed via the platform role) is unchanged. |
| P3 | Insert check | A raw `INSERT` with `project_id = B1` under scope `A1` fails with an RLS `WITH CHECK` violation, for every table |
| P4 | Missing context | Any statement on any tenant table in a transaction without `set_config` errors. It never returns rows. |
| P5 | Catalog sweep | For **every** table found in `pg_class` (not hand-listed): `count(*)` under `A1` equals the platform-role count filtered to `A1`, and `UPDATE … SET <pk> = <pk>` in a rolled-back transaction touches only `A1` rows |
| P6 | Uniqueness scopes | The same identifier in `A1` and `B1` succeeds. The same identifier twice in `A1` violates the unique constraint (ADR-0075 §3). |
| P7 | Cross-project FK | Inserting an `A1` child that references a `B1` parent fails |
| P8 | Platform audit | Every `PlatformScope::acquire` writes exactly one `platform_audit_events` row with its reason |
| P9 | Pool reuse | With a pool of size 1, interleaving `A1`, `B1` and no-scope transactions 1,000 times never yields a cross-scope row and always errors without a scope |
| P10 | Tenant level | Under a tenant-only scope for `A`, `projects` returns only `A1` and `A2`, and project-level tables error (no project context) |
| P11 | Catalog lint | L1–L8 pass |

**Coverage gate:**
- *Decision (spec-level):* repository methods are declared through a `repository!` macro that also records them in a static registry.
- The test fails if any registered method was not exercised by P1/P2.
- New tables are covered automatically by P3, P5 and P11.

**Case counts:** proptest runs 256 cases per property in PR CI and 10,000 nightly. Failures persist regression seeds in `crates/auvora-store/tests/testdata/proptest-regressions/`.

**CI:** the `rust` job needs Docker, which `ubuntu-latest` provides. The test is part of `just test`, with no feature flag.

**P0 scope:** the P0 schema (`tenants`, `projects`, `environment_groups`, `principals`, `users`, `identifiers`, `audit_events`, `outbox_events`, `jobs`) passes P1–P11. From P1 on, each new table is covered automatically by P3, P5 and P11, and each new repository method by the coverage gate.

---

## 9. Conflicts found while writing this spec

| # | Conflict | Resolution here |
|---|---|---|
| T1 | ADR-0017's Verification section named `docs/specs/global-tables.md`; this spec keeps the allowlist in §5 instead. | Resolved: ADR-0017 link fixed |
| T2 | ADR-0039 §2 introduces an `auvora_retention` role that ADR-0017 §3 doesn't list. | Adopted as a fourth NOLOGIN role (§2) |
| T3 | ADR-0017 names `TenantTx` and `PlatformScope` only. This spec adds `TenantScope` (the scope value) and `PlatformTx` (the bypass transaction). | Compatible extension; no ADR change |
