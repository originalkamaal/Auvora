# ADR-0027: Per-project identity scope

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D27, §7.3
- **Related:** ADR-0026, ADR-0034, ADR-0075, ADR-0088, ADR-0019

## Context

v0.2 asked for "one global human identity with multiple linked login accounts" and also "realm-isolated identity." A cross-tenant global identity would leak data between unrelated customers, complicate residency (ADR-0019) and create correlation risks. B2B SaaS still needs one person to belong to many customer organizations.

## Decision

1. **The user pool is per project.** A principal exists in exactly one project. The same email in two projects means two independent principals with no shared credentials, sessions or data.
2. **Within a project**, one human is one principal with:
   - many identifiers (ADR-0075),
   - many linked identities (social, enterprise, LDAP; ADR-0034),
   - many organization memberships with distinct roles and policies (ADR-0088).
3. **Across projects**, identity is shared only through explicit **federation**: project B configures project A as an OIDC/SAML connection. Auvora offers a one-click "Auvora project connection" type for this, which is still standard OIDC under the hood.
4. **Across environments** (linked projects), users are not synced. Test users are seeded per environment. Config, not identities, is promoted (ADR-0070).
5. **Subject identifiers:**
   - `sub` is an opaque per-project value derived from `principal_id`, never the email.
   - Pairwise `sub` per sector identifier is optional per client (`subject_type = "pairwise"`).
   - `sub` never reveals the tenant or project internals.
6. **No global search** across projects exists, except for the platform role (audited) for operational support (ADR-0057).
7. **Org-scoped identity:** organizations can enforce their own policies (MFA, IdP routing) but never get a separate user pool. A user's org memberships are visible only to the user, project admins, and admins of that org.

## Rationale

Per-project scope gives isolation, residency compliance and privacy by default, while the membership model covers B2B "one person, many customers" without a global identity.

## Alternatives considered

- **Global identity across all tenants**: privacy and residency violations, plus a cross-customer correlation risk. Rejected.
- **Per-organization user pools**: breaks B2B users belonging to many orgs. Rejected.

## Consequences

- Positive: strong isolation; simple residency; no cross-customer correlation.
- Negative: a developer with several products must federate projects or use one project with multiple applications.
- Risks and mitigations: docs give a decision guide ("one project, many apps" vs "federated projects").

## Fallback / revisit trigger

None.

## Implementation notes

P1. The uniqueness constraints in `auvora-store` all include `project_id`. Pairwise subject computation is in `auvora-oauth` (P5).

## Verification

Isolation property tests (cross-project reads must return nothing); an OIDF pairwise test in the Dynamic profile; a unit test that `sub` never equals any identifier value.
