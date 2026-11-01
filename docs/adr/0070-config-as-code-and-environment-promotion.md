# ADR-0070: Config-as-code and environment promotion

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §1.2 principle 16, §7.2 (Environment link), §13.5
- **Related:** ADR-0009, ADR-0020, ADR-0026, ADR-0071, ADR-0024

## Context

Enterprises need reproducible identity configuration across dev, staging and prod, with review, diff and rollback. Console-only settings cause drift and prevent GitOps.

## Decision

1. **Every** project setting (apps, clients, resource servers, connections, flows, policies, themes, templates, hooks, roles, ReBAC models, org templates) is representable in a declarative document: **Auvora Config Document** (`auvora.config/v1`). It is YAML/JSON and schema-published.
2. **Secrets are never exported in clear.** Exports use references: `secret_ref: env:NAME`, `secret_ref: kms:…`, or `secret_ref: vault:…`. Client secrets are generated per environment unless explicitly referenced.
3. **CLI:**
   - `auvora config export|diff|plan|apply|validate --project prj_…`.
   - `plan` shows a field-level diff.
   - `apply` is transactional (all-or-nothing) and optimistic-concurrency checked against the config version.
4. **Environments:** separate projects linked as an environment chain (`dev → staging → prod`).
   - Promotion copies the selected config with environment overrides (`overrides/<env>.yaml` for URLs, domains, secrets refs).
   - Promotion into a project marked `protected` requires approval by ≥ 1 Admin who is not the requester (configurable 1–3).
5. **Drift detection:** the declared state can be registered. A worker job compares it hourly and emits `auvora.config.drift_detected.v1`.
6. **Terraform provider** covers 100% of the Admin API (gated at P3). The Config Document and Terraform are equivalent; neither is privileged.
7. **Rollback:** any config version can be restored from entity history (ADR-0071). Restore is itself a new version.
8. Config writes from all surfaces (console, API, CLI, Terraform) produce the same audit events.

## Rationale

Declarative config makes reviews possible, enables GitOps, and is required by enterprises. It is also how competitors (Keycloak realm export, Zitadel, Auth0 deploy CLI) are used in practice, and Auvora unifies these.

## Alternatives considered

- **Terraform only**: excludes non-Terraform users and CI pipelines.
- **Raw DB dumps**: unsafe and not diffable.

## Consequences

- Positive: GitOps, review, reproducibility.
- Negative: every new feature needs schema, export and import support (enforced by the DoD, ADR-0107).
- Risks: secrets leakage in exports. Mitigation: a secret-ref-only serializer and tests.

## Fallback / revisit trigger

None.

## Implementation notes

- `auvora-config` owns the schema and the (de)serializers.
- Import/export land in P1 for core entities and extend per phase.
- The Terraform provider repo is `terraform-provider-auvora`.

## Verification

- Round-trip property test (export → apply to an empty project → export equals the original, minus IDs and secrets).
- Schema coverage test: every Admin API writable field appears in the Config Document.
- A secret-leak test scans exports.
