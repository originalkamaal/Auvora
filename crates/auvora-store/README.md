# auvora-store

PostgreSQL persistence with tenant isolation enforced by row-level security. Design: [`docs/specs/tenancy-rls.md`](../../docs/specs/tenancy-rls.md), [ADR-0017](../../docs/adr/0017-tenancy-isolation-postgres-rls.md).

| Path | Contents |
|---|---|
| `sql/bootstrap.sql` | One-time database bootstrap (roles, schema), run by a superuser |
| `migrations/` | sqlx migrations, run as `auvora_owner` only (expand/contract, ADR-0025) |
| `src/store.rs` | `Store` (runtime `auvora_app` pool + boot safety check) and `TenantTx` |
| `src/scope.rs` | `TenantScope`: the tenant/project a transaction acts for |
| `src/repo/` | Repositories; each takes `&mut TenantTx` and never sees the pool. `audit` keeps the per-project SHA-256 hash chain (ADR-0039) |
| `src/platform.rs` | `PlatformScope`: the only RLS-bypassing access; every acquisition is recorded in `platform_audit_events` |
| `tests/rls_isolation.rs` | Isolation properties P1–P11, audit-chain and boot-check tests against PostgreSQL 16 (needs Docker) |

**Rules:**
- Every new table calls `auvora.enable_tenant_rls(...)` in its migration and tags every column with `pii=…`.
- The catalog lint (P11) and the catalog sweeps (P3–P5) pick up new tables automatically.
- Every new repository method must be exercised by P1/P2; the coverage gate enforces this.
