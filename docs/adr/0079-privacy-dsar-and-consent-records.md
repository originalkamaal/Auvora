# ADR-0079: Privacy, DSAR and consent records

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §6.1 (PII tagging), §7.3, §13.3 (registration consent), §13.4 (account portal privacy), §27 P2
- **Related:** ADR-0053, ADR-0054, ADR-0072, ADR-0019, ADR-0039, ADR-0097, ADR-0109

## Context

Auvora holds personal data for customers subject to GDPR, UK GDPR, India DPDP Act 2023, CCPA/CPRA, LGPD and sector rules. The plan requires data-subject export and erasure, terms and privacy-version acceptance tracking, marketing consent, age gating and parental consent, and data residency. These are distinct from OAuth consent grants (ADR-0053), which record client authorization rather than legal consent.

## Decision

1. **Legal documents.** Each project (with org overrides) manages versioned `legal_documents` (terms, privacy, cookie, custom), with locale variants and a `requires_reacceptance` flag. When a new version requires re-acceptance, the login flow inserts an `accept-terms` step.
2. **Consent records.** `consent_records` is an append-only table. Each row has: principal, purpose (`terms`, `privacy`, `marketing_email`, `marketing_sms`, `analytics`, custom), document version, decision (granted/withdrawn), method (checkbox/API/import/admin), timestamp, IP/UA (policy-controlled), and evidence hash. Withdrawal is a new row, never an update. The records are exposed through the Admin API, the account portal and events (`auvora.consent.changed.v1`).
3. **DSAR export.** Users (account portal) and admins (API) can request a machine-readable export. It is a JSON bundle plus a human-readable HTML summary, covering every column tagged `pii` (ADR-0072), plus linked identities, sessions, devices, consents, grants, delegations, org memberships, and audit events where the user is the subject. The export runs as a worker job, is encrypted with a one-time key, downloads from a signed URL valid for 24 h, and requires AAL2 step-up to download. Default SLA is 7 days; typical completion is under 1 h.
4. **Erasure.** Erasure follows ADR-0054: soft delete, a 30-day purge, then pseudonymized audit. Legal hold blocks erasure and records the hold reason. Erasure emits `auvora.user.erased.v1` so downstream systems (via SCIM client or webhooks) can cascade.
5. **Rectification, restriction and objection:** handled through profile edit, `restricted` status (blocks processing except security and audit), and marketing-consent withdrawal.
6. **Age gating:** per-project `minimum_age` and a `birthdate` profile claim (optional, ADR-0076). Under-age registrations are either blocked or routed to a `parental_consent` flow step, in which a parent verifies by email or IDV (ADR-0098) and a linked guardian relation is stored.
7. **Data residency:** project PII stays in the project's home cell (ADR-0019). DSAR exports are generated in-cell.
8. **Privacy-preserving defaults:** IP and UA storage in audit is configurable (`privacy.store_ip = full|truncated|none`, default `full` for security events and `truncated` for activity). Risk signals store hashes, not raw fingerprints (ADR-0097).
9. **Phasing:** consent records, terms versions, DSAR export and erasure ship in **P2**. Age gating and parental consent ship in P5.

## Rationale

Without first-class privacy features, every customer has to build them on top. With them, Auvora is compliant by default across major regimes, and this exceeds most competitors, which offer erasure but rarely consent evidence or DSAR bundles.

## Alternatives considered

- **Leave privacy to customers via APIs only.** Rejected: it creates inconsistent evidence and makes compliance customers' problem.
- **Mutable consent flags on the user.** Rejected: no audit evidence.

## Consequences

- Positive: regulator-grade evidence; one DSAR button.
- Negative: storage for the append-only consent log; mitigated by partitioning (ADR-0109).
- Risks: an export leak exposes everything. Mitigated by step-up, encryption, short expiry and audit.

## Fallback / revisit trigger

New regulation that requires a capability not covered here triggers an additive ADR.

## Implementation notes

- Crates: `auvora-core` (consent types), `auvora-store`, `auvora-flows` (accept-terms, parental steps), worker jobs in `auvora-server`.
- Config section: `[projects.<id>.privacy]`.

## Verification

- A schema lint fails CI if any column is missing a `pii` tag classification.
- DSAR golden test: export contains every tagged column.
- Erasure test: no PII remains after the purge job (scan of all tagged columns and history tables).
