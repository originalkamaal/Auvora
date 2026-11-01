# Phase status

**Current phase:** P0 (Foundations). No phase has exited yet.

Update this file in the same commit that completes a deliverable. Link evidence such as a commit, CI run or document. A phase exits only when every exit criterion is checked ([ADR-0001](../adr/0001-delivery-model-and-quality-gates.md)).

## P0: Foundations

### Deliverables

| # | Deliverable | Status | Evidence |
|---|---|---|---|
| 1 | Plan v1.0 and decision records ADR-0000…0112 | Done | [`docs/product/plan.md`](../product/plan.md), [`docs/adr/`](../adr/README.md) |
| 2 | Licensing: AGPL root, Apache SDK/packages, EE license | Done (EE terms pending) | `LICENSE`, `ee/LICENSE` |
| 3 | Repository skeleton per ADR-0111, community files | Done | root `README.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, `CONTRIBUTING.md` |
| 4 | Rust workspace, toolchain pins, lint policy (ADR-0047), cargo-deny (ADR-0004/0012) | Done | `Cargo.toml`, `.clippy.toml`, `.deny.toml`, `rust-toolchain.toml`, `.mise.toml` |
| 5 | ID scheme implementation (ADR-0018) and `Clock` | Done | `crates/auvora-core` |
| 6 | Repository checks + commit-msg hook (ADR-0000/0111/0112), `justfile` | Done | `scripts/`, `justfile` |
| 7 | CI: fmt, clippy, test, deny, docs, commits | Done (first run pending push) | `.github/workflows/ci.yml` |
| 8 | Supply chain extras (ADR-0046, ADR-0001) | Partial: cargo-vet, gitleaks, Renovate done; SBOM, signed releases, reproducible builds and the `release-gate` job come with the release workflow | `.cargo/supply-chain/`, `.gitleaksignore`, `.github/renovate.json` |
| 9 | CLA process published (ADR-0004) | Todo | |
| 10 | Domain model spec + ERD | Done (Draft) | [`docs/specs/domain-model.md`](../specs/domain-model.md) |
| 11 | Tenancy/RLS design spec, incl. isolation property tests plan (ADR-0017) | Done (Draft) | [`docs/specs/tenancy-rls.md`](../specs/tenancy-rls.md) |
| 12 | Cell-ready routing design (ADR-0019) | Done (Draft) | [`docs/specs/cells.md`](../specs/cells.md) |
| 13 | Threat-model template + platform threat model; data classification and PII tagging scheme (ADR-0049, ADR-0072) | Done (Draft) | [`docs/threat-models/`](../threat-models/README.md), [`docs/security/data-classification.md`](../security/data-classification.md) |
| 14 | `auvora-store` skeleton with RLS harness (Testcontainers) | Done | [`crates/auvora-store`](../../crates/auvora-store/README.md): full P0 schema; properties P1–P11, audit-chain and boot-check tests; `scripts/check-store-rules` |
| 15 | `auvora-crypto` (KeyProvider: local + AWS KMS) and `auvora-jose` skeletons with fuzz targets | Todo | |
| 16 | Conformance harness (OIDF suite container) and bench harness | Todo | |
| 17 | Spikes with go/no-go reports: samael interop (5 IdPs), webauthn-rs, CEL hardening, wasmtime limits, Cedar entity loading | Todo | |
| 18 | Brand assets placement (logo) | Done | [ADR-0113](../adr/0113-brand-assets-location.md), [`docs/brand/`](../brand/README.md) |

### Exit criteria

- [ ] All ADRs merged (done) and the spikes are go, or a fallback was triggered and documented
- [ ] CI green with fuzzing, cargo-deny and SBOM
- [x] RLS isolation property tests pass (P1–P11 in `crates/auvora-store/tests/rls_isolation.rs`, mutation-checked)
