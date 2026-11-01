# ADR-0059: Compliance targets

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D59, §3.2, §18, §23 (FIPS), §27 (P9)
- **Related:** ADR-0012, ADR-0039, ADR-0049, ADR-0057, ADR-0063, ADR-0072, ADR-0079, ADR-0062

## Context

Enterprise and regulated customers buy identity platforms based on product compliance capabilities (FIPS, audit, residency) and, for Cloud, operator certifications. Some properties (FIPS, residency, audit immutability) must be designed in from P0.

## Decision

1. **Product capabilities (built in, available to self-hosted):**
   - **FIPS 140-3:** a FIPS build (`--features fips`) using the aws-lc-rs FIPS module (ADR-0012), restricting algorithms to the approved set, with self-tests at boot and fail-closed. The build exists from P0; officially supported from P7.
   - **Audit:** tamper-evident (ADR-0039) with export for evidence.
   - **Data residency:** cell pinning (ADR-0019).
   - **Privacy:** DSAR, erasure and consent records (ADR-0079, ADR-0054).
   - **Mappings:** compliance mapping docs per framework, showing which Auvora controls satisfy which requirements.
2. **Operator certifications for Auvora Cloud:**

| Certification | Target |
|---|---|
| SOC 2 Type I | Cloud private beta (after P5) |
| SOC 2 Type II | Cloud GA, ≤ 12 months after private beta |
| ISO/IEC 27001 (+ 27017, 27018, 27701) | Cloud GA + 6 months |
| GDPR + DPDP Act (India) + CCPA | DPA and processes at Cloud private beta |
| HIPAA | BAA available at Cloud GA (controls + docs) |
| PCI DSS | Service-provider SAQ-D readiness (Auvora isn't in the cardholder data flow; docs for customers' scoping) |
| CSA STAR Level 1 | Cloud GA |
| FedRAMP Moderate / StateRAMP | "FedRAMP-ready" architecture (P9); authorization pursued only with a sponsoring agency |
| India MeitY empanelment, EU Cloud Code of Conduct | Evaluate at Cloud GA based on demand |

3. **Compliance evidence packs (EE, P9):** generated reports (access reviews, admin changes, MFA coverage, key rotations, audit chain verification, support-access ledger) mapped to SOC 2 CC, ISO 27001 Annex A, HIPAA §164.312 and PCI DSS 4.0 req. 7/8.
4. **Control framework:** the Cloud uses a unified control set (SOC 2 + ISO 27001 superset) tracked as code in a separate internal repo; continuous control monitoring via a GRC tool chosen at Cloud private beta.
5. **Security standards for the product:** OWASP ASVS 5.0 Level 3 self-assessment for core crates before GA (published), NIST SP 800-63-4 alignment (ADR-0050).

## Rationale

Designing FIPS, audit and residency in from the start avoids costly retrofits. Operator certifications are timed to Cloud milestones, when they become commercially necessary.

## Alternatives considered

- **Certify everything at GA**: rejected as cost-prohibitive before revenue.
- **No FIPS until requested**: rejected because crypto-provider choices would be locked in by then.

## Consequences

- Positive: enterprise-ready story; reusable evidence for customers.
- Negative: audit costs; process overhead for the Cloud team.

## Fallback / revisit trigger

Certification dates move with the Cloud milestones (ADR-0063). Additional certifications are pursued when ≥ 3 qualified deals require them.

## Implementation notes

P0 FIPS build target in CI; P5 ASVS self-assessment; Cloud: SOC 2 readiness work starts with private-beta planning. Docs: `docs/site/trust/`.

## Verification

FIPS CI job (boot self-test, algorithm restriction tests); ASVS checklist tracked per release; external auditor reports.
