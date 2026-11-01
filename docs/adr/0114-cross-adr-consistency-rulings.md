# ADR-0114: Cross-ADR consistency rulings

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §7, §3.2
- **Related:** amends ADR-0055, ADR-0063, ADR-0077, ADR-0088; see also ADR-0018, ADR-0023, ADR-0111

## Context

Writing the P0 specs (`docs/specs/domain-model.md`, `tenancy-rls.md`, `cells.md`) uncovered places where Accepted ADRs disagree. Pure typos and broken links were fixed in place, as ADR-0000 allows (ADR-0017 allowlist path, ADR-0026 tenant prefix `ten_`, ADR-0039 key class `audit_checkpoint_signing`). Three disagreements change a decision, so they are ruled on here.

## Decision

### R1: Organization role names (amends ADR-0055 and ADR-0088)

- Built-in organization roles use the **namespaced form `org:<name>`**, matching permission strings (`resource:action`).
- The built-in set is the union of both ADRs: `org:owner`, `org:admin`, `org:member`, `org:member_manager`, `org:sso_admin`, `org:auditor`.
- ADR-0055's `org_owner`-style names are replaced by these. Custom org roles use `org:custom:<slug>`.

### R2: Anonymous users and the OIDC `sub` (amends ADR-0077)

- An anonymous principal's **admin/API identifier** keeps the `anon_` prefix (ADR-0018).
- Its **OIDC `sub` is always minted as `usr_` + the same UUID**. When the anonymous user upgrades to a full user, the principal's kind changes, but `sub` doesn't. This preserves ADR-0018's rule that `sub` never changes.
- Relying parties therefore never see `anon_` in tokens. They detect anonymous users via the `amr` claim value `anon` and the `auvora_principal_kind` claim.

### R3: Auvora Cloud code location (amends ADR-0063)

The owner directive is that everything lives in this single repository (ADR-0023, ADR-0111), so there is **no separate private `cloud/` repository**:

- Cloud-only services (signup, metering, billing integration) live in `ee/crates/auvora-cloud-*` under the commercial license. They still integrate only through public APIs and the global control plane (ADR-0019).
- Cloud environment configuration, credentials and infrastructure secrets are **never committed to any repository**. They live in the Cloud's secret manager and deployment pipeline.
- This repository is public, so nothing in `ee/crates/auvora-cloud-*` may rely on obscurity for security.

## Rationale

- R1: the namespaced form avoids collisions with project and application role names, and matches how permissions are expressed.
- R2: preserves the immutable-`sub` guarantee, which relying parties depend on for account keys.
- R3: honours the single-repository directive while keeping secrets out of version control.

## Alternatives considered

- **R1, snake_case `org_owner`:** collides visually with project roles and doesn't namespace. Rejected.
- **R2, change `sub` on upgrade:** breaks every relying party's account linkage. Rejected.
- **R3, private repo for Cloud:** contradicts the owner directive. Rejected.

## Consequences

- Positive: specs and ADRs agree; no hidden second repository.
- Negative: four ADRs must be read together with this one, which is mitigated by the "amended by ADR-0114" notes on each.
- Risks and mitigations: Cloud code in a public repo could leak operational detail. Mitigated by the no-secrets rule and code review.

## Fallback / revisit trigger

None for R1 and R2. For R3: if Cloud operations require code that can't be public (e.g. fraud heuristics), write a superseding ADR that defines a minimal private companion repository for that code only.

## Implementation notes

- Add "amended by ADR-0114" to the Related lines of ADR-0055, ADR-0063, ADR-0077 and ADR-0088.
- `docs/specs/domain-model.md` and `tenancy-rls.md` use R1 and R2.

## Verification

- R1: enforced by the role registry tests in `auvora-authz` (P4).
- R2: covered by an upgrade test asserting `sub` stability (P2).
- R3: `scripts/check-repo-layout` plus gitleaks secret scanning in CI.
