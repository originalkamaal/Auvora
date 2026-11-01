# ee/: Enterprise Edition

Commercial code, licensed under [`ee/LICENSE`](LICENSE) rather than the repository's AGPL-3.0.

- What belongs here is defined by [ADR-0062](../docs/adr/0062-edition-boundary.md): governance, JIT privileged elevation, multi-approver support access, multi-region cells, HSM/FIPS extras and long-term audit retention. Every protocol and authentication method stays open source.
- Licensing rules: [ADR-0004](../docs/adr/0004-licensing-agpl-apache-commercial-cla.md).
- Dependency direction: open crates MUST NOT depend on `ee/` crates. EE plugs in through traits behind the Cargo feature `ee` ([ADR-0023](../docs/adr/0023-monorepo-layout.md)).

| Path | Contains |
|---|---|
| `crates/` | EE Rust crates (none yet) |
