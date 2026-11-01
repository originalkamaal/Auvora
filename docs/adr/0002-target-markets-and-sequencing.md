# ADR-0002: Target markets and sequencing

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §1, §2, §3.1, §4 D02, §27
- **Related:** ADR-0001, ADR-0003, ADR-0062, ADR-0064, ADR-0091

## Context

IAM buyers fall into several segments with different first needs. The owner wants Auvora to cover all of them: "mostly as recommended but we want to cover most of types including enterprise B2B." Without an explicit order, every segment competes for early phases and none is served well.

## Decision

1. **All segments are in scope:**
   - B2B SaaS developers
   - AI agent and workload platforms
   - Enterprise B2B (customers' IT admins configuring SSO/SCIM)
   - Workforce IAM
   - Consumer CIAM
   - Authorization/FGA buyers
   - Governance/compliance teams
2. **Primary segment:** B2B SaaS developers, positioned against Auth0, WorkOS and Clerk. They decide the defaults, docs voice, quickstarts and the MVP (0.5 Public Beta).
3. **Secondary segment:** agent and workload identity. This is the headline differentiator at GA 1.0 (ADR-0091, ADR-0092).
4. **Sequencing**, with the phase at which each segment becomes *fully served*:

   | Segment | Fully served at |
   |---|---|
   | B2B SaaS developers (CIAM core, orgs, SDKs) | P3 (0.5 Beta) → P4 (0.7) |
   | Enterprise B2B (SSO/SCIM, Org Admin Portal) | P4 (0.7) |
   | Agent, machine and workload identity | P5 (1.0 GA) |
   | Authorization platform (FGA) | P6 (1.1–1.2) |
   | Workforce / legacy (SAML IdP, LDAP, RADIUS, Kerberos, proxy) | P7 (1.3–1.4) |
   | Governance / privileged access | P8 (2.0) |
   | Sovereign / regulated / multi-region | P9 (2.x) |

5. Consumer CIAM is served from P2 onward through the shared authentication engine. No separate B2C product exists.
6. Every segment uses the same binary, data model and APIs. Segment-specific code paths are forbidden except as modules.

## Rationale

B2B SaaS developers have the strongest current pull, need the broadest base (authn, OIDC, orgs) that every other segment reuses, and adopt open-source self-service. Agent identity is where the market is least served in 2026. Workforce and governance depend on federation and authorization foundations, so they come later.

## Alternatives considered

- **Workforce-first (Keycloak replacement):** needs SAML IdP, LDAP and Kerberos before anything else; slow to first value. Rejected as primary.
- **Consumer CIAM-first:** crowded, and a price-driven market. Rejected as primary.
- **Agent-only niche:** too narrow alone, and depends on a full OAuth stack anyway. Chosen as secondary.

## Consequences

- Positive: each phase serves a real buyer; the foundations get reused.
- Negative: workforce buyers wait until P7 for the complete legacy story.
- Risks: early enterprise prospects asking for SAML IdP. Mitigation: SAML SP and the broker arrive in P4. The IdP in P7 isn't deferred further.

## Fallback / revisit trigger

If ≥ 3 design-partner organizations commit to production on condition of a later-phase feature, a superseding ADR may pull that feature forward one phase. Removing features is never allowed.

## Implementation notes

- Docs IA (ADR-0048) is organized by segment, with "Getting started" written for the B2B developer.
- `examples/` contains one reference app per segment, added in the phase where that segment is fully served.

## Verification

Each phase's exit review confirms that the segment(s) listed for that phase have a working reference app in `examples/` and an end-to-end test in CI.
