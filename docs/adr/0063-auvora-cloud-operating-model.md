# ADR-0063: Auvora Cloud operating model and launch regions

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §3.2, §5.3, §27 (P5, P9)
- **Related:** ADR-0003, ADR-0019, ADR-0057, ADR-0059, ADR-0062

## Context

The business model is open-core plus a managed cloud (ADR-0003). Its timing, regions and operating rules must be fixed so that the architecture (cells, residency) and compliance work are planned, not improvised.

## Decision

1. **Same artifact:** Cloud runs the exact released `auvora` + `ee` binaries. There is no cloud-only fork. Cloud-only code is limited to billing, signup and metering services in a separate `cloud/` private repository that integrates only through public APIs and the global control (ADR-0019).
2. **Timing:**
   - Private beta after GA 1.0 (end of P5).
   - Public GA after P9 exit (cells GA, SOC 2 Type II, ISO 27001).
3. **Launch regions (one cell each):**
   - EU: Frankfurt
   - US: N. Virginia
   - India: Mumbai
   Additional cells (UK, Australia, Canada, Japan, Singapore) are added on demand.
4. **Residency:** a project's home cell is chosen at creation and is immutable except via the migration tool. PII never leaves the home cell (ADR-0019).
5. **Tiers:**
   - Free/developer: shared cell, limits on projects, MAU and M2M tokens.
   - Pro: shared cell.
   - Enterprise: shared or dedicated cell, EE features.
   - Dedicated/private: a single-tenant cell in a customer-chosen region or cloud account.
6. **Metering units:** MAU (per project, unique principals with an authentication event in the calendar month), M2M token issuances, agent principals, Organizations with SSO connections. Metering is aggregated in-cell; only counts leave the cell.
7. **Staff access** to customer data is only via Supervised Support Access (ADR-0057).
8. **SLO:** 99.95% data plane at Cloud beta, 99.99% after cells GA (ADR-0105).
9. **Dogfooding:** the Cloud console and staff authentication use Auvora itself (a dedicated internal project).
10. **Infrastructure:** Kubernetes, managed Postgres (or Patroni where unavailable), cloud KMS per cell. Terraform for all cell infrastructure.

## Rationale

Starting Cloud after GA avoids operating an unproven security product for others. Three regions cover the primary early markets and residency regimes (GDPR, US, DPDP).

## Alternatives considered

- **Cloud before GA**: rejected; reputational risk.
- **Single global region**: rejected; residency requirements.
- **A cloud-only fork**: rejected; it diverges from self-hosted quality.

## Consequences

- Positive: self-hosted and Cloud share one tested artifact; residency built in.
- Negative: three cells from day one cost more than one.
- Risks: the operational load of a small team. Mitigation: automation, error budgets, staged rollouts per cell.

## Fallback / revisit trigger

If private beta demand is under 20 active design-partner projects after 6 months, keep a single EU cell and postpone US/India.

## Implementation notes

The cell design is frozen in P0 (ADR-0019). Billing and signup are built post-P5. SOC 2 Type I during private beta.

## Verification

- A release pipeline check verifies that Cloud image digests equal public release digests.
- A residency audit job detects cross-cell PII flow.
