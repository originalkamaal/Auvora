# ADR-0101: Governance and privileged access

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §18, §3.2 (editions), §11.3, §2.1 (Okta Workflows, Entra PIM/entitlements/reviews), §27 P4/P8
- **Related:** ADR-0062, ADR-0015, ADR-0016, ADR-0038, ADR-0055, ADR-0091, ADR-0094, ADR-0097, ADR-0099

## Context

Workforce and regulated customers need identity governance and administration (IGA): lifecycle automation, access requests, entitlements, periodic reviews, separation of duties (SoD) and just-in-time (JIT) privileged access. Okta and Entra offer these. Smaller deployments mustn't be forced to run a heavy IGA subsystem (principle from v0.2 §16).

## Decision

1. **Module layout:** `auvora-lifecycle` (open: engine, triggers, built-in steps) and `ee/crates/auvora-governance` (EE: requests, packages, reviews, SoD, JIT, advanced posture, workflow designer UI). Governance tables exist only when EE governance is enabled (`[governance] enabled = true`).
2. **Lifecycle Workflows (open engine):**
   - Triggers: principal created/updated/attribute-changed (CEL predicate), HR events (ADR-0099), schedule, manual, and `leave_date` reached.
   - Steps: assign/remove role/group/org membership, provision/deprovision via SCIM client, send notification, revoke sessions, rotate credentials, transfer ownership (service accounts and agents), wait/delay, approval (EE), HTTP hook (ADR-0038), Wasm action (ADR-0081).
   - Workflows are versioned, dry-runnable and idempotent per `(workflow_version, principal, trigger_event)`.
3. **Access requests and access packages (EE):** a package bundles roles, groups, app assignments and ReBAC tuples, with eligibility (CEL), an approval chain (sequential/parallel, manager/owner/specific/group; default 2-step for privileged packages), a duration (max 365 days) and auto-expiry.
4. **Access reviews (EE):**
   - Campaigns over users, service accounts, **agents and delegations** (ADR-0091) and support-access policies (ADR-0094).
   - Reviewers: manager, resource owner, self, or a group.
   - Decisions: approve, revoke, delegate. Optional recommendations based on last-used and peer analysis.
   - An unreviewed item follows the configured default: `keep`, or `revoke` (the default for privileged items).
5. **SoD (EE):** conflict rules expressed as Cedar policies (ADR-0015), evaluated at assignment time (`block` or `require_exception_approval`) and in a daily detective scan.
6. **JIT privileged elevation (EE):**
   - Roles flagged `eligible_only` can't be permanently assigned.
   - Activation needs AAL3 (ADR-0050), a justification, optional approval, and a max duration (default 1 h, cap 24 h).
   - It emits CAEP `token-claims-change` and auto-revokes at expiry.
7. **Break-glass accounts (open, P4):** sealed credentials (hardware key), excluded from conditional access only with explicit policy, high-severity alert on every use, monthly use-test reminder.
8. **Posture/ITDR reports:**
   - Open: dormant users (> 90 days), MFA coverage, stale credentials (> 365 days).
   - EE: over-privilege, orphaned service accounts and agents, toxic combinations.
9. **Delegated administration (open, P4):** admin roles scoped to org, app or group, building on ADR-0055.

## Rationale

The engine is open so that JML automation is available to everyone. The governance processes that large operators pay for go in EE, following ADR-0062 with no SSO tax. Reviews include agents and delegations, which competitors largely don't cover yet.

## Alternatives considered

- **Integrate with an external IGA (SailPoint, Saviynt) only:** the connectors are still provided via SCIM and events, but Auvora-native governance is needed for the superset.
- **Governance fully open:** removes the main EE value; the edition boundary was set by ADR-0062.

## Consequences

- Positive: Entra/Okta-class governance in the same product.
- Negative: a large P8 scope, mitigated by design-partner pilots (the P8 exit criterion).
- Risk: workflow bugs could mass-revoke access. Mitigations: dry-run, the safety threshold (shared with ADR-0099), and workflow version rollback.

## Fallback / revisit trigger

If two design partners can't complete a quarterly review campaign in the pilot, revisit the review UX and recommendation scope before 2.0 GA.

## Implementation notes

- P4: break-glass, delegated admin.
- P8: everything else.
- Config: `[lifecycle]`, `[governance]`.

## Verification

- Workflow idempotency property tests.
- SoD Cedar policy tests.
- JIT expiry revocation ≤ 5 s p99.
- Campaign end-to-end Playwright suite.
- Design-partner pilot report.
