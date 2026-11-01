# ADR-0004: Licensing: AGPL server, Apache SDKs, commercial EE, CLA

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §3.2, §4 D04, §30
- **Related:** ADR-0003, ADR-0023, ADR-0062, ADR-0046

## Context

Licensing has to:
- protect the hosted-service business from free-riding cloud resellers;
- keep integration friction low for developers embedding SDKs in proprietary apps;
- let the commercial EE coexist in the same repo.

Dual licensing requires the project to hold sufficient rights over contributions.

## Decision

| Path | License | SPDX |
|---|---|---|
| `crates/**`, `apps/**` (server, console, hosted UI, org portal) | GNU Affero GPL v3.0 only | `AGPL-3.0-only` |
| `packages/**` (JS/TS SDKs, components), external SDK repos, Terraform provider, Helm charts, `deploy/**`, `examples/**` | Apache License 2.0 | `Apache-2.0` |
| `ee/**` | Auvora Commercial License | `LicenseRef-Auvora-Commercial` |
| `docs/**` | CC BY 4.0 | `CC-BY-4.0` |
| Hosted-UI SPA `apps/hosted-ui` (forkable for self-hosted UI, ADR-0021); the only `apps/` exception to AGPL | Apache-2.0 (also published to npm as `@auvora/hosted-ui` for forking) | `Apache-2.0` |

1. Every source file carries an SPDX header. The repo root has `LICENSE` (AGPL), `LICENSES/` (all texts) and `REUSE.toml`.
2. **Contributions require the Auvora CLA** (individual + corporate), enforced by a CLA bot on PRs. The CLA grants a copyright license broad enough for dual licensing. Contributors keep their copyright.
3. **Third-party dependency policy:**
   - Server crates may depend on MIT, Apache-2.0, BSD-2/3, ISC, Zlib, Unicode-3.0, MPL-2.0 (file-level), and LGPL dynamically linked (libxmlsec1, ADR-0014).
   - GPL-only dependencies are forbidden in Apache-licensed packages.
   - SSPL, BSL and Commons Clause dependencies are forbidden everywhere.
4. Commercial licensing of the AGPL server (an exception for OEM embedding) is available via the commercial license.
5. A public **License FAQ** explains that using SDKs in proprietary apps is unrestricted, and that calling the Auvora server over the network doesn't make the caller's code AGPL.

## Rationale

AGPL closes the SaaS loophole for the server. Apache SDKs remove adoption friction, since SDKs get linked into customers' code. The CLA makes dual licensing and EE legally clean.

## Alternatives considered

- **Apache-2.0 everywhere:** allows hyperscaler resale without contribution. Rejected for the server.
- **BSL/FSL with time-delayed conversion:** not OSI open source; harms trust in security software. Rejected.
- **DCO instead of CLA:** insufficient for relicensing EE or OEM. Rejected.

## Consequences

- Positive: a protected business model, friendly SDKs, legal clarity.
- Negative: some enterprises have AGPL policies; the CLA adds contributor friction.
- Risks: AGPL adoption hesitancy. Mitigation: the License FAQ plus the commercial license option.

## Fallback / revisit trigger

If AGPL demonstrably blocks adoption (tracked as ≥ 10 documented lost evaluations citing AGPL in a 12-month window), a superseding ADR may consider relicensing the server. The CLA makes this possible.

## Implementation notes

- P0: add license files, REUSE compliance, the CLA bot (CLA Assistant or equivalent), and `cargo-deny` license allowlists per workspace area.
- `cargo-deny` configs: `.deny.toml` (server), `packages/.licensee.json` (JS).

## Verification

- `reuse lint` and `cargo deny check licenses` are required CI checks.
- The CLA status check is required on PRs from non-maintainers.
