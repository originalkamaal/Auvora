# crates/: Rust server crates

Rust workspace members that make up the `auvora` binary. One crate per plan §5.5 area (`auvora-core`, `auvora-store`, `auvora-oauth`, …).

- **License:** AGPL-3.0
- **Governing decisions:** [ADR-0005](../docs/adr/0005-modular-monolith-single-binary-roles.md), [ADR-0023](../docs/adr/0023-monorepo-layout.md), [ADR-0047](../docs/adr/0047-rust-coding-standards.md)
- **Placement rules:** [ADR-0111](../docs/adr/0111-repository-organization-and-file-placement.md)

## Contents

| Path | Purpose |
|---|---|
| [`auvora-core/`](auvora-core/) | Domain primitives: prefixed UUIDv7 IDs (ADR-0018), injectable `Clock` (ADR-0047) |
| [`auvora-store/`](auvora-store/) | PostgreSQL persistence with RLS tenant isolation and isolation property tests (ADR-0017) |
| [`auvora-server/`](auvora-server/) | The `auvora` binary; wires roles together (ADR-0005) |
