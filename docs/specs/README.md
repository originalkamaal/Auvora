# Specs

Technical specifications: **how** a subsystem works. Decisions and their reasons live in [`../adr/`](../adr/README.md); a spec must not contradict an Accepted ADR (ADR-0111 §2).

| Spec | Covers | Status |
|---|---|---|
| [`domain-model.md`](domain-model.md) | Canonical entities, fields, invariants, lifecycles, ERDs | Draft |
| [`tenancy-rls.md`](tenancy-rls.md) | Tenant isolation: database roles, per-transaction context, RLS templates, isolation property tests | Draft |
| [`cells.md`](cells.md) | Multi-region cells, global router and directory, cell migration, invariants that must hold from P1 | Draft |
