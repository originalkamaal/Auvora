# ADR-0025: Database migrations: expand/contract

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D25, D45, §6, §29
- **Related:** ADR-0005, ADR-0017, ADR-0045, ADR-0071, ADR-0109

## Context

Auvora promises zero-downtime rolling upgrades from N-1 to N (ADR-0045). During a rolling upgrade, old and new binaries run against the same PostgreSQL schema at the same time. Identity data is critical: a broken migration can lock out every user.

## Decision

1. **Tool:** `sqlx migrate`. Migrations live in `crates/auvora-store/migrations/`, are embedded in the binary and are run by `auvora migrate up`. They never run implicitly on `serve` unless `[database] auto_migrate = true` (default `true` only in `auvora dev`, `false` otherwise).
2. **Expand/contract only.** Every schema change is split across releases:
   - **Expand (release N):** add nullable columns, new tables or new indexes (`CREATE INDEX CONCURRENTLY`). New code writes both old and new shapes and reads the new shape with a fallback.
   - **Migrate:** backfill via a resumable, batched background job (`auvora-store` backfill framework, ≤ 5,000 rows per batch, throttled).
   - **Contract (release ≥ N+1):** drop old columns or constraints only after the backfill completes and N-1 is no longer supported.
3. **Forbidden in a single release:** column rename, type change in place, `NOT NULL` without a default on a populated table, dropping anything still read by N-1, and long table-locking DDL. A migration linter (`squawk` + custom rules) enforces this.
4. **Lock safety:** every migration sets `lock_timeout = '5s'` and `statement_timeout`. Exceeding them fails the migration rather than blocking traffic.
5. **Versioning:** the schema version is recorded. The binary refuses to start if the DB schema is older than its minimum or newer than its maximum known version (N+1 contract-safe).
6. **Down migrations** are not used in production. Rollback means redeploying N-1, which works because of expand/contract. Backups are taken before `migrate up` in production profiles (documented runbook).
7. **RLS:** every new tenant-scoped table must ship with its RLS policy in the same migration (ADR-0017). A linter check enforces this.
8. **History tables** are created alongside their entity tables (ADR-0071).
9. **Partitioned tables** are managed by `auvora-store` partition jobs, not hand-written migrations (ADR-0109).
10. Refinery is not used.

## Rationale

`sqlx migrate` integrates with SQLx compile-time checks and keeps one tool. Expand/contract is the only way to guarantee N-1 compatibility with zero downtime.

## Alternatives considered

- **refinery**: no advantage over sqlx migrate. Rejected (plan D25).
- **ORM auto-migrations**: unsafe. Rejected.
- **Blue/green with DB copy**: impractical for large identity stores. Rejected.

## Consequences

- Positive: safe rolling upgrades; predictable rollback.
- Negative: schema changes take two or more releases; dual-write code is temporarily more complex.
- Risks and mitigations: forgotten contract steps are tracked by a `contract-pending` registry file that CI reports on.

## Fallback / revisit trigger

None. This is permanent for production. Breaking changes are only allowed in a major version with a documented offline migration.

## Implementation notes

- P0: migration framework, linter, `auvora migrate {up,status,verify}`.
- P1: upgrade test job (deploy N-1, load data, roll to N under k6 load, assert zero failed requests).
- Config: `[database] url`, `auto_migrate`, `migration_lock_timeout`; env `AUVORA_DATABASE__URL`.

## Verification

CI: migration linter, RLS-policy presence check, upgrade test N-1→N on every release branch, schema-diff snapshot test.
