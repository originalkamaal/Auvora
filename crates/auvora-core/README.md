# auvora-core

Domain primitives shared by every Auvora crate.

| Module | Purpose | Decision |
|---|---|---|
| `id` | External identifiers `<prefix>_<26-char base32>` over UUIDv7, closed prefix registry, strict parsing | [ADR-0018](../../docs/adr/0018-identifier-format-uuidv7-prefixed.md) |
| `time` | `Clock` trait; the only sanctioned wall-clock access | [ADR-0047](../../docs/adr/0047-rust-coding-standards.md) §10 |

Planned additions: tenancy context (ADR-0017) and the shared error model (ADR-0020).
