# ADR-0065: Product non-goals

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §2.4, §1.2 principle 13
- **Related:** ADR-0066, ADR-0100, ADR-0101, ADR-0096

## Context

The goal is superset coverage of IAM products. Explicit non-goals stop scope creep into *adjacent categories* that IAM competitors don't make core, while keeping integration points open.

## Decision

These are **not** built:

| Non-goal | Integration point instead |
|---|---|
| Endpoint management / MDM (device enrolment, configuration, remote wipe) | Consume device-compliance signals via SSF/CAEP receivers (ADR-0096), hooks, and Access Policy conditions |
| End-user password manager / secrets vault | Passkeys + Credential Exchange guidance; Token Vault only stores *upstream OAuth tokens for delegation* (ADR-0091) |
| Privileged session recording (keystroke/video/screen) | JIT elevation, audit and proxy access logs (ADR-0101, ADR-0100); export to dedicated recorders |
| Network firewall / ZTNA tunnel / VPN server | RADIUS for VPNs, identity-aware proxy, forward-auth (ADR-0100) |
| General workflow automation / iPaaS | Lifecycle Workflows limited to identity lifecycle; webhooks to external automation |
| Email marketing / CRM | Consent records and webhooks only |
| SIEM | Stream to SIEMs (ADR-0039) |

**Clarification:** the browser remote-access gateway (RDP/SSH/VNC) *is* in scope as an optional EE module (P10), because a reference IAM product (authentik) ships it. Session *recording* inside it remains a non-goal.

## Rationale

These categories have dedicated vendors and their own threat models. Adding them would dilute the identity core.

## Alternatives considered

- **Build MDM-lite**: rejected; large surface, low leverage.
- **Session recording**: rejected; high data-protection liability.

## Consequences

- Positive: focus, and clear messaging for sales and contributors.
- Negative: some buyers need partners for these functions.

## Fallback / revisit trigger

If ≥ 3 enterprise design partners need a non-goal and no integration path satisfies them, write a new ADR evaluating it as an EE add-on.

## Implementation notes

List the non-goals in the docs FAQ and the contributor guide.

## Verification

Reviewers reject PRs adding the listed capabilities, citing this ADR.
