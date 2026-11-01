# ADR-0089: Org Admin Portal

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §12.4, §5.5 (`apps/org-portal`), §27 P4, §2.1 (WorkOS Admin Portal)
- **Related:** ADR-0088, ADR-0014, ADR-0099, ADR-0090, ADR-0095, ADR-0068, ADR-0021

## Context

The hardest part of enterprise SSO for a SaaS developer is onboarding each customer's IT admin: exchanging SAML metadata, mapping attributes, issuing SCIM tokens and debugging failures. WorkOS built a business on a self-service portal for this. Auvora includes it in the open edition as a B2B wedge.

## Decision

1. **The portal** is a standalone SPA (`apps/org-portal`) served by the binary, authenticated by a **portal session**, not a full Auvora account.
2. **Access:**
   - The developer (via API/SDK/console) generates a portal link: `POST /v1/organizations/{id}/portal-links` with `intents[]` (`sso`, `scim`, `domains`, `audit_logs`, `log_streams`, `branding`), `return_url` and `expires_in` (default 5 min to first use, then a 60-min session).
   - The link is single-use, and its token is bound to the org.
   - Alternatively, an org member holding `org:admin` + `org:sso:manage` can open the portal through normal sign-in. That path is used after initial onboarding.
3. **SSO setup wizards** per IdP: Okta, Entra ID, Google Workspace, ADFS, Ping, OneLogin, JumpCloud, Keycloak, generic SAML, generic OIDC. Each wizard:
   - Shows provider-specific screenshots and steps.
   - Exposes Auvora SP metadata (URL, XML download, ACS, entity ID).
   - Accepts IdP metadata (URL or XML upload, SSRF-guarded via ADR-0103).
   - Previews attribute mapping.
   - Runs a **live test sign-in** with a step-by-step decoded trace (signature, conditions, attributes, errors with fix hints).
4. **SCIM setup:**
   - Generates a SCIM base URL and bearer token, shown once (rotatable, two active during overlap).
   - Wizards per IdP.
   - A live provisioning log viewer.
   - Attribute mapping and group → role mapping.
5. **Domains:** add and verify via DNS TXT with live checking; set the auto-join and SSO-capture policy (ADR-0088).
6. **Certificates:** expiry warnings to org admins at 30, 14, 7 and 1 days; guided rollover with dual-certificate overlap.
7. **Audit and log streams:** org-scoped audit viewer and Customer Audit Log API streams (ADR-0095); SIEM stream setup.
8. **Embedding modes:**
   - Hosted at `https://{custom-domain}/org-portal`.
   - React and Web Components (`<AuvoraOrgPortal/>`) rendered in the developer's origin, using a short-lived portal token. **No iframes.**
   - Headless: every portal action is a public Admin API endpoint scoped to the org.
9. **Branding:** inherits the project and org theme (ADR-0068). Developer logo and support link.
10. **Security:**
    - Portal tokens only grant actions for the stated intents and org. Every action is audited, with the actor recorded as `portal_session` plus the linked user if known.
    - Changes to an active SSO connection require test success before activation (a staged → active swap with rollback).

## Rationale

Self-service onboarding removes the biggest B2B integration cost and matches WorkOS, with debugging depth no competitor offers in open source.

## Alternatives considered

- **Console-only configuration by the developer.** Rejected: puts the developer in the middle of every customer IT exchange.
- **iframe embedding.** Rejected: clickjacking and CSP complexity.

## Consequences

- Positive: a strong B2B differentiator; reduced support load.
- Negative: wizards for each IdP need ongoing screenshot and doc maintenance.

## Fallback / revisit trigger

None.

## Implementation notes

- App: `apps/org-portal`. APIs: `auvora-api` (org-scoped endpoints). Depends on SAML SP and SCIM server in P4.

## Verification

- Playwright E2E runs per wizard against IdP test tenants (ADR-0060).
- Portal-token scope tests.
- Staged-activation rollback test.
