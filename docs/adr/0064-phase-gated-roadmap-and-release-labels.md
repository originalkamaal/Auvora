# ADR-0064: Phase-gated roadmap and release labels

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §27, §28 (D1, D13)
- **Related:** ADR-0001, ADR-0045, ADR-0105, ADR-0106, ADR-0107

## Context

The scope is the whole IAM market, delivered over years with no feature compromise. Without gates, breadth erodes quality. The owner set no deadlines, so progress must be governed by verifiable outcomes.

## Decision

1. **Phases P0–P10** as defined in plan §27. Their order is normative. A phase *starts* only when the previous phase's exit criteria are met. P10 items start when their own prerequisite phase has exited.
2. **Parallel tracks** inside a phase are allowed. Work from a later phase may start early only as a spike behind a feature flag, without shipping it as supported.
3. **Release labels:**

| Phase exit | Version | Label | Support |
|---|---|---|---|
| P1 | 0.1 | Alpha | No compatibility guarantees |
| P2 | 0.2 | Alpha | No compatibility guarantees |
| P3 | 0.5 | **Public Beta = MVP** | API stable best-effort; migration tooling provided |
| P4 | 0.7 | Beta | Same |
| P5 | **1.0** | **GA** | SemVer guarantees (ADR-0045) |
| P6 | 1.1–1.2 | Minor | — |
| P7 | 1.3–1.4 | Minor | — |
| P8 | 2.0 | Major | — |
| P9 | 2.x | — | — |
| P10 | 3.x+ | Continuous | — |

4. **Standing gates (every phase):**
   - Threat models updated.
   - 0 open high/critical findings.
   - ≥ 24 h fuzzing per new parser with no crash.
   - Upgrade test from the previous release.
   - Docs and examples for every feature.
   - Benchmark regression ≤ 5%.
   - Admin API, console and Terraform parity for shipped features.
5. **Feature maturity flags:** `experimental` (off by default, may change) → `beta` → `stable`. Only `stable` features count toward a phase exit.
6. **Exit reviews** are recorded in `docs/releases/<version>-exit.md` with evidence links (CI runs, conformance results, benchmark reports).

## Rationale

Gates make "no compromise" achievable by refusing to trade quality for breadth. Labels set user expectations.

## Alternatives considered

- **Date-based releases**: rejected; they encourage shipping incomplete security features.
- **Strictly sequential phases with no spikes**: rejected; it delays risk discovery.

## Consequences

- Positive: predictable quality; evidence-backed releases.
- Negative: slower apparent progress.
- Risks: phase creep. Mitigation: scope changes need an ADR.

## Fallback / revisit trigger

If a phase exceeds 2× its internally estimated effort, hold a re-planning ADR that splits the phase. The superset scope is never cut.

## Implementation notes

The flag registry lives in `auvora-config` (`features.<name>.maturity`). The exit template lives in `docs/releases/`.

## Verification

- A release pipeline refuses to tag a phase-exit version without a merged exit document.
- A CI gate list is enforced by required checks.
