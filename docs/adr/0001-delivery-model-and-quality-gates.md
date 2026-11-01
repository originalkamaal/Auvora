# ADR-0001: Delivery model: multi-year, gated by quality

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §1, §4 D01, §27, §28 (D1, D13)
- **Related:** ADR-0002, ADR-0045, ADR-0064, ADR-0105, ADR-0106, ADR-0107

## Context

Auvora's scope is the whole IAM market: CIAM, B2B, workforce, federation, workload and agent identity, authorization and governance. The owner has said team size and timeline are not constraints ("we have lot of time and can deliver this in years") and that features must not be compromised. The v0.2 review found two linked risks: unbounded scope with no shippable slices, and vague exit criteria. A delivery model has to keep the full scope while still producing verifiable, releasable increments.

## Decision

1. Auvora is delivered as a **multi-year program** of phases P0–P10 (ADR-0064). Phases are gated by **exit criteria**, never by calendar dates. No phase may be declared complete, and no dependent phase may start its main track, until every exit criterion of its prerequisite phase is met.
2. **Scope is never reduced to meet time.** Time yields; scope and quality do not. A feature may move to a later phase only through a new ADR that names its new phase.
3. **Standing gates** apply to every phase and every release:
   - The threat model is updated for every new or changed feature (ADR-0049).
   - There are 0 open high or critical security findings.
   - Every new parser has fuzzed ≥ 24 h with no crashes.
   - The upgrade test from the previous release (N-1 → N) passes with zero downtime (ADR-0025, ADR-0045).
   - Docs and a runnable example exist for every new user-facing feature (ADR-0048).
   - The benchmark suite shows no regression greater than 5% against the previous release (ADR-0105).
4. **Exit criteria must be measurable.** Each one is a test, metric, certification or external artifact with a pass/fail result. Wording like "secure" or "works well" isn't acceptable.
5. Every phase ends in a **shippable, tagged release** with the label set in ADR-0064 (0.1 Alpha … 1.0 GA …).
6. Work inside a phase is organized into **parallel tracks** (Core, Authn, Flows, OAuth, DX, Ops…). Tracks may proceed independently within the phase's dependency graph.
7. Phase status is tracked in `docs/roadmap/STATUS.md`, with links to the CI evidence for each exit criterion.

## Rationale

Date-driven delivery is the usual way security products ship incomplete protocol handling. Gating on evidence gives the "earn trust through correctness" principle from plan v0.2 §28 an operational form. Shippable phases keep a multi-year program honest and give early adopters something real.

## Alternatives considered

- **Date-based roadmap with fixed scope:** forces quality cuts. Rejected.
- **MVP-first with scope reduction:** contradicts the owner's no-compromise stance. Rejected.
- **Big-bang 1.0 after all features:** no feedback for years, high integration risk. Rejected.

## Consequences

- Positive: predictable quality; every release is defensible; exit evidence doubles as marketing ("conformance-as-a-feature").
- Negative: time to GA is long, and early users see a deliberately incomplete product.
- Risks: gate fatigue and pressure to waive gates. Mitigation: waivers aren't permitted; changing a gate needs a superseding ADR.

## Fallback / revisit trigger

None. Changes to the gate list require a superseding ADR.

## Implementation notes

- P0 creates `docs/roadmap/STATUS.md` and a CI job `release-gate` that aggregates the standing-gate results (fuzz duration, security findings count from the tracker, upgrade test, bench diff) and blocks release tagging on any failure.
- Release tags are created only by the release workflow after `release-gate` passes.

## Verification

- The `release-gate` CI job is a required check for the `release/*` workflow.
- Phase-completion PRs must link evidence for each exit criterion. The reviewer checklist in ADR-0107 enforces this.
