# ADR-0057: Cloud staff access only via Supervised Support Access

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D57, §16, §3.2
- **Related:** ADR-0094, ADR-0055, ADR-0063, ADR-0039, ADR-0059, ADR-0017

## Context

Customers of an identity SaaS must trust that operator staff can't silently read or modify their users. Insider abuse is in the abuse-case list (§26). Supervised Support Access is a headline feature, and dogfooding it on Cloud proves it.

## Decision

1. **No standing access.** Auvora Cloud staff have **no** standing read or write access to customer tenant data. `platform_operator` (ADR-0055) can operate infrastructure (deploy, scale, view metrics/logs that are PII-redacted) but can't query tenant tables. The RLS `platform` role is not granted to humans.
2. **All customer-data access goes through Supervised Support Access (ADR-0094)** with:
   - a customer-initiated or customer-approved request, tied to a support ticket ID;
   - default mode `view-as` with field masking. `act-as` requires explicit customer Owner/Admin approval per session;
   - max session duration 60 min (extendable once by customer approval), AAL3 for the staff member, and access from managed devices only (device-posture claim required);
   - every action audited in **the customer's own audit log** and visible in the console "Support access ledger" in real time, with an email notification to tenant Owners at session start and end.
3. **Emergency (incident) access:** for platform-wide incidents with no customer approval possible, a **break-glass** path requires two `platform_superadmin` approvals, is limited to the incident scope, and auto-notifies affected tenant Owners within 24 h with the full audit trail. It's reviewed in the next monthly security review and counted in the transparency report.
4. **Database-level access** (e.g. for corruption repair) uses just-in-time Postgres credentials issued for ≤ 60 min via the same dual-approval process. Sessions are logged with pgaudit, and the customer is notified when their rows are touched.
5. **Transparency:** a semi-annual transparency report publishes counts of support sessions, break-glass uses and law-enforcement requests.
6. **Contractual:** the Cloud DPA and Trust page state this ADR's guarantees.
7. **Self-hosted:** this ADR doesn't apply to self-hosted operators, but the same tooling is available to them.

## Rationale

This is the strongest trust posture an identity SaaS can offer. It turns a compliance requirement into a demonstration of the product's USP.

## Alternatives considered

- **Role-based internal admin console with audit** (industry norm): rejected because it's invisible to customers and allows standing access.
- **No access at all**: rejected because some support cases genuinely need it.

## Consequences

- Positive: customer trust; SOC 2/ISO evidence; dogfooding drives feature quality.
- Negative: slower support resolution; operational discipline required.

## Fallback / revisit trigger

None (permanent commitment for Cloud).

## Implementation notes

Effective from Cloud private beta (after P5). Supervised Support Access is delivered in P6, so between the P5 Cloud private beta and P6, only break-glass dual approval with customer notification is used, and customers are informed of this interim state.

## Verification

Quarterly access review; automated check that no human principal holds the RLS `platform` role; pgaudit log reconciliation against the support ledger; included in the SOC 2 control set (ADR-0059).
