# ADR-0094: Supervised Support Access

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §16 (and v0.2 §14 retained in full), §2.2 (#1), §2.3 (silent impersonation row), §27 P6, D57
- **Related:** ADR-0086, ADR-0057, ADR-0015, ADR-0039, ADR-0050, ADR-0055, ADR-0088, ADR-0091, ADR-0096, ADR-0101, ADR-0078

## Context

Support teams must see what a customer sees and sometimes act for them. Competitors offer "impersonation" that substitutes the support agent's identity for the user's, which erases accountability and invites abuse. Supervised Support Access is Auvora's headline differentiator. It meets the need through standards-based delegation with consent, scoping, visibility and full audit.

## Decision

### 1. Roles and entities

- **Support agent:** a principal with the control-plane role `Support` (ADR-0055), or a project-defined support role. The org-level equivalent is `org:support` for a customer's own helpdesk.
- **Support policy:** per project, with org overrides. It defines:
  - eligibility (CEL over the agent's attributes, groups and AAL)
  - permitted modes
  - permitted apps and resource servers
  - the support permission set (scopes, RAR types, Cedar policy set `support`)
  - consent mode, max duration and idle timeout
  - required ticket-system validation (optional hook)
  - field-masking classification
- **Support session:** `sup_` ID. Contains subject, agent, tenant/project/org, app/client, mode, scopes, purpose (ticket ID + free-text reason, ≥ 10 chars), approvals[], start/expiry/ended, end reason, and linked derived tokens.

### 2. Modes

- `view_as`: read-only. Write HTTP methods and mutation scopes are denied at the resource server. Sensitive fields are masked by default.
- `act_as`: scoped writes from the support permission set.
- `co_browse`: user-initiated from the account portal or app ("Let support see my screen/session"). The user must stay present (heartbeat). This mode allows view only, unless the user upgrades it to act.

### 3. Effective permissions

`effective = target_user_permissions ∩ agent_permissions ∩ support_policy_permissions`

These are evaluated by the authz engine (ADR-0015) at issuance **and** on every AuthZEN check through the `support_session` context. Always denied, regardless of policy:
- credential, MFA or recovery changes for the subject (they go through ADR-0078 admin-assisted recovery);
- changes to identifiers;
- creating support sessions;
- acting on other support agents or admins;
- payment-instrument reveal (resource servers tag these as `sensitive:never`);
- privilege grants.

### 4. Consent and approval

- Consent modes: `none` (tenant policy says the ToS covers it; still visible and logged), `user_approval` (in-app, email or CIBA push; default TTL 10 min), `org_admin_approval`, or `multi_approver` (n-of-m, EE).
- Org policy may only tighten the project policy.
- Workforce and regulated projects default to `user_approval`; CIAM projects default to `user_approval` for `act_as` and `none` for `view_as`.

### 5. Workflow (v0.2 §14.3, normative)

1. The agent authenticates with **AAL3 (phishing-resistant)**. AAL2 is allowed only if the policy explicitly sets `min_aal = 2`.
2. The agent selects tenant, user, app and mode.
3. The agent records the ticket and reason. An optional `pre-support-session` hook validates the ticket in the ticketing system.
4. Policy evaluation (Cedar `Action::"support_session.create"`).
5. Approvals are collected.
6. The session is created; default duration 30 min, max 4 h, idle 10 min.
7. A delegated token is issued via token exchange (ADR-0086): `sub` = user, `act` = {`sub`: agent, `support_session`: `sup_…`}, claim `support` = {`mode`, `restrictions`, `session_id`, `expires_at`}. TTL ≤ 5 min, re-minted while the session is active. **No refresh tokens.**
8. The app shows a support banner (SDK `useSupportSession()` / `<SupportBanner/>`; required by the client-profile flag `support_aware`). Clients that aren't marked `support_aware` cannot be targeted.
9. Every request is audited with actor and subject.
10. When the session ends (expiry, revocation, user, admin), derived tokens are invalidated within ≤ 5 s (ADR-0033), an SSF `session-revoked` event goes out, and a summary is sent to the user (if configured).

### 6. Visibility

- **Customer-visible ledger:** users see support sessions in the account portal (agent display name or pseudonym per policy, reason, time, mode, actions summary). Org admins see them for their org. Exportable.
- **Live controls:** user, org admin and project admin can terminate the session immediately.

### 7. Field masking

- Profile and token claims are filtered by the PII classification (ADR-0072): `sensitive` is masked in `view_as`.
- Resource servers receive `support.restrictions` (e.g. `mask:pii.sensitive`, `deny:payments`) and the SDK middleware enforces common patterns.
- Resource servers must enforce read-only at the API layer. The SDK provides `requireNotSupport()` and `allowSupportModes([...])` guards.

### 8. Break-glass

A separate `break_glass` capability (ADR-0101, EE workflow; basic sealed accounts are open):
- dual control, mandatory incident ID;
- immediate alerts to tenant owners and security contacts;
- post-incident review task;
- never available from a support session.

### 9. Auvora Cloud staff

Staff access customer tenants **only** through this mechanism (ADR-0057), with the support policy owned by the customer tenant and visible to them.

### 10. Configuration

`[projects.<id>.support] enabled` (default false), `modes`, `consent`, `min_aal`, `default_duration`, `max_duration`, `idle_timeout`, `ticket_hook`, `ledger_agent_display = name|pseudonym`; plus org overrides.

### 11. Edition

The core (all modes, consent, ledger, banner, audit) is **open**. Multi-approver workflows, ticketing-system connectors and advanced analytics are EE (ADR-0062).

## Rationale

Supervised Support Access replaces impersonation in Clerk, Keycloak and FusionAuth with something those can't offer: no silent actor substitution, least privilege through intersection, consent, customer-visible accountability and standards-based tokens. That makes it a credible enterprise and regulator-facing differentiator.

## Alternatives considered

- **Classic impersonation (log in as the user).** Rejected: erases the actor and allows privilege escalation.
- **Shared support accounts per tenant.** Rejected: no subject context and poor accountability.
- **UI-only read-only enforcement.** Rejected: bypassable. Enforcement happens at the resource/API layer.

## Consequences

- Positive: safe, auditable support; Auvora Cloud dogfoods it.
- Negative: resource servers must integrate the SDK middleware for full enforcement. Clients that aren't `support_aware` are excluded by design.
- Risks: social engineering of approval. Mitigated by binding messages, the ledger and anomaly detection on support activity (ADR-0097).

## Fallback / revisit trigger

An independent review finding a design-level flaw blocks P6 exit until it is fixed via a superseding ADR.

## Implementation notes

- Crates: `auvora-support`, `auvora-oauth` (exchange, claims), `auvora-authz` (support context), `auvora-flows` (approval, co-browse consent), `packages/*` (banner, guards).
- Ships in P6. Depends on token exchange (P5).

## Verification

- An abuse-case suite at 100% (P6 exit): nested support session, privilege escalation, refresh-token request, credential change, actor substitution, expiry/revocation propagation, non-`support_aware` client targeting, approval replay.
- Independent security review.
- Audit completeness test (every request carries `act`).
