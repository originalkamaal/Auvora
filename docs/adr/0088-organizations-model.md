# ADR-0088: Organizations model

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §7.1, §7.2 (Organization, Membership, Group, Role), §12.1, §12.4, §27 P4, §2.1 (Clerk/Auth0/Logto/WorkOS orgs)
- **Related:** ADR-0026, ADR-0027, ADR-0015, ADR-0089, ADR-0055, ADR-0099, ADR-0090, ADR-0029; amended by ADR-0114

## Context

B2B SaaS is the primary market. Developers need to model their customers (organizations), each with members, roles, SSO, SCIM, policies and branding. That has to happen inside one project (one user pool, ADR-0027), so a person can belong to many orgs with one account.

## Decision

1. An **Organization** belongs to one Project. Fields:
   - `org_` ID, `slug` (unique per project), `name`, `display_name`, `metadata` (public/private)
   - `parent_org_id` (nesting)
   - `status` (active/suspended/deleted)
   - `verified_domains[]`, `policies`, `branding_ref`, `connections[]`, `created_at`
2. **Nesting:** max depth 5 (configurable up to 10). Role inheritance down the tree is **opt-in per role** (`inherits_to_children`). Admins of a parent org may administer children only if granted `org:children:manage`.
3. **Membership:** principal ↔ org, with `roles[]`, `attributes` (typed per project schema), and `status` (invited/active/suspended/removed). Memberships apply to users, service accounts and agents. A user may have unlimited memberships, with a default cap of 1,000 per user for performance.
4. **Invitations:**
   - Email or link invitations with a role preset.
   - Expiry default 7 days; single-use.
   - Bound to the invited email unless `allow_any_email` is set.
   - Accepting requires sign-in or sign-up, and verifying the invited email.
5. **Verified domains:** DNS TXT `_auvora-verify.<domain>`. Uses:
   - Home realm discovery (ADR-0089 / plan §12.1).
   - Auto-join policy (`off` | `suggest` | `auto` for users with a verified email in the domain).
   - Domain capture (redirect matching users to the org's SSO).
   A domain can be verified by only one org per project.
6. **Per-org overrides** (applied over project defaults, most restrictive wins for security settings):
   - Allowed auth methods, MFA/AAL requirement, session lifetimes.
   - IdP routing, enforced SSO (`sso_required`), IP/network allowlist.
   - Third-party app policy (ADR-0085), support-access consent mode (ADR-0094).
   - Branding/theme (ADR-0068), email templates and sender.
7. **Org-scoped tokens:**
   - The authorization request can carry `organization=<org_id|slug>` (or org selection in the `org-select` flow).
   - Issued tokens include an `org` claim `{id, slug}` and org roles/permissions (size-limited; ADR-0029 claim budget, otherwise queried via AuthZEN).
   - Switching org issues a new token; no single token mixes orgs.
   - A project-level setting `org_required` forces org context for chosen apps.
8. **Org admins:** delegated administration through built-in org roles `org:owner`, `org:admin`, `org:member` (customizable), scoped strictly to their org subtree. They are separate from control-plane admin roles (ADR-0055).
9. **Groups** may be project-scoped or org-scoped. Dynamic groups use CEL rules (ADR-0016).
10. **Org lifecycle events:** `auvora.org.*`, `auvora.membership.*`. Deleting an org soft-deletes it, removes memberships after 30 days, and deletes org connections and SCIM tokens immediately.

## Rationale

This is the union of Auth0/Clerk/Logto organizations, WorkOS's org-scoped SSO/SCIM and Zitadel's org isolation, inside a shared user pool so that consultants and agencies can belong to many customers.

## Alternatives considered

- **Org = separate user pool** (Zitadel-style). Rejected: breaks multi-org users. Isolation is available via separate projects instead.
- **Flat orgs only.** Rejected: enterprises need divisions and subsidiaries.

## Consequences

- Positive: complete B2B model; enterprise-grade per-org security.
- Negative: claim size with many orgs is avoided by the one-org-per-token rule.

## Fallback / revisit trigger

If membership-cap complaints come from real customers, raise the cap after benchmark validation.

## Implementation notes

- Crates: `auvora-core`, `auvora-store`, `auvora-flows` (`org-select`), `auvora-oauth` (org claims), `auvora-authz` (org roles).
- Basic orgs (membership, invitation) ship in P1/P3. Full model ships in P4.

## Verification

- An isolation test suite (an org admin cannot read or modify a sibling org).
- Token tests (one org per token).
- A domain-verification uniqueness test.
