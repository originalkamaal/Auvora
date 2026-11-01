# ADR-0055: Control-plane admin roles

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D55, §3, §7.1, §18, §27 (P1)
- **Related:** ADR-0026, ADR-0015, ADR-0050, ADR-0057, ADR-0088, ADR-0094, ADR-0101

## Context

The control plane needs an authorization model from P1, before the full authorization platform (P6). Admins exist at several scopes: the installation/platform, the tenant, the project and the organization (delegated admins of developers' B2B customers).

## Decision

1. **Admin identities** are principals of a reserved **system project** (`prj_system`) per installation/cell. Auvora authenticates its own admins (dogfooding). Federation into the system project (e.g. company Okta/Entra) is supported.
2. **Scopes and built-in roles:**

| Scope | Roles |
|---|---|
| Platform (installation/cell) | `platform_superadmin`, `platform_operator` (ops, no data read), `platform_auditor`, `platform_support` (only via ADR-0094) |
| Tenant | `Owner`, `Admin`, `Developer`, `Support`, `Auditor`, `Billing` |
| Project | Same six roles, assignable per project (or per environment) |
| Organization (delegated) | `org_owner`, `org_admin`, `org_member_manager`, `org_sso_admin`, `org_auditor` |

3. **Built-in permission matrix** (summary; full matrix in `crates/auvora-api/permissions.toml`, which is generated into docs):
   - `Owner`: everything including deletion, billing and role management.
   - `Admin`: all config and identity management; no tenant deletion or billing.
   - `Developer`: apps, clients, resource servers, hooks, themes, dev/stg projects. Prod projects are read-only unless explicitly granted.
   - `Support`: read users, sessions and logs; reset MFA/password via a governed flow; start Supervised Support Access sessions if policy allows.
   - `Auditor`: read-only on config and audit; export. No PII beyond what audit holds.
   - `Billing`: invoices, plans, usage.
4. **Custom roles** (P4) are compositions of fine-grained permissions (`users.read`, `users.write`, `clients.secrets.rotate`, …), at tenant, project or org scope. They can't exceed the granting admin's permissions (no escalation).
5. **Assurance:** `platform_*` roles require AAL3 (ADR-0050). Tenant `Owner`/`Admin` require AAL2 phishing-resistant by default (relaxable to AAL2 per tenant; never below). Sensitive operations (key rotation/export, deletion, role grants to Owner, DCR open mode, disabling MFA policy) require re-auth within 5 min.
6. **Dual control:** legal hold, tenant deletion, platform key export and break-glass activation require two distinct principals.
7. **Break-glass:** ≥ 1 sealed emergency account per tenant (P4): a hardware key or offline credential, usable only via the break-glass flow, alerting all Owners and triggering an incident audit event.
8. **Implementation:** P1 uses native RBAC tables + an in-process permission check (`auvora-authz::admin`). From P6 the same rules are expressed as Cedar policies with the identical decision results (verified by a differential test).
9. **Admin API tokens:** admin API access uses OAuth tokens issued by the system project to admin users, service accounts or CI workloads (workload federation, ADR-0092). Static admin API keys are allowed only as scoped API keys (ADR-0093).

## Rationale

Clear roles from P1 prevent the "everyone is superadmin" anti-pattern. Dogfooding validates the product, and enforced AAL protects the most valuable accounts.

## Alternatives considered

- **Separate admin user store**: rejected because it duplicates authn features and isn't dogfooded.
- **Wait for P6 Cedar**: rejected because the control plane needs authz in P1.

## Consequences

- Positive: least privilege from day one; migration path to Cedar.
- Negative: two implementations during P1–P6 (mitigated by a differential test).

## Fallback / revisit trigger

None.

## Implementation notes

P1 built-in roles + AAL enforcement; P4 custom roles, org delegated admin, break-glass; P6 Cedar parity. Bootstrap: `auvora init` creates the system project and first Owner via a one-time setup token printed once to stdout (expires in 1 h).

## Verification

Permission matrix tests (every endpoint × role); escalation tests; differential tests between RBAC and Cedar in P6; bootstrap-token single-use test.
