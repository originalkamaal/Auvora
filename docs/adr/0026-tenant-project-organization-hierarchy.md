# ADR-0026: Tenant → Project → Organization hierarchy

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D26, §7.1, §7.2, §1.2 principle 17
- **Related:** ADR-0017, ADR-0018, ADR-0019, ADR-0027, ADR-0055, ADR-0070, ADR-0088, ADR-0090

## Context

The v0.2 plan used Realm, Tenant and Organization with overlapping meanings. This hierarchy decides table keys, token `iss`, URL scheme, key isolation, admin permissions, billing and cell placement, so it is the hardest decision to change later. Keycloak realms are heavyweight and scale poorly into the thousands. Auvora must support at least 100,000 lightweight projects per cell, plus B2B organizations inside each project.

## Decision

1. **Hierarchy:**
   - **Installation / Cloud**: the deployment.
   - **Tenant** (`tnt_`): an Auvora customer account. It owns billing, platform admins (ADR-0055), home region/cell (ADR-0019) and the projects.
   - **Project** (`prj_`): exactly one OAuth/OIDC issuer, one user pool, one key set, one configuration. It is the unit of isolation for identity data, RLS (ADR-0017), keys (ADR-0031) and cell placement.
   - **Organization** (`org_`): a B2B customer of the developer, inside a project. Organizations may nest up to **5 levels** (`parent_org_id`). Details are in ADR-0088.
2. **Environments** are separate projects linked into an **environment group** (`env_group_id`, with a `stage` label: `development | staging | production | custom`). Config promotion happens between linked projects (ADR-0070). Projects with `stage=development` get relaxed rules where documented (e.g. wildcard redirect URIs, ADR-0085), and the UI shows a dev banner.
3. **Issuer:**
   - Default `https://{project_slug}.{base_domain}` in Cloud, or `https://{host}/p/{project_slug}` in path mode for self-hosting (`[http] issuer_mode = "subdomain" | "path"`).
   - A verified custom domain overrides it (ADR-0090).
   - The issuer is immutable once the project has issued tokens, except through an explicit issuer-migration procedure that serves both issuers during a migration window.
4. **Everything identity-related belongs to exactly one project:** applications, clients, resource servers, principals, connections, policies, hooks, themes, keys, sessions and audit. Organizations belong to one project.
5. **Tenant-level objects** are only: billing, platform admin memberships, tenant settings, the project list and tenant audit.
6. **Terminology:** the product, API and UI use *Tenant*, *Project* and *Organization*. "Realm" appears only in migration docs as "Keycloak realm ≈ Auvora project."
7. **Self-hosted installs** have one implicit tenant by default (`[tenancy] multi_tenant = false`). Multi-tenant mode is enabled for IDaaS operators.
8. **Limits (defaults, configurable):**
   - projects per tenant: 1,000;
   - organizations per project: unlimited (tested to 1M);
   - org nesting depth: 5.

## Rationale

- Projects as rows (not runtimes) make massive multi-tenancy cheap, matching Zitadel's strength while keeping Keycloak's realm semantics.
- Environment-as-project gives hard isolation between dev and prod keys and users while still allowing promotion.
- Organizations inside projects model B2B SaaS (Auth0/Clerk/WorkOS style) without creating new issuers per customer.

## Alternatives considered

- **Realm = Tenant (Keycloak style)**: conflates billing and issuer; doesn't scale. Rejected.
- **Organization as issuer**: breaks SaaS apps that need one issuer across customers. Rejected; per-org SSO is done via connections instead.
- **Environments as a sub-layer inside a project**: shared keys/users between dev and prod is a security risk. Rejected.

## Consequences

- Positive: a clear model; cheap projects; clean isolation and residency.
- Negative: cross-environment config sync needs tooling (ADR-0070).
- Risks and mitigations: customers needing users across projects use federation between projects (ADR-0027).

## Fallback / revisit trigger

None. This is foundational and changes require a major version.

## Implementation notes

P0 schema design; P1 implementation of tenants, projects and environment groups; P4 full organizations. Crates: `auvora-core` (types), `auvora-store`, `auvora-config`.

## Verification

Isolation property tests (ADR-0017); a schema lint requiring `project_id` on every identity table; load test creating 100k projects in a cell with no latency regression for unrelated projects.
