# ADR-0015: Layered authorization architecture

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D15, §20, §2.1, §2.2 (#3)
- **Related:** ADR-0016, ADR-0006, ADR-0029, ADR-0055, ADR-0084, ADR-0094, ADR-0101

## Context

The market covers authorization in pieces:
- Keycloak: Authorization Services, UMA
- Ory Keto, OpenFGA, SpiceDB: Zanzibar ReBAC
- Cedar and OPA: policy-based authorization
- Auth0 and WorkOS: FGA products

v0.2 listed Cedar plus "SpiceDB or OpenFGA," which means two to three external engines, an unresolved choice, and extra services. The owner's stance requires full RBAC + ABAC + ReBAC coverage that runs embedded in the minimum profile.

## Decision

Authorization is a stack of five layers in **`auvora-authz`**, all embedded:

1. **Scopes + RAR (RFC 9396):** coarse API delegation in tokens (ADR-0013, ADR-0029).
2. **Native RBAC** (P4):
   - project-, org- and app-scoped roles and permissions, stored in Postgres;
   - composite roles (max depth 5) and conditional assignments (CEL, ADR-0016);
   - emission into tokens is optional and size-budgeted (default ≤ 50 role/permission values, else omitted with an `authz_ref` claim pointing to the AuthZEN API).
3. **Cedar** (embedded `cedar-policy` crate, P6):
   - ABAC/PBAC. The Cedar schema is generated from the Auvora entity model (principals, orgs, groups, roles, apps, resources) plus customer-defined entity types.
   - Policies are versioned, validated against the schema at publish time, and evaluated with entities loaded from the snapshot (ADR-0009) and request context.
   - Shadow mode (evaluate a new version without enforcing it; log diffs).
4. **Embedded ReBAC engine "Auvora Graph"** (P6):
   - Zanzibar model, **OpenFGA DSL 1.1 compatible** (import/export, same semantics for `define`, `or`, `and`, `but not`, `from`, conditions).
   - Tuples in Postgres (`rebac_tuples`, partitioned by project), with check / list-objects / list-users / expand / read / watch APIs.
   - Contextual tuples; conditions via CEL.
   - Consistency: every write returns a **consistency token** (Postgres LSN + project write sequence). Checks accept `minimize_latency` (cache) | `at_least_as_fresh(token)` | `fully_consistent`.
   - Leopard-style precomputed group-membership index for deep or nested groups (P6.x, enabled when depth > 3).
   - Check cache with invalidation from the outbox.
5. **AuthZEN Authorization API 1.0** (P6): the **public PDP interface** for evaluation, batch evaluations, and subject/resource/action search. A decision fans out across layers 2–4 with a combining rule (below). The same API is available embedded in SDKs (local evaluation from synced snapshots) for latency-critical PEPs.

**Combining rule (normative):** a request is permitted iff:
- no applicable Cedar `forbid` matches, **and**
- at least one layer permits: an RBAC permission, a Cedar `permit`, or a ReBAC relation, as configured per resource type in the **authorization model** (`resource_types.<type>.sources = ["rbac","cedar","rebac"]`).

Support-session restrictions (ADR-0094) and token scopes are intersected afterwards.

**Explain:** every decision can return a trace (matched roles, policies by ID, and the tuple path). Traces are logged when `explain=true` or when the decision is a deny in shadow mode.

**External adapters (optional, P6+):**
- OpenFGA and SpiceDB: tuple sync-out, or delegate checks per resource type;
- OPA: delegate a policy decision via its REST API.
They are never required.

**UMA 2.0** (P6) and generic **Entities** are built on this stack.

## Rationale

Embedding Cedar (a Rust-native library) and a native ReBAC engine meets ADR-0006, avoids extra services, and lets every decision be explained across layers. That is a USP no single competitor offers. OpenFGA-DSL compatibility removes lock-in, and AuthZEN keeps the API standard.

## Alternatives considered

- **External SpiceDB or OpenFGA required:** an extra Go service; violates ADR-0006. Rejected as a requirement; kept as adapters.
- **OPA/Rego as the main policy language:** Go runtime, harder static analysis. Rejected; Cedar is analyzable and embeddable.
- **RBAC only:** fails the superset requirement. Rejected.

## Consequences

- Positive: one engine for all models; explainability; standard API; no extra infrastructure.
- Negative: building and maintaining a Zanzibar engine is substantial work.
- Risks: ReBAC performance at scale. Mitigation: Leopard indexing, caching, benchmarks (ADR-0105: p99 < 5 ms check), and external adapters as an escape hatch.

## Fallback / revisit trigger

If the embedded ReBAC engine fails the ADR-0105 check-latency target at 10× the reference tuple volume after optimization, a superseding ADR makes the external OpenFGA adapter the recommended backend for the Scale profile. The embedded engine stays for the minimum profile.

## Implementation notes

- `auvora-authz::{rbac, cedar, graph, authzen, explain, combine}`. CLI: `auvora authz test`, `auvora authz model import --openfga`.
- Admin's own permission model (ADR-0055) is implemented with this same engine from P1 (RBAC subset).

## Verification

- An OpenFGA model compatibility test-suite runs against Auvora Graph.
- Cedar schema validation runs in CI for built-in policies.
- AuthZEN interop tests.
- Property tests: combining-rule invariants (any `forbid` ⇒ deny).
