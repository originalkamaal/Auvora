# ADR-0054: Deletion and retention of identities

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D54, §6.1, §7.3, §13.4
- **Related:** ADR-0039, ADR-0071, ADR-0072, ADR-0079, ADR-0109, ADR-0026, ADR-0018

## Context

GDPR, DPDP (India) and CCPA require erasure. Operators need undo for accidental deletes. Audit must stay intact and tamper-evident without retaining PII forever. Legal holds must override deletion.

## Decision

1. **States:** `active` → `suspended` (reversible, no login) → `deleted` (soft) → `purged` (hard). Applies to users, service accounts, agents, organizations and projects (with cascading rules below).
2. **Soft delete:**
   - immediately revokes all sessions, refresh tokens, grants, delegations, Token Vault entries, API keys and push devices, and emits CAEP/RISC `account-disabled`;
   - identifiers are released after purge, not at soft delete (prevents takeover-by-reregistration during the undo window);
   - the principal is restorable for `retention.soft_delete_days = 30` (project setting, range 0–90; 0 = immediate purge).
3. **Purge (hard delete):** a worker job after the window. It:
   - deletes PII-tagged columns and rows (ADR-0072), linked identities, credentials, profile and metadata;
   - deletes or rewrites `_history` rows (ADR-0071) of the principal to tombstones (PII fields nulled; structural fields kept);
   - removes search-index documents (ADR-0042);
   - **keeps** the immutable `principal_id` in a `tombstones` table with only `deleted_at` and `purged_at`.
4. **Audit:** events keep the pseudonymous `principal_id` but never re-identify after purge. Audit stores only PII fields classified `audit_allowed` (e.g. email at event time), following the canonical scheme in ADR-0039 §7: sidecar table `audit_event_pii` encrypted with the per-principal DEK, chain over an HMAC. Purge destroys the DEK (crypto-shredding) and deletes sidecar rows; hash-chain verification is unaffected.
5. **Legal hold:** `legal_hold` on a principal, org or project (set by admins with the `Auditor`+`Owner` dual approval, ADR-0055) blocks purge and DEK destruction until released. Holds have a reason, a case ID and an expiry review date.
6. **Organization deletion:** soft-deletes memberships, org connections and org-scoped roles. Users are not deleted unless `delete_members_without_other_memberships = true` is chosen explicitly.
7. **Project/tenant deletion:** a 30-day soft delete (non-configurable, below which deletion needs Owner + AAL3 + typed confirmation). Purge removes all data and destroys the project KEK/DEKs. Signing-key public JWKS stay published (verify-only) for `max token lifetime + 7 days`, then stop.
8. **Inactive accounts:** an optional policy `lifecycle.inactive_delete_after_days` (off by default) with notification at T-30 and T-7.
9. **Backups:** purged data persists in backups until backup expiry (default 35 days for Cloud). Restores re-apply the tombstone list before the service is reopened (`auvora restore --apply-tombstones`).

## Rationale

Crypto-shredding reconciles immutable audit with the right to erasure. The soft-delete window prevents irreversible admin mistakes. Identifier release after purge prevents takeover.

## Alternatives considered

- **Immediate hard delete**: rejected because there's no recovery and it races with audit.
- **Removing audit rows on erasure**: rejected because it breaks tamper evidence.

## Consequences

- Positive: compliant erasure with intact audit; undo window.
- Negative: complexity of per-principal DEKs in audit and restore tombstone handling.

## Fallback / revisit trigger

If per-principal DEK volume causes measurable audit write latency (> 10% of audit insert p99), switch to per-principal-per-month DEKs (new ADR).

## Implementation notes

P1 soft delete + purge + tombstones; P2 DSAR integration (ADR-0079); P4 legal hold and org deletion; P9 restore tombstone tooling. Crates: `auvora-store`, `auvora-audit`, `auvora-crypto`.

## Verification

Integration tests: post-purge DB scan for PII fields; audit chain verification after purge; restore test with tombstones; legal-hold blocking test.
