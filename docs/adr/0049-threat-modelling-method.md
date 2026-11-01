# ADR-0049: Threat modelling method

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D49, §26, §27 (standing gates), §29
- **Related:** ADR-0106, ADR-0107, ADR-0108, ADR-0072, ADR-0059

## Context

Principle #1 makes threat modelling a release requirement. Without a fixed method, artifact location and enforcement point, it degrades into ad-hoc review.

## Decision

1. **Methods:** **STRIDE** per component/data flow for security, and **LINDDUN** for privacy threats on any feature that processes PII (ADR-0072 tags). Abuse cases from plan §26 form the seed catalog.
2. **Artifacts:** `docs/threat-models/<feature>.md` (one per feature/protocol). Contents:
   - scope and assets;
   - data-flow diagram (Mermaid) with trust boundaries;
   - STRIDE table (threat, impact, likelihood [1–5], mitigation, test reference);
   - LINDDUN table where applicable;
   - residual risks with owner sign-off.
   Sensitive details use front-matter `public: false` and aren't published.
3. **Severity scoring:** likelihood × impact (1–25). ≥ 15 = High, which MUST have a mitigation and a test before merge.
4. **Traceability:** every mitigation links to a test ID (`abuse::<feature>::<case>`) in the security regression suite (ADR-0106). CI verifies that referenced test IDs exist.
5. **When required:** any PR labelled `security` or touching crates under CODEOWNERS security ownership (ADR-0046) MUST update or reference a threat model. The PR template has a mandatory "Threat model" field.
6. **Cadence:** models are reviewed at every phase exit (standing gate) and before external pentests. They are re-opened on any security incident or advisory in the area.
7. **Platform model:** `docs/threat-models/platform.md` covers the system as a whole: tenancy isolation, admin/insider, supply chain, key custody, cells.

## Rationale

STRIDE is widely understood and maps well to components. LINDDUN covers privacy, which STRIDE misses. In-repo artifacts with test traceability make the models living documents.

## Alternatives considered

- **PASTA**: too heavyweight for per-feature use.
- **Attack trees only**: useful, but allowed inside models as a supplement, not as the primary method.
- **External tooling (IriusRisk, Threat Dragon)**: rejected because docs-as-code keeps models reviewable in PRs. Threat Dragon files are optional.

## Consequences

- Positive: auditable security reasoning; evidence for SOC 2/ISO (ADR-0059).
- Negative: overhead per feature.

## Fallback / revisit trigger

None.

## Implementation notes

P0: template `docs/threat-models/_template.md`, `platform.md`, `tenancy.md`, `keys.md`; PR template; CI checker script `scripts/check-threat-models`.

## Verification

CI checks that security-labelled PRs touch or reference a threat model, and that mitigation test IDs resolve. The phase-exit checklist includes model review sign-off.
