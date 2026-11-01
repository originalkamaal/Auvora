# ADR-0016: CEL for expressions

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D16, §8.3, §10, §13.1, §14, §20.1, §22
- **Related:** ADR-0015, ADR-0038, ADR-0067, ADR-0081, ADR-0087, ADR-0092, ADR-0097

## Context

Many features need tenant-authored logic:
- claim mapping (like Keycloak mappers);
- attribute transforms (JIT, SCIM, LDAP);
- flow and access-policy conditions (like Entra Conditional Access);
- dynamic groups, workload trust policies, risk rules;
- social provider manifests and registration form validators.

Running arbitrary code is dangerous and slow. Inventing a DSL fragments the product. Cedar is designed for authorization decisions, not value transformation.

## Decision

1. **CEL (Common Expression Language)** is the single expression language for conditions and transformations across Auvora. Cedar stays the language for authorization decisions (ADR-0015). Each language has a clear role:
   - **CEL:** "compute a value or a boolean condition"
   - **Cedar:** "is principal P permitted to do A on R"
2. **Runtime:** crate `auvora-cel`, based on the `cel-interpreter` crate. It is **forked or vendored if needed** to guarantee hardening (decided by the P0 spike). Required properties:
   - non-Turing-complete; no I/O; no host access except the provided bindings;
   - **cost limits:** a static cost estimate at compile time (reject if > `max_cost = 10_000`) and a runtime step budget (default 100,000 steps);
   - max expression source 8 KiB; max output size 64 KiB; max list comprehension size 10,000;
   - regex via the Rust `regex` crate only (linear-time, no backtracking, ReDoS-safe), with compiled-regex caching;
   - deterministic: no time functions except `request.time` bound by the host.
3. **Standard environments** (typed declarations per use site; type-checked at publish time):
   - `claims.mapping`: `user`, `identity` (upstream claims), `org`, `app`, `client`, `session` → output value.
   - `flow.condition`: `request` (ip, geo, ua, time), `risk`, `user`, `org`, `app`, `session.aal`, `device` → bool.
   - `access.policy`: same as above, plus `resource` → bool.
   - `workload.trust`: `token.claims`, `issuer` → bool + mapped principal.
   - `group.dynamic`: `user`, `memberships` → bool.
   - `scim.mapping`, `ldap.mapping`, `saml.attribute`, `form.validator`, `risk.rule`, `rebac.condition`.
4. **Extensions:** an Auvora function library (`email.domain()`, `ip.inCidr()`, `semver.compare()`, `strings.normalize()`, `time.between()`, `geo.distanceKm()`) with fixed costs.
5. **Authoring tooling:** Admin API endpoint `POST /v1/projects/{id}/cel/validate` (type-check + cost + test cases), console editor with autocomplete, `auvora cel eval` CLI, and test fixtures stored with the expression.
6. Expressions are stored as source plus a compiled AST hash. They are compiled into config snapshots (ADR-0009).

## Rationale

CEL is the Kubernetes/Envoy/Google IAM standard, safe by construction, fast, and familiar to platform engineers. One language across all extension points reduces learning cost and attack surface.

## Alternatives considered

- **JavaScript (Auth0 Actions style) in-process:** heavy, unsafe without strong sandboxing. Rejected for expressions (Wasm covers code use cases, ADR-0081).
- **Cedar for everything:** not designed for value transformation. Rejected.
- **JSONata/JMESPath:** weaker typing and less adoption for conditions. Rejected.
- **Custom DSL:** maintenance burden, no ecosystem. Rejected.

## Consequences

- Positive: safe, fast, typed, consistent.
- Negative: CEL is less expressive than general code. Complex logic needs hooks or Wasm (ADR-0038).
- Risks: interpreter bugs or cost-limit bypass. Mitigation: fuzzing, the step budget, and the ability to fork the interpreter.

## Fallback / revisit trigger

If the upstream `cel-interpreter` crate is unmaintained for > 6 months or fails conformance with cel-spec tests, Auvora maintains its own fork in `auvora-cel` (pre-agreed, no new ADR needed beyond recording the fork).

## Implementation notes

- P0: spike + hardening review. P1: claims mapping. P2: flow conditions. P4: access policies, JIT/SCIM mappings. P5: workload trust.

## Verification

- The cel-spec conformance tests run in CI.
- cargo-fuzz on parser and evaluator.
- A cost-limit regression suite (pathological expressions must be rejected or terminated).
