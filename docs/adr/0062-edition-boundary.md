# ADR-0062: Edition boundary (no SSO tax)

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §3.2, §18, §19
- **Related:** ADR-0003, ADR-0004, ADR-0063, ADR-0101

## Context

Open-core products must draw a line between the open edition and the commercial Enterprise Edition (EE). Gating SSO, MFA or standard protocols (the "SSO tax") damages trust and contradicts the security-first principle. EE still needs enough value to fund the project.

## Decision

**Rule:** anything that makes an identity *secure or interoperable* is open. EE contains capabilities that mainly matter to *large operators, governance programs and regulated scale*.

**Open edition (AGPL-3.0) always includes:**
- All authentication methods (password, passkeys, all MFA factors, push SDK, social, anonymous).
- All protocols: OAuth 2.x (all grants), OIDC, SAML IdP and SP, WS-Fed, SCIM server and client, LDAP sync, LDAP gateway, RADIUS, Kerberos, SSF/CAEP/RISC, AuthZEN, UMA.
- Organizations, Org Admin Portal, delegated admin, break-glass, RBAC/ABAC (Cedar)/ReBAC, explain.
- Machine, workload and agent identity, Token Vault, MCP.
- Supervised Support Access core (single approver, consent modes, ledger).
- Hosted UI, components, Flow API, hooks/CEL/Wasm, webhooks, Terraform.
- Audit with hash chain (90-day default retention), SIEM streaming, the Lifecycle Workflow engine, basic posture reports.
- Single-region HA, local keystore, AWS KMS / GCP KMS / Azure Key Vault / Vault Transit.

**EE (`ee/`, commercial license):**
- Governance: access requests, access packages, access reviews and certifications, SoD rules, the Lifecycle Workflow *designer*, HR connectors.
- JIT privileged elevation.
- Multi-approver support-access workflows.
- Multi-region cells, cross-cell control, project cell migration, dedicated and air-gapped packaging.
- PKCS#11 HSM, the FIPS-validated build.
- Audit retention > 90 days, WORM checkpoint export, compliance evidence packs.
- Advanced ITDR and posture analytics.
- Browser remote-access gateway.
- 12-month LTS, SLA support.

**Mechanics:**
- EE code lives only in `ee/`. Open code must compile and pass all tests without `ee/`.
- EE features are activated by a signed offline license file (`[license] file = …`). The license is verified locally with no phone-home (ADR-0041).
- Moving a feature from EE to open is allowed at any time. Moving open → EE is **forbidden**.

## Rationale

Avoiding an SSO tax is a differentiator against WorkOS/Auth0-style pricing. Governance and multi-region are where enterprise budgets live.

## Alternatives considered

- **SSO/SAML in EE**: rejected as the SSO tax.
- **Everything open + support only**: underfunds a multi-year roadmap.
- **Usage-gated open edition** (MAU caps): rejected; it harms self-hosters.

## Consequences

- Positive: trust, adoption, and clear messaging.
- Negative: a smaller EE surface, so Cloud must carry revenue (ADR-0063).
- Risks: contributors adding open features that depend on EE. Mitigation: CI builds and tests without `ee/`.

## Fallback / revisit trigger

Revisit only to move features *into* open. The open → EE direction is closed permanently.

## Implementation notes

The Cargo feature `ee` is enabled only from `ee/` crates. The license verifier is in `auvora-server`. `EDITIONS.md` publishes this table.

## Verification

- A CI job `build-oss` runs with `ee/` removed.
- A license-header lint enforces commercial headers inside `ee/` and AGPL/Apache headers outside it.
- Any PR that moves code from open → EE is blocked by CODEOWNERS.
