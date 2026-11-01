# Auvora
## Unified Identity, Access and Authorization Platform: Product, Architecture and Engineering Plan

**Version:** 1.0 (supersedes v0.2, archived at `docs/product/archive/plan-v0.2.md`)
**Status:** Decisions locked. The ADRs in `docs/adr/` are authoritative and refine this plan; changes go through new ADRs (ADR-0000).
**Date:** 1 November 2026
**Inputs:** v0.2 plan, review `docs/product/reviews/2026-11-01-plan-v0.2-review.md`, owner answers (Q1–Q5, Q11, Q18–Q19, Q41–Q42). Every other review question uses the recommended answer. See the Decision Register (§4).

---

# 1. Vision and USP

**Auvora is one identity platform that does what the rest of the market does in pieces, and does each piece in the standards-correct, secure way.**

Today a company typically assembles:
- Keycloak for SAML/LDAP/Kerberos,
- Ory or Auth0 for developer-friendly CIAM,
- WorkOS for enterprise SSO onboarding,
- OpenFGA/SpiceDB for fine-grained authorization,
- authentik/Authelia for a proxy, LDAP or RADIUS outpost,
- Entra/Okta for governance and conditional access,
- a separate tool for workload and AI-agent credentials.

Auvora replaces that stack with **one Rust binary + PostgreSQL** that scales from a laptop to multi-region cells.

## 1.1 The five pillars of the USP

| # | Pillar | Promise |
|---|---|---|
| U1 | **Superset coverage** | Every capability that a major IAM product is known for (§2) has a first-class Auvora equivalent. Insecure legacy mechanisms are never copied; the *need* behind them is met with a secure equivalent. |
| U2 | **One binary, zero mandatory dependencies beyond PostgreSQL** | Authentication, OAuth/OIDC, SAML, SCIM, RBAC+ABAC+ReBAC authorization, events, hooks, LDAP/RADIUS gateways and proxy all run in one process. Each is a role you can split out horizontally. Optional add-ons (Valkey, NATS, KMS/HSM, OpenSearch, ClickHouse) improve scale; they are never required for features. |
| U3 | **Human + machine + agent identity in one model** | Users, service accounts, workloads and AI agents share one principal model, one audit trail, one policy language and one delegation model (token exchange with actor chains). |
| U4 | **Every integration style from one engine** | Hosted UI, drop-in components, headless Flow API and fully self-hosted UI are four front-ends over **one** tested flow state machine. There's no second, weaker code path. |
| U5 | **Provable security** | OIDF-certified, fuzzed parsers, hash-chained audit, explainable authorization, supervised (never silent) support access, and published conformance results per release. |

## 1.2 Product principles

The 12 principles from v0.2 are retained. Added:

13. **Meet the need, not the anti-pattern.** If a competitor offers an insecure mechanism (ROPC, Implicit, silent impersonation, arbitrary JS in the login origin), Auvora provides a secure mechanism covering the same use case (§2.3).
14. **Embedded first, external optional.** Every subsystem has an embedded implementation. External engines are adapters, never prerequisites.
15. **One engine, many surfaces.** UI, SDK, API, CLI, Terraform and MCP are all clients of the same public APIs. Nothing is console-only.
16. **Everything is configuration-as-code.** Every setting is exportable, diffable, versioned and promotable across environments.
17. **Multi-tenancy is cheap.** A project is a few rows, not a heavyweight runtime object. One cell supports at least 100,000 projects.

---

# 2. Market capability coverage (superset matrix)

> The middle column lists what each product is *known for*, to make sure Auvora covers it. Re-verify against current vendor docs before using it in external marketing.

## 2.1 Capability → reference product → Auvora answer

| Capability | Known strength of | Auvora answer | Phase |
|---|---|---|---|
| Realms, deep admin console | Keycloak | Tenant → Project → Organization hierarchy. Projects are lightweight (100k+/cell). Full console + API parity | P1/P3 |
| SAML IdP **and** SP, identity brokering | Keycloak, authentik | SAML SP (P4), SAML IdP (P7), any-to-any protocol bridging broker | P4/P7 |
| LDAP/AD user federation, Kerberos/SPNEGO | Keycloak | LDAP/AD sync and pass-through bind, Kerberos/SPNEGO desktop SSO | P7 |
| UMA 2.0, authorization services | Keycloak | UMA 2.0 grant + resource registration on top of the native authz engine | P6 |
| SPI extensibility | Keycloak | 4-tier extensibility: webhooks, sync HTTP hooks, CEL expressions, Wasm actions + compile-time Rust module trait | P3/P8 |
| FAPI, CIBA, client policies | Keycloak | FAPI 2.0 profiles, CIBA, client policy profiles (enforced rules per client class) | P5/P10 |
| Event-sourced history, massive multi-tenancy | Zitadel | Versioned entity history (time travel of any identity object) + hash-chained audit; projects as rows | P1 |
| Actions/targets | Zitadel, Auth0 | Hooks + CEL + Wasm | P3/P8 |
| Headless flow API | Ory Kratos | Flow API is the core primitive (§13) | P1–P3 |
| Certified high-scale OAuth server | Ory Hydra | Native Rust OP, OIDF-certified, stateless data plane | P1–P5 |
| Zanzibar ReBAC | Ory Keto, OpenFGA, SpiceDB | **Embedded** ReBAC engine with OpenFGA-DSL compatibility + consistency tokens; external adapters optional | P6 |
| Identity-aware proxy / forward auth | Ory Oathkeeper, Authelia, authentik | Auvora Access Proxy role (reverse proxy + forward-auth endpoint for nginx/Traefik/Caddy/Envoy) | P7 |
| Visual flows and stages | authentik, Descope | Declarative flows (P2) → visual Flow Designer over the same engine | P8 |
| LDAP server outpost, RADIUS server | authentik, Kanidm | LDAP gateway (read + bind) and RADIUS server roles backed by Auvora identities | P7 |
| Unix/SSH integration | Kanidm, Teleport | SSH certificate authority (short-lived certs from OIDC login), NSS/PAM module | P10 |
| Remote access (browser RDP/SSH) | authentik RAC | Access Gateway module (optional, EE) | P10 |
| Beautiful CIAM UI, organizations, M2M, agent auth | Logto | Themes, drop-in components, orgs, M2M, MCP auth | P3–P5 |
| Lambdas, advanced registration forms, entities | FusionAuth | Hooks/CEL; schema-driven registration forms (multi-step, conditional fields); generic Entities via ReBAC | P3/P6 |
| Session recipes, anti-CSRF | SuperTokens | Server-side sessions, `__Host-` cookies, DBSC, token-binding SDK helpers | P1/P5 |
| Admin portal for customer IT (SSO + SCIM), Audit Logs API, FGA | WorkOS | Self-service Org Admin Portal; **Audit Log API for customers' own app events**; native FGA | P4/P6 |
| Drop-in components, impersonation, B2B orgs | Clerk | React + Web Components; **Supervised Support Access** (impersonation done right) | P3/P6 |
| Actions, Universal Login, bot detection, breached passwords, Organizations, FGA, AI agents (token vault, async auth) | Auth0 | All covered: hooks, hosted UI, risk + bot protection, breach screening, orgs, native FGA, **Token Vault** + CIBA for agents | P2–P6 |
| Workflows, lifecycle management, Universal Directory | Okta | Lifecycle Workflows (JML automation, no-code steps + hooks), attribute-sourcing precedence | P8 |
| Conditional Access, PIM, entitlement management, access reviews, workload identity federation | Microsoft Entra | Access Policies (CEL conditions), JIT privileged elevation, access packages, reviews, workload federation | P4–P8 |
| Identity threat detection (ITDR), posture | Okta ITP, Entra ID Protection | Risk engine + SSF/CAEP + identity posture reports | P5/P8 |
| Anonymous/guest users, phone auth | Firebase, Supabase | Anonymous principals with secure upgrade/merge; phone + SMS/WhatsApp/voice OTP | P2 |
| Postgres RLS integration | Supabase | `auvora-pg` helper: JWT-claim → Postgres session vars for app-level RLS | P5 |
| Device fingerprinting/fraud | Stytch | Privacy-conscious device intelligence signals in the risk engine | P5 |
| Lightweight Rust OIDC, embedded HA | Rauthy, Kanidm | Rust single binary; Postgres-only HA | P0+ |

## 2.2 Where Auvora goes beyond everyone

1. **Supervised Support Access**: delegated, scoped, consented, visible, audited support sessions with `act` chains (§16). Nobody does this properly.
2. **Agent identity as a first-class principal**: owner, delegation chain, per-tool consent, CIBA approval, Token Vault, MCP-native (§11).
3. **One policy story**: CEL for conditions and transformations, Cedar for authorization decisions, an embedded ReBAC graph, all exposed through the **AuthZEN** standard API, with explain/trace for every decision.
4. **Identity time travel**: "what did this user/org/client look like, and who could access what, at time T?"
5. **Conformance-as-a-feature**: each release publishes OIDF, SAML, SCIM and WebAuthn interop results.

## 2.3 Meet the need, not the anti-pattern

| Legacy mechanism others offer | Auvora's secure equivalent |
|---|---|
| ROPC (password grant) for native apps | OAuth for First-Party Applications (native credential UX without ROPC), Native SSO |
| ROPC for legacy migrations | Lazy migration hook + password-hash import |
| Implicit grant for SPAs | Code + PKCE + DPoP; BFF SDK pattern |
| Silent impersonation | Supervised Support Access |
| Arbitrary JS in the login page | Drop-in components in *your* origin; sandboxed Wasm actions server-side; theme tokens + slots |
| Long-lived API keys | Short-lived tokens via workload federation. Where API keys are unavoidable: scoped, hashed, expiring, rotatable, prefixed and secret-scannable |

## 2.4 Explicit non-goals (with reason)

- Endpoint/MDM device management: a different product category. Integrate device posture signals via SSF/APIs instead.
- Password manager / secrets vault for end users: a different category.
- Privileged session *recording* (keystroke/video): out of scope. JIT elevation and audit are in scope.

---

# 3. Product domains and editions

## 3.1 Domains

| Domain | Covered by |
|---|---|
| CIAM (B2C) | §8 authentication, §13 UX, §14 extensibility, §15 comms |
| B2B SaaS identity | §7 orgs, §12 enterprise SSO, §17 provisioning, Org Admin Portal |
| Workforce IAM | §12 federation, §18 lifecycle/IGA, §19 legacy gateways |
| Machine, workload and agent identity | §10, §11 |
| Authorization (FGA) | §20 |
| Governance and privileged access | §18 |

## 3.2 Licensing and editions (locked)

| Component | License |
|---|---|
| Server (`crates/`, `apps/`) | **AGPL-3.0** |
| SDKs, client libraries, Terraform provider, Helm charts | **Apache-2.0** |
| `ee/` directory | Auvora Commercial License |
| Contributions | **CLA** (required for dual licensing) |

**Edition boundary (no SSO tax):** every protocol and every authentication method is open: OIDC, OAuth (all grants), SAML IdP/SP, SCIM, LDAP, RADIUS, passkeys, MFA, orgs, RBAC/ABAC/ReBAC and Supervised Support Access core.

**EE** contains capabilities that matter to large operators:
- governance (access reviews, certifications, SoD, entitlement packages, lifecycle workflows designer),
- JIT privileged elevation,
- multi-approver support-access workflows,
- multi-region cells and dedicated deployments,
- HSM/PKCS#11, FIPS-validated build,
- audit retention > 90 days + WORM export, compliance report packs,
- browser remote-access gateway,
- SLA support and 12-month LTS.

**Auvora Cloud:** operated SaaS. Private beta after GA 1.0. Launch regions: EU (Frankfurt), US (Virginia), India (Mumbai), one cell each.

---

# 4. Decision register (all locked)

Each row becomes an ADR in `docs/adr/` during P0. "Fallback trigger" defines when the decision is revisited, so revisiting is planned, not an open question.

| ID | Decision | Choice | Fallback trigger |
|---|---|---|---|
| D01 | Team/timeline | Multi-year, quality over speed; phases are gated by exit criteria, not dates | — |
| D02 | Market | B2B SaaS developers (primary) + agent/workload (secondary) + enterprise B2B + workforce + CIAM, sequenced in §27 | — |
| D03 | Business model | Open-core + Auvora Cloud | — |
| D04 | License | AGPL server, Apache SDKs, commercial `ee/`, CLA | — |
| D05 | Deployment unit | Modular monolith, single binary `auvora`, roles: `all, edge, data, control, worker, ldap, radius, proxy, scim` + optional sidecar role `node` (not in `all`) | Split a role into its own service only when profiling proves contention |
| D06 | Minimum profile | `auvora` + PostgreSQL. **All features work** in this profile | — |
| D07 | Cache | Valkey optional; Postgres fallback for rate limits, replay, sessions (UNLOGGED tables + advisory locks) | — |
| D08 | Events | Transactional outbox + embedded workers; NATS JetStream optional bus; Kafka/Kinesis/PubSub as sinks | — |
| D09 | CP→DP config | Shared DB + versioned in-memory config cache, invalidated via LISTEN/NOTIFY; signed config snapshots for cross-cell | Propagation SLO breach (> 5 s p99) |
| D10 | Edge | No bundled gateway; built-in TLS (rustls), ACME, rate limiting; reference configs for nginx/Envoy/Traefik/Caddy/Cloudflare | — |
| D11 | Key custody | `KeyProvider` trait: local encrypted keystore, AWS KMS, GCP KMS, Azure Key Vault, Vault/OpenBao Transit, PKCS#11 HSM | — |
| D12 | Crypto | **aws-lc-rs** everywhere (rustls, JOSE, WebAuthn, X.509); FIPS build flag; PQ hybrid TLS on | — |
| D13 | OAuth/OIDC engine | Own Rust engine `auvora-oauth`, spec-driven, OIDF suite in CI from P1 | — |
| D14 | SAML | `samael` + libxmlsec1 in isolated crate `auvora-saml`, strict profile, fuzzed, run in a separate sandboxed worker process | > 2 unresolved interop failures with top-10 IdPs, or a critical CVE pattern → JVM sidecar (Shibboleth OpenSAML) as alternate backend behind the same trait |
| D15 | Authorization | Native RBAC → Cedar (embedded) → **embedded ReBAC engine** (OpenFGA DSL compatible) → AuthZEN API; external OpenFGA/SpiceDB adapters optional | — |
| D16 | Expressions | **CEL** (non-Turing-complete) for conditions, claim mapping, attribute transforms, flow rules | — |
| D17 | Tenancy isolation | Shared tables + `tenant_id`/`project_id` + Postgres **RLS** enforced via per-transaction role; dedicated DB/deployment for regulated tenants | — |
| D18 | IDs | UUIDv7 internal; prefixed external IDs (`usr_`, `org_`, `prj_`, `app_`, `cli_`, `ses_`, `sa_`, `agt_`, `wl_`…); `sub` opaque per project; pairwise optional | — |
| D19 | Multi-region | Cell architecture; tenant/project pinned to a home cell; global router maps host/issuer → cell; no active-active writes | — |
| D20 | API | REST/JSON, OpenAPI 3.1, `/v1`, additive-only, RFC 9457 errors, cursor pagination, `Idempotency-Key`, ETag/If-Match; gRPC only internally if roles are split | — |
| D21 | UI architecture | Flow API state machine; hosted UI = static React SPA served by the binary; drop-in React + Web Components; account portal; admin console | — |
| D22 | Frontend stack | React + TypeScript + Vite + TanStack Router/Query + Radix + Tailwind + design tokens; Playwright | — |
| D23 | Repo | Single monorepo for **everything** (incl. all SDKs, mobile app, integrations, docs); CI-generated read-only mirrors only where an ecosystem requires a separate repo (ADR-0023, ADR-0111) | — |
| D24 | SDK strategy | Admin SDKs generated from OpenAPI; runtime SDKs hand-written and thin; native Swift/Kotlin hand-written | — |
| D25 | Migrations | `sqlx migrate`, expand/contract only, N-1 compatible | — |
| D26 | Hierarchy | Tenant → Project → Organization (§7). "Realm" is a docs alias for Project | — |
| D27 | Identity scope | Per-project user pools; no cross-project global identity except via federation | — |
| D28 | Password policy | NIST SP 800-63-4; Argon2id (m=19 MiB, t=2, p=1 baseline, tunable); hashing semaphore; HIBP k-anon + offline list | — |
| D29 | Access tokens | JWT (RFC 9068) 10 min TTL default (M2M client_credentials 1 h, agents 5 min); opaque per resource server; audience via RFC 8707 | — |
| D30 | Refresh tokens | Rotation + family revocation on reuse (public clients); sender-constrained (DPoP/mTLS) for confidential; 30 d idle / 90 d absolute | — |
| D31 | Signing | ES256 default; RS256, PS256, EdDSA supported; per-project keys; auto-rotate 90 d, overlap 2 keys; ML-DSA when JOSE registrations finalize | — |
| D32 | Sessions | Server-side; `__Host-` cookies; Postgres authoritative + optional Valkey; CIAM 14 d idle/30 d absolute, workforce 8 h/12 h; DBSC | — |
| D33 | Revocation SLO | ≤ 5 s p99 in-region for sessions/introspection/config; AT ≤ 10 min; SSF/CAEP push | — |
| D34 | Account linking | Never silent; proof of control; per-IdP "trusted verified email" opt-in | — |
| D35 | Enumeration | Resistant by default (uniform responses + timing); per-project opt-out | — |
| D36 | Comms | Email: SMTP, SES, Postmark, SendGrid, Resend, webhook. SMS/voice: Twilio, Vonage, MessageBird, AWS SNS, webhook. WhatsApp: Meta Cloud API, Twilio | — |
| D37 | Bot protection | Turnstile, hCaptcha, reCAPTCHA Enterprise; risk-triggered; built-in proof-of-work fallback (no third party) | — |
| D38 | Extensibility | Webhooks (P3) + sync HTTP hooks (P3) + CEL (P2) + Wasm actions via wasmtime (P8) + compile-time Rust module trait (P3) | — |
| D39 | Audit | Append-only partitioned table, per-tenant hash chain, signed checkpoints → WORM object storage, SIEM streaming | — |
| D40 | Observability | OpenTelemetry (traces/metrics/logs) + Prometheus endpoint; Grafana dashboards shipped as artifacts | — |
| D41 | Telemetry | None by default; opt-in anonymous | — |
| D42 | Search | Postgres FTS + pg_trgm default; OpenSearch optional adapter | — |
| D43 | Analytics | Postgres rollups default; ClickHouse sink optional | — |
| D44 | Platforms | PostgreSQL 16+; Linux amd64/arm64; macOS dev; Kubernetes N-2; Compose + Helm + Terraform modules; no Ansible | — |
| D45 | Release | SemVer; monthly minor; critical security fix ≤ 7 days; zero-downtime N-1→N; EE LTS 12 months | — |
| D46 | CI/supply chain | GitHub Actions; cosign/Sigstore; SLSA L3; CycloneDX SBOM; cargo-deny/audit/vet; Renovate; reproducible builds | — |
| D47 | Rust standards | `forbid(unsafe_code)` except FFI crates; clippy pedantic; MSRV = stable − 2; cargo-fuzz for every parser | — |
| D48 | Docs | Astro Starlight, versioned; OpenAPI reference; runnable examples tested in CI | — |
| D49 | Threat modelling | STRIDE + LINDDUN per feature, in-repo, required by PR template for security-labelled changes | — |
| D50 | Assurance | NIST AAL1–3/IAL/FAL mapping → `acr`/`amr` (RFC 8176) | — |
| D51 | OIDF certification | OP Basic, Implicit-free Config, Dynamic, Form Post, RP-Initiated/Back-Channel/Front-Channel Logout by GA; FAPI 2.0 + CIBA post-GA | — |
| D52 | DCR | Supported; off by default; IAT/software-statement/open policy; Client ID Metadata Documents | — |
| D53 | Consent | First-party skip; third-party always; scope catalog per resource server; revocable grants | — |
| D54 | Deletion | Soft delete 30 d → hard delete + PII purge; audit keeps pseudonymous ID; legal hold | — |
| D55 | Admin roles | Owner, Admin, Developer, Support, Auditor, Billing + custom roles; platform super-admin AAL3-only | — |
| D56 | Push MFA | Push SDK embeddable in customer apps (P7) **and** the Auvora Authenticator app (P7) | — |
| D57 | Cloud staff access | Only via Supervised Support Access, customer-visible | — |
| D58 | Migration importers | Auth0, Keycloak, Firebase, Cognito, Clerk, Supabase, Okta, Zitadel, Ory | — |
| D59 | Compliance | FIPS-capable from day one; SOC 2 Type II + ISO 27001 at Cloud launch; HIPAA, PCI-ready docs; FedRAMP-ready architecture | — |
| D60 | Interop targets | Okta, Entra ID, Google Workspace, Keycloak, ADFS, Ping, OneLogin, JumpCloud, authentik, Zitadel; SCIM clients from Okta/Entra/OneLogin | — |
| D61+ | Further decisions | Recorded as ADR-0061 … ADR-0112 (index: `docs/adr/README.md`), incl. repository organization (ADR-0111) and commit policy (ADR-0112) | — |

---

# 5. Architecture

## 5.1 Shape: modular monolith, role-split at scale

```
                ┌─────────────────────────── auvora (single binary) ───────────────────────────┐
 Browsers/Apps ─┤ edge: TLS, ACME, routing by host→project, rate limit, bot checks             │
 Services/Agents┤ data: Flow API, OAuth/OIDC, SAML, sessions, tokens, introspection, AuthZEN     │
 Admins/CI    ──┤ control: Admin API, console, config, keys, policy publish                      │
 Legacy        ─┤ ldap / radius / proxy / scim: protocol gateways                               │
                │ worker: outbox, webhooks, comms, SCIM client, sync, rotation, retention, ACME  │
                └──────┬──────────────┬────────────────┬─────────────────┬────────────────────┘
                       │              │                │                 │
                  PostgreSQL     [Valkey]          [NATS/Kafka sinks] [KMS/HSM]
                  (required)     (optional)        (optional)         (optional; local keystore default)
```

- `auvora serve --roles=all` on a laptop, 3 replicas for HA, or each role as its own Deployment in Kubernetes.
- Roles share crates and code; separating them is a deployment choice, not a rewrite.
- The data plane never calls the control plane synchronously. It reads config from versioned caches (D09).

## 5.2 Deployment profiles (same features in each)

| Profile | Components | Use |
|---|---|---|
| Dev | `auvora dev` (embedded Postgres via `postgresql_embedded` or Docker) | Local development, tests |
| Small | 1–3 × `auvora` + Postgres (managed or Patroni) | Self-hosted SMB |
| Scale | Role-split Deployments + Postgres HA + Valkey + NATS + KMS | Large self-hosted |
| Cloud cell | Scale profile per cell + global router + cross-cell control | Auvora Cloud, dedicated |
| Air-gapped | Small/Scale + offline breach list + HSM + no outbound calls | Government, regulated |

## 5.3 Cell architecture (designed P0, built P9)

- **Cell** = a full Auvora stack + Postgres in one region.
- A Project has a `home_cell`, immutable except via the migration tool.
- **Global router:** a tiny, highly cached service (or DNS) mapping custom domain / issuer / project ID → cell.
- **Global control:** tenant directory, billing and the cross-cell admin view. It holds no identity data.
- **Data residency:** a project's PII never leaves its home cell; only aggregated metrics leave.
- **Failover:** each cell runs multi-AZ. Cross-region DR uses an async replica with RPO ≤ 1 min and RTO ≤ 1 h.

## 5.4 Consistency model

| Data | Store | Consistency |
|---|---|---|
| Identities, credentials, config | Postgres | Strong (primary) |
| Config in data plane | In-memory cache | ≤ 5 s p99 (NOTIFY + version check on miss) |
| Sessions | Postgres (+ Valkey read-through) | Revocation ≤ 5 s p99 |
| Auth codes, PAR, nonces, DPoP jti, assertion replay | Valkey or Postgres UNLOGGED | Single-use, **fail closed** |
| Rate limits | Valkey or Postgres | Approximate; fail *open for reads/closed for credential endpoints* per policy |
| ReBAC tuples | Postgres | Zookie/consistency-token semantics (at-least-as-fresh) |
| Audit | Postgres partitioned, append-only | Committed with the transaction (outbox) |

## 5.5 Crate and repository layout (D23)

```
auvora/
├── crates/
│   ├── auvora-core          # domain types, IDs, errors, tenancy context
│   ├── auvora-store         # Postgres, RLS, migrations, repositories
│   ├── auvora-crypto        # KeyProvider, aws-lc-rs, X.509, PQ hooks
│   ├── auvora-jose          # JWS/JWE/JWK/JWT, SD-JWT
│   ├── auvora-config        # config model, versioning, cache, export/import
│   ├── auvora-flows         # flow state machine (login/registration/recovery/settings/verification/consent)
│   ├── auvora-authn         # password, OTP, TOTP/HOTP, WebAuthn, magic link, recovery codes, push
│   ├── auvora-sessions      # sessions, devices, DBSC
│   ├── auvora-oauth         # AS/OP: all grants, PAR/JAR/JARM/RAR/DPoP/mTLS/CIBA/DCR/UMA
│   ├── auvora-federation    # OIDC/OAuth upstreams, social providers, broker, HRD, JIT
│   ├── auvora-saml          # isolated SAML (sandboxed worker)
│   ├── auvora-scim          # SCIM server + client + filter parser
│   ├── auvora-directory     # LDAP/AD sync, Kerberos
│   ├── auvora-ldap-gw       # LDAP server gateway
│   ├── auvora-radius        # RADIUS server
│   ├── auvora-proxy         # identity-aware proxy + forward-auth
│   ├── auvora-authz         # RBAC, Cedar, ReBAC graph, AuthZEN, explain
│   ├── auvora-cel           # CEL runtime, sandbox limits
│   ├── auvora-hooks         # webhooks, sync hooks, Wasm actions, module trait
│   ├── auvora-events        # outbox, CloudEvents, SSF/CAEP/RISC transmitter/receiver
│   ├── auvora-comms         # email/SMS/WhatsApp/voice, templates, i18n
│   ├── auvora-risk          # signals, scoring, bot, device intelligence
│   ├── auvora-audit         # hash chain, checkpoints, SIEM sinks, customer Audit Log API
│   ├── auvora-workload      # service accounts, workload federation, SPIFFE, X.509 CA
│   ├── auvora-agents        # agent principals, delegation, token vault, MCP
│   ├── auvora-support       # Supervised Support Access
│   ├── auvora-lifecycle     # JML workflows, provisioning orchestration
│   ├── auvora-migrate       # importers (Auth0, Keycloak, Firebase…), Auvora Import Format
│   ├── auvora-vc            # OID4VCI/OID4VP, SD-JWT VC, mdoc verification (P10)
│   ├── auvora-api           # Admin + runtime HTTP APIs (axum), OpenAPI generation
│   ├── auvora-server        # binary, roles, wiring
│   └── auvora-cli           # `auvora` CLI (same binary subcommands)
├── ee/crates/               # governance, PIM, multi-approver, cells control, HSM extras, RAC
├── apps/
│   ├── console/             # admin console
│   ├── hosted-ui/           # login/registration/consent/account portal SPA
│   └── org-portal/          # self-service SSO/SCIM setup for customers' IT admins
├── packages/                # @auvora/js, react, nextjs, elements (Web Components), node, vue, angular, svelte
├── sdks/                    # non-JS SDKs: go, python, java, dotnet, swift, kotlin, react-native, flutter, php, ruby, rust-client
├── mobile/                  # Auvora Authenticator (ios/, android/)
├── integrations/            # terraform-provider, mcp-server, framework plugins
├── tests/                   # cross-crate/system and E2E tests
├── bench/                   # benchmark suites
├── scripts/                 # repo tooling
├── deploy/                  # helm/, compose/, terraform-modules/
├── conformance/             # OIDF, SAML, SCIM, WebAuthn interop harnesses
├── fuzz/
├── docs/                    # product/, adr/, specs/, threat-models/, security/, roadmap/, releases/, runbooks/, site/ (ADR-0111)
└── examples/                # reference apps (dogfood)
```

## 5.6 Technology choices (final)

| Area | Choice |
|---|---|
| Language/runtime | Rust (stable), Tokio |
| HTTP | Axum + tower; hyper; rustls (aws-lc-rs) |
| DB | PostgreSQL 16+, SQLx, `sqlx migrate` |
| Cache | Valkey (optional) |
| Events | Outbox; NATS JetStream (optional); Kafka/Kinesis/PubSub/EventBridge sinks |
| Policy | Cedar (embedded), CEL (`cel-interpreter` or own hardened fork), native ReBAC |
| Wasm | wasmtime with fuel/memory limits, WASI-preview2 without network/fs (host functions only) |
| SAML | samael + libxmlsec1 (sandboxed worker) |
| WebAuthn | webauthn-rs (audited) + own policy layer |
| Workload | SPIFFE Workload API client; built-in X.509 CA; SPIRE federation optional |
| UI | React, TS, Vite, TanStack, Radix, Tailwind, design tokens (Style Dictionary) |
| Mobile | Swift (SPM), Kotlin (Maven Central), React Native, Flutter |
| Telemetry | OpenTelemetry, Prometheus |
| Packaging | OCI images (distroless), Helm, Compose, static binaries, Homebrew, deb/rpm |
| Testing | cargo test, proptest, cargo-fuzz, Testcontainers, Playwright, k6, OIDF conformance suite, Jepsen-style failure tests (toxiproxy) |

---

# 6. Data architecture

## 6.1 Storage rules

- PostgreSQL is authoritative for everything durable.
- Every tenant-owned table carries `tenant_id` and `project_id`. RLS policies use `current_setting('auvora.project_id')`, set per transaction through a dedicated role. Cross-tenant queries are only possible from the `platform` role, and they are audited.
- Large tables (audit, events, sessions, tokens metadata) use time partitioning with retention jobs.
- **Entity history:** every config and identity entity has a `_history` table (versioned rows with actor and transaction ID). It powers identity time travel, diffs and rollback.
- Secrets at rest (TOTP seeds, upstream tokens in the Token Vault, client secrets if reversible, SAML private keys) use envelope encryption: a per-project DEK wrapped by a KEK in the KeyProvider. Passwords and API-key secrets are hashed, never encrypted.
- PII columns are tagged in the schema (`pii: direct|indirect|sensitive`) for DSAR export, erasure, log redaction and residency checks.

## 6.2 Events and audit

- **Outbox → workers → sinks:** webhooks, NATS/Kafka, SIEM (syslog/CEF, OTLP logs, Splunk HEC, Datadog, Sentinel, Chronicle), SSF.
- Envelope format: CloudEvents 1.0, with a versioned event catalog (`auvora.user.created.v1`…) published as JSON Schema.
- **Audit fields:** event ID, time, tenant, project, org, actor (+ `act` chain), subject, client, session, IP/UA (policy-controlled), outcome, reason, trace ID, before/after hashes.
- **Tamper evidence:** a per-project hash chain, plus periodic signed checkpoints written to WORM storage (S3 Object Lock / GCS retention / Azure immutable).
- **Customer Audit Log API:** customers can post *their own* application events into Auvora's audit store and expose them to their enterprise customers (WorkOS-style).

---

# 7. Identity domain model

## 7.1 Hierarchy (D26)

```
Installation / Cloud
└── Tenant                 Auvora customer: billing, platform admins, home region
    └── Project            Issuer + user pool + keys + config. Environments = linked projects (dev/stg/prod)
        ├── Applications ──► OAuth clients / SAML SPs (1:N) ──► Resource servers (APIs) + scopes
        ├── Principals: User · Service account · Workload · Agent · Anonymous user
        ├── Connections: social, enterprise OIDC/SAML, LDAP/AD, SCIM directories
        ├── Policies: flow, access (conditional), authz (Cedar), ReBAC model
        └── Organizations   B2B customers of the developer (may nest: org → sub-org)
            ├── Memberships (user ↔ org, roles, attributes)
            ├── Org connections (their SSO/SCIM), verified domains, org policies, branding
            └── Org-scoped invitations, admins, audit view
```

## 7.2 Core entities

| Entity | Notes |
|---|---|
| Tenant | Billing account, platform admins, home region |
| Project | `iss` = `https://{custom-domain or project-slug}.{base}`; keys; per-environment |
| Environment link | Promotes config dev → stg → prod with diff and approval |
| Principal | Supertype with `kind` ∈ {user, service_account, workload, agent, anonymous} |
| User | Profile (typed schema), status, assurance (IAL), metadata (public/private/unsafe) |
| Identifier | Email, phone, username, external subject. Uniqueness is per project; normalization rules apply |
| Credential | password, passkey, totp, hotp, recovery_codes, push_device, x509, private_key_jwk, api_key, client_secret |
| Linked identity | Upstream IdP account (`connection_id`, upstream `sub`, issuer) |
| Organization | Hierarchical, domains, policies, branding, connections |
| Membership | Principal ↔ Org + roles + attributes + status (invited/active/suspended) |
| Group | Project- or org-scoped; static or dynamic (CEL rule) |
| Role / Permission | Project-, org- and app-scoped; composite roles |
| Application | Product grouping; has clients, branding, allowed connections |
| OAuth client | Protocol artifact; type (public/confidential), auth method, profile (e.g. FAPI2) |
| Resource server | API: identifier (audience), scopes, token format, RAR types |
| Session | Browser/native; devices; AAL; `amr`; risk; parent SSO session ↔ client sessions |
| Device | Registered device, attestation, push token, trust level |
| Grant | Consent record (client, scopes, RAR details, expiry) |
| Delegation | Agent/support/token-exchange relationship (`actor` → `subject`, constraints, expiry) |
| Support session | §16 |
| Connection | Upstream IdP or directory with mappings (CEL) |
| Hook / Action | Extension registrations |
| Policy | Flow policy, access policy, Cedar policy set, ReBAC model (all versioned) |
| Key | Signing/encryption keys, state machine: pending → active → retiring → retired → destroyed |
| Audit event, Domain event | §6.2 |

## 7.3 Identity rules

- An immutable `principal_id` is the only key. Email, phone and username are mutable identifiers.
- One human per project may have many identifiers, linked identities and org memberships.
- Normalization: NFC; case-folded emails (no provider-specific dot stripping); E.164 phones; usernames run through a UTS #39 confusable/skeleton check; identifiers on a reserved-word list are rejected.
- Linking requires proof of control (D34). Merge is an audited, reversible-for-30-days operation.
- **Anonymous users** get a principal with no identifiers. Upgrading attaches a credential. Merging into an existing account is explicit.
- Deletion follows D54. Legal hold overrides it.

---

# 8. Authentication

## 8.1 Methods (all in the open edition)

| Category | Methods |
|---|---|
| Knowledge | Password (email/username/phone identifier), with rehash-on-login and imported hash algorithms: bcrypt, scrypt, Firebase-scrypt, PBKDF2-SHA*, Argon2i/d/id, salted SHA/MD5 (legacy; force rehash) |
| Possession, phishing-resistant | Passkeys (synced and device-bound), FIDO2 security keys, PIV/CAC smart cards, X.509 client certs |
| Possession, other | TOTP, HOTP, push approval with number matching (SDK + Auvora Authenticator), email OTP, magic link (same/cross-device), SMS OTP, WhatsApp OTP, voice OTP |
| Recovery | Recovery codes, recovery passkey, admin-assisted recovery with identity re-proofing, trusted-contact recovery (optional) |
| Federated | Social, enterprise OIDC, SAML, LDAP bind, Kerberos/SPNEGO, FedCM |
| Identity proofing | Connector framework for IDV vendors (Onfido, Persona, Veriff, Jumio, DigiLocker/Aadhaar-offline-XML via partner, EUDI wallet via OID4VP) → IAL2 |
| Anonymous | Guest sessions with upgrade |

## 8.2 Controls

- **Assurance:** each method maps to AAL (D50). Policies can require an AAL or phishing-resistance per app, org, role or action.
- **Downgrade protection:** a user enrolled in phishing-resistant MFA can't be pushed to a weaker fallback unless policy allows it explicitly. Recovery then follows the stronger-recovery policy.
- **Passkeys:** discoverable credentials, conditional mediation (autofill), BE/BS flag handling, attestation policy (none/indirect/enterprise with an AAGUID allowlist via the FIDO MDS3 feed), Related Origin Requests, Signal API (keeping credential lists in sync), Credential Exchange (CXP/CXF) import/export guidance.
- **Brute force:** per identifier + per IP + per project limits; progressive delays; soft lock with notification; no permanent lockout DoS (unlock via a verified channel).
- **Argon2 DoS guard:** a bounded semaphore; queue with timeout → 429; pre-hash rate limits; hashing done off the async runtime (`spawn_blocking` pool).
- **SMS fraud:** off by default; per-project spend caps; country allowlist; velocity per prefix; anomaly detection; provider-side fraud APIs.
- **Breached passwords:** HIBP k-anonymity (padding enabled) + bundled offline bloom filter, updated per release.
- **MFA fatigue:** number matching, rate-limited pushes, "report suspicious" leading to automatic lock + alert.

## 8.3 Social providers (maintained)

Google, Apple, Microsoft (personal + Entra), GitHub, GitLab, Bitbucket, Facebook, Instagram, X, LinkedIn, Discord, Slack, Twitch, Spotify, Amazon, Salesforce, Atlassian, Notion, Zoom, Dropbox, Line, Kakao, Naver, WeChat, Yahoo Japan, PayPal, Shopify, Okta (as a social IdP), plus the **generic OIDC** and **generic OAuth 2.0 + userinfo** connectors.

Each provider definition is a declarative manifest (endpoints, scopes, claim mapping in CEL, quirks), contributed via PR and tested with recorded fixtures.

---

# 9. OAuth 2.x and OpenID Connect

## 9.1 Grants and flows

| Flow | Notes |
|---|---|
| Authorization Code + PKCE (S256 only) | Mandatory PKCE for all clients |
| Client Credentials | M2M |
| Refresh Token | D30 |
| Device Authorization (RFC 8628) | TVs, CLIs |
| Token Exchange (RFC 8693) | Delegation, impersonation-with-actor, cross-domain, agents |
| JWT Bearer (RFC 7523) and SAML 2.0 Bearer (RFC 7522) | Assertion grants with strict trust config |
| CIBA (poll, ping, push) | Agents, call centers, step-up on another device |
| UMA 2.0 grant | Resource-owner-controlled sharing |
| OAuth for First-Party Applications (draft) | Native credential UX without ROPC |
| Native SSO for Mobile Apps | Device secret across an app family |
| Pre-authorized code (OID4VCI) | P10 |
| **Not supported** | Implicit, ROPC (see §2.3) |

## 9.2 Endpoints and features

Authorization, token, revocation (RFC 7009), introspection (RFC 7662, plus JWT response RFC 9701), UserInfo (plain/signed/encrypted), discovery (OIDC + RFC 8414), JWKS, **Protected Resource Metadata (RFC 9728)** hosting for registered resource servers, PAR (RFC 9126), JAR (RFC 9101), JARM, RAR (RFC 9396), Resource Indicators (RFC 8707), `iss` response parameter (RFC 9207), DPoP (RFC 9449), mTLS client auth + certificate-bound tokens (RFC 8705), DCR/DCRM (RFC 7591/7592), Client ID Metadata Documents, Step-Up Authentication Challenge (RFC 9470), `form_post` response mode, `prompt`/`max_age`/`acr_values`/`claims` parameter, `login_hint`/`id_token_hint`, request objects with encryption, ID token encryption (JWE), pairwise subjects, OIDC Session Management, Front-Channel/Back-Channel/RP-Initiated Logout, `offline_access`.

## 9.3 Client policy profiles

Named, enforced bundles assignable per client or resource server:
- `baseline`: Code + PKCE, exact redirect match.
- `spa`: + DPoP recommended, refresh rotation.
- `native`: + claimed HTTPS / loopback redirect rules (RFC 8252).
- `confidential`: private_key_jwt or mTLS preferred.
- `fapi2-security`, `fapi2-message-signing`: PAR + DPoP/mTLS + JAR/JARM + strict algorithms.
- `mcp`: RFC 9728 + RFC 8707 + CIMD/DCR + PKCE.
- `agent`: token exchange + `act` + CIBA for sensitive scopes.

## 9.4 Token rules

The v0.2 rules are retained. Additions:
- `typ` headers (`at+jwt`, `logout+jwt`, `secevent+jwt`…).
- Explicit `aud` per resource server; never an `aud` array spanning unrelated APIs.
- Claim minimization; custom claims via CEL mappers with a size budget.
- `act`/`may_act` chains capped at a configurable depth.
- Introspection carries revocation state within D33.
- Redirect URIs: exact match; no wildcards in prod projects. Wildcard subdomains are allowed only in dev projects with a banner. Loopback any-port for native apps per RFC 8252.

---

# 10. Machine and workload identity

| Concept | Definition |
|---|---|
| Service account | Auvora-managed non-human principal; credentials: private_key_jwt keys, mTLS certs, API keys (last resort) |
| Workload | Externally attested runtime; **no stored secret**; authenticates by presenting a trusted platform token |
| OAuth client | Protocol artifact, not a principal |

**Workload federation sources:** Kubernetes projected SA tokens (per-cluster issuer trust), AWS (STS `GetCallerIdentity` signed request + IAM Roles Anywhere certs), GCP (identity tokens), Azure (managed identity tokens), GitHub Actions OIDC, GitLab CI OIDC, Bitbucket Pipelines, CircleCI, Buildkite, Terraform Cloud, SPIFFE JWT-SVID/X.509-SVID, generic OIDC issuer.

**Trust policies:** CEL over the incoming claims (`repo`, `ref`, `environment`, `namespace`, `service_account`…) → mapped principal + allowed audiences and scopes + max TTL.

**Built-in CA:** issues short-lived X.509 (mTLS) and SPIFFE-compatible SVIDs. Roots can be KMS/HSM-backed; supports intermediate import (enterprise PKI); CRL/OCSP.

**Lifecycle:** owner (required), purpose, environment tags, last-used, credential age, rotation policy, expiry alerts, unused-identity detection, inventory API, risk signals (new ASN, burst, scope anomaly).

**Transaction Tokens (draft):** issued for internal call chains, propagating user and workload context across microservices.

**`auvora-pg`:** helper for apps that map validated JWT claims into Postgres session settings for row-level security.

---

# 11. AI agent identity (headline capability)

## 11.1 Model

- **Agent:** principal `kind=agent`. Fields: owner (user/org/service), publisher, version, declared capabilities, risk tier, allowed tools/resources, max delegation depth, attestation (optional: workload federation or signed agent manifest).
- **Delegation record:** user/org → agent. Scopes + RAR details + resource constraints + spend/rate limits + expiry, revocable from the account portal and the admin console.

## 11.2 Flows

| Need | Mechanism |
|---|---|
| Agent acts on behalf of a user | Token exchange → token with `sub`=user, `act`={agent}, narrowed scopes/RAR |
| Agent calls another agent | Nested `act` chain (depth-capped), each hop audited |
| Human approval for sensitive actions | CIBA push to the user's device/app; RAR shows exactly what is approved |
| Agent needs third-party APIs (Gmail, Slack, GitHub…) | **Token Vault**: Auvora stores upstream refresh tokens (envelope-encrypted) and mints short-lived upstream access tokens on request, bound to delegation scope |
| MCP servers protected by Auvora | RFC 9728 metadata + RFC 8707 audiences + PKCE + CIMD/DCR; `@auvora/mcp` middleware for TS/Python/Go/Rust MCP servers |
| MCP clients (agent hosts) | Client ID Metadata Documents, or DCR under policy; consent per tool/resource |
| Autonomous (no-human) agents | Workload federation or service-account credentials; same audit and policy |
| Kill switch | Revoke agent globally, per user or per tool. Propagation ≤ 5 s + SSF event |

## 11.3 Governance

Agent inventory, owner attestation, per-agent activity timeline, anomaly detection (tool-call bursts, new resources), and periodic delegation reviews (P8).

## 11.4 Auvora MCP server

An official MCP server exposes the Admin API (read-only by default; writes need step-up + approval), so admins can manage IAM through AI assistants safely.

---

# 12. Federation and enterprise SSO

## 12.1 Upstream (Auvora as RP/SP)

- Enterprise OIDC and SAML SP connections, at project or **org** scope.
- **Home realm discovery:** by email domain, org slug, IdP hint, or `login_hint`.
- JIT provisioning with CEL attribute/role mapping; group → role mapping.
- Metadata URL refresh (with SSRF guard) and certificate rollover alerts.
- Multiple IdPs per org; IdP-initiated SAML only per connection opt-in, with RelayState/replay protections.

## 12.2 Downstream (Auvora as OP/IdP)

- OIDC OP (§9).
- **SAML IdP** with per-SP metadata, NameID formats, attribute statements via CEL, encrypted assertions, SLO.
- **WS-Federation passive** (for older Microsoft stacks), shipped in P7 because Keycloak/ADFS users need it.

## 12.3 Broker matrix

Any upstream (OIDC, SAML, LDAP, Kerberos, social, passkey) → any downstream (OIDC, SAML, WS-Fed, LDAP gateway, RADIUS, proxy headers). Claims are normalized through the canonical principal model.

## 12.4 Org Admin Portal (self-service)

The developer generates a time-limited portal link for a customer's IT admin. The portal offers guided SSO setup (per-IdP wizards for Okta, Entra, Google, ADFS, Ping, OneLogin, JumpCloud, generic SAML/OIDC), SCIM setup, domain verification (DNS TXT), test-connection with live debugging, certificate expiry notices, and an org audit view. It ships as hosted, embeddable (iframe-free component) or API-only.

## 12.5 OpenID Federation 1.0 and IPSA

- OpenID Federation: trust-chain resolution for large federations (education, government, health). P10.
- IPSA profile compliance (SSO + SCIM + SSF): target in P8.

---

# 13. Experience layer: Flow API, hosted UI, components, portals

## 13.1 Flow API (single engine)

- Flow types: `login`, `registration`, `recovery`, `verification`, `settings`, `consent`, `device`, `ciba-approval`, `org-select`, `step-up`, `link-account`, `logout`.
- Each flow is a server-side state machine. The client gets `GET /v1/flows/{type}/{id}`, which returns `{state, nodes[], messages[], csrf}`. Nodes are typed UI-agnostic descriptors (input, passkey challenge, redirect, QR, choice…). The client does `POST` to transition.
- **Flow policy** (D16, CEL): steps, factor options, conditions (org, app, risk, AAL, country, device), progressive profiling, terms acceptance.
- Browser mode: cookie + CSRF. Native mode: token-bound flow IDs + app attestation (App Attest, Play Integrity).

## 13.2 Integration surfaces (all over the Flow API)

1. **Hosted UI:** custom domain, themes, templates.
2. **Drop-in components:** `@auvora/react`, `@auvora/elements` (Web Components for Vue/Angular/Svelte/vanilla), `<SignIn/>`, `<SignUp/>`, `<UserProfile/>`, `<OrgSwitcher/>`, `<OrgProfile/>`. These render in the customer's origin.
3. **Headless SDKs:** full control.
4. **Self-hosted UI:** a fork of the hosted-UI SPA (Apache-licensed) deployed anywhere.

## 13.3 Theming and customization

- Design tokens → CSS variables; component **slots**; layout presets; per-page templates (safe, logic-less: no JS execution).
- Per project / app / org theme overrides; versioned with preview URLs, A/B and rollback.
- Localization: 40+ locales shipped, ICU MessageFormat, per-tenant overrides, RTL support.
- **Accessibility:** WCAG 2.2 AA, automated axe checks + manual audit per release.
- **Security:** strict CSP with nonces, Trusted Types, `frame-ancestors` per policy, no tenant JS in the login origin.
- **Registration forms:** schema-driven, multi-step, conditional fields, custom field validators (CEL), consent checkboxes linked to terms versions.

## 13.4 Account portal (end user)

Profile, identifiers, password, passkeys, MFA devices, sessions/devices (with sign-out), linked accounts, connected apps and agents (revoke delegations), org memberships, privacy (download data, delete account), notification preferences, security activity log.

## 13.5 Admin console and developer tools

- Full Admin API parity; environment switcher; config diff/promote; identity time travel; authz decision explorer.
- Flow debugger, token inspector (secrets redacted), SAML tracer, SCIM log viewer, webhook replay, hook test console.
- Quickstarts per framework; sample apps; local dev server (`auvora dev`) with seeded tenants and test users; mailbox catcher for email/SMS in dev.

---

# 14. Extensibility (D38)

| Tier | Mechanism | Runs | Use |
|---|---|---|---|
| 1 | **Webhooks** | Async, signed (HMAC + optional JWS), retries with backoff, DLQ, replay, ordering keys | Integrations, analytics |
| 2 | **Sync HTTP hooks** | At defined points with timeout, circuit breaker, fail-open/closed per hook | Pre-registration checks, claims enrichment, lazy password migration, risk vendors, IDV |
| 3 | **CEL expressions** | In-process, bounded cost | Mappings, conditions, dynamic groups, flow rules |
| 4 | **Wasm actions** | In-process wasmtime sandbox; fuel/memory/time caps; capability-scoped host functions (http-fetch via allowlist, KV, log) | Auth0-Actions-class logic without external hosting |
| 5 | **Rust modules** | Compile-time trait implementations (custom builds) | Keycloak-SPI-class deep customization |

**Hook points:** pre-user-create, post-user-create, pre-login, post-authentication, pre-token-issue, pre-userinfo, pre-saml-assertion, password-verify (migration), mfa-required decision, pre-org-join, post-org-join, scim-pre-apply, pre-support-session, pre-agent-delegation, credential-exchange.

**Invariant:** no extension can skip credential verification, signing, audit or policy enforcement. Extensions can only add data, deny, or require more.

---

# 15. Communications

- Channels: email, SMS, WhatsApp, voice, push (APNs/FCM), in-app (webhook).
- Provider abstraction with failover chains and per-project credentials (D36).
- Templates: MJML + Handlebars-like logic-less syntax; per-locale; preview; versioning; test-send.
- Per-project sender domains with SPF/DKIM/DMARC verification guidance; bounce/complaint handling; suppression lists.
- Dev mode: built-in catcher UI.
- Abuse: per-recipient and per-IP send limits, OTP resend backoff, SMS fraud controls (§8.2).

---

# 16. Supervised Support Access

v0.2 §14 is retained in full: identity semantics, the formula `effective = target ∩ agent ∩ policy`, workflow, and security requirements. Additions:

- **Modes:** `view-as` (read-only, masked) · `act-as` (scoped write) · `co-browse` (user-initiated, user-present).
- **Consent options:** none (tenant policy), user approval in-app/email/CIBA, org-admin approval, multi-approver (EE).
- **Data classification:** field-level masking/denial in tokens (`support_restrictions` claim) and in the SDK middleware helpers for resource servers.
- **Customer-visible ledger:** end users and org admins see every support session on their account.
- **Session controls:** live termination, max duration, idle timeout, no refresh tokens, no nested support sessions, no credential/MFA changes, and no support actions on other support agents.
- **SDK support:** `useSupportSession()` banner component; server middleware enforcing `act` + restrictions.
- **Dogfood:** Auvora Cloud staff access uses this only (D57).

---

# 17. Provisioning and directories

- **SCIM 2.0 server:** Users, Groups, EnterpriseUser, custom schemas, PATCH, filters (fuzzed parser), sorting, pagination, bulk, ETags; per-org SCIM tokens; Okta/Entra/OneLogin/JumpCloud quirk profiles; provisioning logs.
- **SCIM client (outbound):** provisions Auvora identities into downstream apps with mapping, retries, reconciliation and dry-run.
- **SCIM events (draft) / SSF:** push changes.
- **LDAP/AD:** scheduled and incremental sync (USN/DirSync), pass-through bind, group mapping, write-back (optional), multi-domain forests, LDAPS/StartTLS.
- **HR-driven provisioning (P8):** connectors framework (Workday, BambooHR, SuccessFactors, generic CSV/SFTP, generic REST) as authoritative sources with attribute precedence rules.
- **Import/export:** bulk JSON/CSV; streaming; password hash import; lazy migration hook; importers listed in D58.

---

# 18. Lifecycle, governance and privileged access

| Capability | Phase | Edition |
|---|---|---|
| Invitations, verification, suspension, deprovisioning | P1–P4 | Open |
| Delegated administration (org admins, scoped admin roles) | P4 | Open |
| Lifecycle Workflows (joiner/mover/leaver; triggers + steps + hooks) | P8 | Engine open; designer EE |
| Access requests + approval chains | P8 | EE |
| Access packages / entitlement management | P8 | EE |
| Access reviews / certifications (users, service accounts, agents, delegations) | P8 | EE |
| Separation-of-duties rules (Cedar) | P8 | EE |
| JIT privileged elevation (time-bound role activation with approval + MFA) | P8 | EE |
| Break-glass accounts (sealed, monitored, alerting) | P4 | Open |
| Identity posture and ITDR reports (dormant, stale credentials, MFA coverage, over-privilege, orphaned service accounts) | P8 | Basic open; advanced EE |
| Compliance evidence packs (SOC 2, ISO 27001, HIPAA mappings) | P9 | EE |

---

# 19. Legacy and infrastructure access gateways

| Gateway | Description | Phase |
|---|---|---|
| LDAP gateway | LDAPv3 server (bind, search) over Auvora directory; per-app virtual DIT; service binds; MFA via bind-time push/OTP suffix | P7 |
| RADIUS server | PAP/EAP-TTLS/EAP-TLS for VPN/Wi-Fi; MFA via push/OTP; per-NAS secrets; RadSec | P7 |
| Kerberos/SPNEGO | Desktop SSO with AD keytab | P7 |
| Identity-aware proxy | Reverse proxy and forward-auth endpoint; header/JWT injection; per-route policies (Cedar) | P7 |
| SSH CA | Short-lived user/host certs after OIDC login; `auvora ssh` CLI | P10 |
| NSS/PAM module | Linux login with Auvora identities, offline cache | P10 |
| Browser remote access (RDP/SSH/VNC) | EE optional module | P10 |

---

# 20. Authorization platform

## 20.1 Layers

1. **Scopes and RAR**: API delegation at the token level.
2. **Native RBAC**: project, org and app roles; composite roles; role assignment conditions (CEL); emitted as claims (size-limited) or queried.
3. **Cedar policies**: ABAC/PBAC with entity data from Auvora (principals, orgs, groups, attributes) plus request context. Policy validation against the schema; versioning; staged rollout (shadow mode).
4. **Embedded ReBAC graph**: OpenFGA-DSL-compatible models, tuples in Postgres, check/list-objects/list-users/expand, contextual tuples, conditions (CEL), consistency tokens, Leopard-style indexing for deep groups, watch API.
5. **AuthZEN Authorization API**: the public PDP interface (evaluation, evaluations batch, search). It's also available as a local sidecar/embedded SDK for low-latency PEPs.

## 20.2 Tooling

- Decision **explain** traces (which policy/tuple/role allowed or denied).
- Policy unit tests in CI (`auvora authz test`).
- Shadow evaluation of new policies against live traffic.
- Policy diff + approval.
- Generic **Entities** (FusionAuth-style) for modelling non-identity resources.
- **UMA 2.0** resource registration for user-controlled sharing.

## 20.3 Enforcement SDKs

Middleware for Node/Express/Fastify/Nest, Next.js, Go (net/http, gin, echo, chi), Python (FastAPI, Django, Flask), Java (Spring Security), .NET (ASP.NET Core), Rust (axum/tower), PHP (Laravel, Symfony), Ruby (Rails). Each validates tokens (JWKS cache, DPoP, mTLS binding), enforces `act`/support restrictions, and calls AuthZEN locally or remotely.

## 20.4 External adapters (optional)

OpenFGA and SpiceDB (sync tuples out or delegate checks), OPA (policy delegation for orgs already on Rego).

---

# 21. Sessions, devices and continuous access

- Server-side SSO session → per-client sessions; session tree visible to users and admins.
- Idle and absolute timeouts per project, org and app; concurrent-session limits; re-authentication for sensitive actions (`max_age`, step-up RFC 9470).
- **DBSC** (Device Bound Session Credentials) where supported; native apps use a device-bound key + DPoP.
- Remembered devices with trust levels and attestation; device inventory.
- **Continuous access:** SSF transmitter and receiver. Auvora emits CAEP (`session-revoked`, `credential-change`, `assurance-level-change`, `device-compliance-change`) and RISC (`account-disabled`, `credential-compromise`…). It also consumes signals from upstream IdPs, EDR and MDM to revoke or step up sessions.
- Logout: RP-initiated, front-channel, back-channel, SAML SLO, global sign-out, per-device sign-out.

---

# 22. Risk engine, bot defense and ITDR

- **Signals:** IP reputation (pluggable feeds + built-in Tor/hosting ASN lists), velocity, impossible travel, new device, device intelligence (privacy-conscious, hashed, consented), credential-stuffing patterns, breached-credential hits, behavioral anomalies, upstream SSF signals, bot score.
- **Scoring:** rule-based (CEL) with weights in v1; an optional ML scorer via hook in P8. Every decision is explainable and logged.
- **Actions:** allow, challenge (CAPTCHA/PoW), require MFA, require phishing-resistant, restrict session, admin approval, deny + alert, lock + notify.
- **Privacy:** geolocation and fingerprinting are configurable and disclosed; no raw fingerprints are stored.

---

# 23. Cryptography and key management

The v0.2 §17 content is retained. Additions:

- **KeyProvider (D11) state machine:** pending → active → retiring (verify only) → retired → destroyed. JWKS publishes active + retiring. Rotation is automatic, scheduled and emergency (one-click compromise rotation with an SSF notification).
- **Key usage separation:** token signing, ID token encryption, SAML signing, SAML encryption, cookie/CSRF HMAC, webhook signing, DEK wrapping, CA roots/intermediates.
- **Algorithm policy:** an allowlist per project/profile. `none` is never accepted. The `alg` is pinned per key (no `alg` from token headers).
- **Post-quantum:** hybrid TLS now; ML-DSA/SLH-DSA for JOSE and X.509 behind a flag once IANA registrations are final; a migration playbook.
- **FIPS build:** aws-lc-rs FIPS module, algorithm restriction, self-tests at boot.
- **Backup:** key material exported only wrapped by the KEK; restore drills each release.

---

# 24. Standards baseline (complete)

**OAuth/OIDC:**
RFC 6749, 6750, 7009, 7519, 7515/7516/7517/7518, 7521, 7522, 7523, 7591, 7592, 7636, 7662, 8176, 8252, 8414, 8628, 8693, 8705, 8707, 9068, 9101, 9126, 9207, 9396, 9449, 9470, 9700, 9701, 9728; OAuth 2.1 (draft); Client ID Metadata Documents (draft); OAuth for First-Party Apps (draft); Transaction Tokens (draft); OpenID Connect Core, Discovery, Dynamic Registration, Session Management, Front-Channel, Back-Channel, RP-Initiated Logout, Native SSO, CIBA, JARM, FAPI 2.0 Security Profile + Message Signing, UMA 2.0, OpenID Federation 1.0, AuthZEN Authorization API 1.0.

**Signals:** OpenID Shared Signals Framework, CAEP, RISC; RFC 8417 (SET), RFC 8935/8936 (SET delivery).

**Federation and provisioning:** SAML 2.0 (core, bindings, profiles, metadata, SLO, metadata interop), WS-Federation (passive), SCIM RFC 7642/7643/7644 + SCIM events (draft), IPSA profile, LDAPv3 (RFC 4510–4519), Kerberos (RFC 4120, 4559 SPNEGO), RADIUS (RFC 2865/2866, 6614 RadSec, EAP 3748/5216/5281).

**Authentication:** WebAuthn Level 3, CTAP 2.2, FIDO MDS3, passkey Related Origin Requests, WebAuthn Signal API, FIDO CXP/CXF, TOTP RFC 6238, HOTP RFC 4226, NIST SP 800-63-4, FedCM, DBSC.

**Workload:** SPIFFE (SVID, Workload API, Federation), X.509 (RFC 5280), ACME (RFC 8555), OCSP (RFC 6960).

**Credentials:** OID4VCI 1.0, OID4VP 1.0, SD-JWT (RFC 9901) and SD-JWT VC, ISO 18013-5/7 mdoc (verification), EUDI ARF.

**APIs and events:** OpenAPI 3.1, RFC 9457 Problem Details, CloudEvents 1.0, OpenTelemetry semantic conventions.

**Security references:** OWASP ASVS 5.0 (target Level 3 for core), OWASP API Top 10, CWE Top 25.

---

# 25. Non-functional requirements

| Metric | Target (GA 1.0, reference: 3 × 4 vCPU nodes + Postgres 8 vCPU) |
|---|---|
| Token endpoint (client_credentials, JWT) | ≥ 10,000 req/s cluster, p99 < 50 ms |
| Password login (excl. Argon2 cost) | p99 < 150 ms; ≥ 1,000 logins/s with hashing pool sized |
| Passkey login | p99 < 100 ms server time |
| JWKS / discovery | Cacheable; p99 < 5 ms |
| AuthZEN check (embedded ReBAC, warm) | p99 < 5 ms; ≥ 50,000 checks/s cluster |
| Config propagation | ≤ 5 s p99 |
| Session revocation propagation | ≤ 5 s p99 in-region |
| Users per project | ≥ 100 M (tested to 10 M at GA) |
| Projects per cell | ≥ 100,000 |
| Binary RSS idle | < 100 MB |
| Cold start | < 2 s |
| Cloud SLO | Data plane 99.95% (GA) → 99.99% (cells) |
| RPO / RTO | ≤ 1 min / ≤ 1 h regional |
| Upgrades | Zero-downtime N-1 → N |

All targets are verified by the reproducible benchmark suite (`bench/`) published per release.

---

# 26. Security engineering and assurance

The v0.2 §20 controls are retained. Additions to the abuse-case suite (each gets a regression test):

- SSRF via tenant URLs (discovery, JWKS, metadata, webhooks, logo/jwks/request URIs). Mitigated by an egress proxy layer with deny-private-ranges, DNS-rebinding protection, and an allowlist option.
- Pre-account-takeover; unverified email from IdP ("nOAuth" class); email-change takeover.
- MFA fatigue, SIM swap, AiTM phishing proxies (mitigated by phishing-resistant policy + DBSC + risk).
- Open redirect, redirect-URI wildcards, loopback abuse, consent phishing (unverified third-party app warnings, publisher verification).
- Clickjacking, CORS, CSRF on flows, XS-leaks, cache poisoning on hosted UI.
- Unicode confusables, normalization collisions, timing attacks (constant-time compares, uniform lookups).
- SMS pumping, email bombing, Argon2 CPU exhaustion, ReDoS (no backtracking regex engines: Rust `regex` crate only), CEL/Wasm resource exhaustion.
- Tenant-admin escalation, cross-project data access (RLS tests on every query path), insider/operator abuse.
- Supply chain (cargo-vet, npm provenance, pinned actions), backup exfiltration.
- Authentication downgrade, refresh-token family theft, DPoP proof replay, `act`-chain forgery, support-session pivoting, agent delegation escalation, Token Vault exfiltration.

**Programs:** public vulnerability disclosure and bug bounty (from public beta), security advisories (GHSA + CVE), external pentest and protocol review before each major release, OWASP ASVS L3 self-assessment published.

---

# 27. Roadmap

Phases are gated by **exit criteria**, not dates. Each phase lists its parallel tracks.

**Standing gates for every phase:**
- Threat model updated.
- Zero open high/critical security findings.
- Fuzzers run ≥ 24 h per new parser with no crash.
- Upgrade test from the previous release passes.
- Docs and examples for every new feature.
- Benchmarks show no regression > 5%.

## P0: Foundations

- Monorepo, CI/CD, supply chain (D46), coding standards (D47), CLA, license files.
- ADRs D01–D60 written; domain model spec + ERD; tenancy/RLS design; ID scheme; cell-ready routing design.
- Threat-model templates; data classification; PII tagging scheme.
- `auvora-core`, `auvora-store`, `auvora-crypto` (KeyProvider: local + AWS KMS), `auvora-jose` skeletons with fuzz targets.
- Conformance harness (OIDF suite container), bench harness, Testcontainers setup.
- Spikes with go/no-go reports: samael interop with 5 IdPs, webauthn-rs, cel-interpreter hardening, wasmtime limits, Cedar entity loading.

**Exit:** all ADRs merged; spikes are go (or fallback triggered and documented); CI green with fuzz, deny and SBOM; RLS isolation property tests pass.

## P1: Identity core + OIDC provider → `0.1 Alpha`

**Tracks:**
- **Core:** tenants, projects, environments, principals, users, identifiers, credentials, groups, admin roles (D55), entity history, audit + outbox, rate limiting, comms (email), config cache + NOTIFY.
- **Authn:** password (Argon2id, breach check, rehash, imported hashes), email OTP, magic link, verification, recovery, sessions.
- **Flows:** Flow API v1 (login, registration, recovery, verification, settings, logout); minimal hosted UI.
- **OAuth:** Code+PKCE, client credentials, refresh rotation, discovery, JWKS, UserInfo, revocation, introspection, RP-initiated logout, key rotation, resource servers + scopes, RFC 9068/8707/9207.
- **Ops:** single binary, Compose, Helm draft, OTel, health/readiness.

**Exit:**
- OIDF Basic + Config OP pass 100% in CI.
- A sample Next.js app and a Go API work end-to-end.
- Cross-project isolation tests pass.
- k6 achieves ≥ 2,000 token/s on one node.
- Upgrade 0.1.x → 0.1.y is zero-downtime.

## P2: Authentication breadth → `0.2`

- Passkeys (full §8.2), TOTP/HOTP, recovery codes, step-up (RFC 9470), AAL mapping, downgrade protection.
- Social connectors (top 12 + generic), account linking/merge, anonymous users.
- SMS/WhatsApp/voice OTP with fraud controls; bot protection (D37); enumeration resistance.
- Flow policies in CEL; progressive profiling; terms versions.
- Account portal; privacy basics (DSAR export, deletion, consent records).
- Device flow, form_post, front/back-channel logout, consent screen.

**Exit:** WebAuthn interop matrix (Chrome, Safari, Firefox, Edge; iOS, Android, Windows Hello; YubiKey, Titan) green; OIDF Logout profiles pass; enumeration timing test (≤ 5 ms variance p95).

## P3: Developer platform → `0.5 Public Beta` (the MVP)

- Hosted UI v1: themes, tokens, slots, i18n (20 locales), WCAG 2.2 AA, custom domains + ACME.
- Drop-in components (React + Web Components); account portal components.
- Admin console v1 (full API parity for P1–P3 features), config export/import/promote, identity time travel UI.
- Webhooks, sync hooks, Rust module trait; event catalog.
- SDKs tier 1: JS core, React, Next.js, Node, Go, Python. Resource-server middleware for those stacks.
- CLI, Terraform provider, docs site, quickstarts, `auvora dev`.
- Migration importers: Auth0, Keycloak, Firebase, Cognito, Supabase, Clerk; lazy migration hook.
- Bug bounty opens.

**Exit:** an external developer integrates a sample app in < 15 minutes (measured with 5 testers); 100% Admin API coverage in Terraform; axe has 0 violations; OIDF Dynamic OP pass.

## P4: B2B and enterprise SSO → `0.7`

- Organizations full (nesting, invitations, domains, per-org policies/branding/connections), org switcher, org-scoped tokens.
- Native RBAC (project/org/app roles), delegated admin, break-glass.
- Enterprise OIDC + **SAML SP**, HRD, JIT, group/role mapping, IdP-initiated (opt-in).
- SCIM 2.0 server with IdP quirk profiles; **Org Admin Portal**.
- Customer Audit Log API; SIEM sinks; hash-chain checkpoints.
- Access Policies (conditional access, CEL): device, network, risk, AAL, time.

**Exit:** SAML SP interop with Okta, Entra, Google, ADFS, Ping, OneLogin, JumpCloud, Keycloak; SCIM provisioning certified flows with Okta + Entra; SAML fuzz 72 h clean; external review of SAML code.

## P5: Advanced OAuth, machine, workload and agent identity → `1.0 GA`

- DPoP, mTLS (auth + bound tokens), PAR, JAR, JARM, RAR, token exchange (`act`), JWT/SAML bearer, CIBA, DCR/DCRM, CIMD, RFC 9728 hosting, client policy profiles, encrypted ID tokens, pairwise.
- Service accounts, built-in CA, workload federation (K8s, AWS, GCP, Azure, GitHub/GitLab CI, SPIFFE), Transaction Tokens, `auvora-pg`.
- **Agents:** agent principals, delegations, CIBA approvals, Token Vault, MCP middleware, Auvora MCP server, kill switch.
- Risk engine v1 + device intelligence; SSF/CAEP/RISC transmitter + receiver.
- SDKs tier 2: Swift, Kotlin, React Native, Flutter, Java/Spring, .NET.
- Hardening: external pentest + protocol review, ASVS L3 self-assessment, DR drills, Cell-ready Postgres partitioning.

**Exit:**
- OIDF certifications obtained: Basic, Config, Dynamic, Form Post, RP-Initiated/Front-Channel/Back-Channel Logout.
- All §25 NFR targets met on reference hardware.
- Pentest: 0 high/critical open.
- MCP interop with ≥ 3 major MCP clients.
- 30-day public-beta stability (no P1 incidents).

**Then:** Auvora Cloud private beta (EU, US, India cells).

## P6: Authorization platform + Supervised Support Access → `1.1–1.2`

- Cedar integration (schema from the Auvora entity model), embedded ReBAC engine, AuthZEN API, explain, shadow mode, `auvora authz test`, UMA 2.0, Entities; OpenFGA/SpiceDB/OPA adapters.
- Supervised Support Access (full §16), SDK banners/middleware; dogfood in Cloud.

**Exit:** ReBAC passes the OpenFGA model test-suite compatibility set; AuthZEN interop event participation; support-access abuse suite 100%; independent review of support access.

## P7: Enterprise federation and legacy gateways → `1.3–1.4`

- SAML IdP, WS-Fed, full broker matrix; LDAP/AD sync + pass-through + Kerberos/SPNEGO.
- LDAP gateway, RADIUS server, identity-aware proxy/forward-auth.
- Push MFA SDK + **Auvora Authenticator** app (iOS/Android); FIPS build; HSM (PKCS#11), GCP/Azure KMS, Vault Transit.
- Visual Flow Designer v1.

**Exit:** SAML IdP interop with top 20 SaaS apps (Salesforce, AWS, Google Workspace, Slack, Zoom, Atlassian, GitHub Enterprise…); RADIUS with ≥ 3 VPN vendors; LDAP gateway with ≥ 5 legacy apps.

## P8: Governance, privileged access, ITDR → `2.0`

- Lifecycle Workflows, HR connectors, SCIM client, access requests, access packages, reviews (users, service accounts, agents), SoD, JIT elevation, posture/ITDR reports, IPSA profile, Wasm actions GA.

**Exit:** a governance pilot with 2 design partners; IPSA interop; Wasm sandbox escape review clean.

## P9: Scale and sovereignty → `2.x`

- Multi-region cells GA, global router, project cell migration tool, dedicated/air-gapped deployments, compliance evidence packs, SOC 2 Type II + ISO 27001 for Cloud, 99.99% SLO.

**Exit:** regional failover drill within RTO/RPO; chaos test suite green; audits passed.

## P10: Frontier → `3.x` (continuous)

FAPI 2.0 + CIBA certification, OID4VCI/OID4VP + EUDI wallet + mdoc verification, OpenID Federation, PQ signatures, SSH CA + NSS/PAM, browser remote-access gateway, more SDKs (PHP, Ruby, Elixir, Rust client), integration marketplace, migration importers for Okta/Zitadel/Ory.

## 27.1 Dependency summary

```
P0 ─► P1 ─► P2 ─► P3 (Beta) ─► P4 ─► P5 (GA) ─► P6 ─► P7 ─► P8 ─► P9
                        │                  ▲
                        └── SDK tiers, docs, Terraform evolve continuously
P10 items start once their prerequisite phase is complete (e.g. OID4VC after P5, SSH CA after P7).
```

---

# 28. Drawback → solution map (from the review)

| Review drawback | Solution in v1.0 (no feature compromise) |
|---|---|
| D1 Boundless scope, no MVP | Full scope retained. Gated phases P0–P10; MVP = 0.5 Public Beta; GA = 1.0; each phase shippable |
| D2 No differentiation | Superset matrix (§2) + 5 pillars (§1.1) + beyond-market features (§2.2) |
| D3 Dependency sprawl | Embedded-first (principle 14): Postgres-only profile runs **every** feature; externals are optional accelerators |
| D4 Realm/tenant ambiguity | Tenant → Project → Organization (§7), per-project pools (D27) |
| D5 No extensibility | 5-tier model (§14) with hook points and invariants |
| D6 Agent identity unplanned | §11, delivered in GA (P5) |
| D7 CP/DP propagation undefined | D09 + §5.4 consistency table + D33 revocation SLO |
| D8 No comms | §15 |
| D9 Business model deferred | §3.2 + D03/D04 |
| D10 No NFRs | §25 |
| D11 Undecided "X or Y" | Decision register §4, each with a fallback trigger |
| D12 SDK burden | Tiered SDKs; admin SDKs generated; Web Components cover all frontend frameworks with one codebase |
| D13 Vague exit criteria | Measurable exit criteria per phase + standing gates (§27) |
| Roadmap inversions | Keys/admin roles/rate limits/comms/hosted UI moved to P1; conformance in CI from P1; social/magic/OTP placed; token exchange before support/agents |

---

# 29. Engineering process

- **Branching:** trunk-based, short-lived branches, required reviews (2 for `security`-labelled), conventional commits, automated changelog.
- **Definition of done:** tests (unit + integration + negative), threat model note, docs, audit events defined, metrics, config-as-code support, Admin API + console + Terraform parity, upgrade-safe migration.
- **Spec-first:** OpenAPI and the event catalog are written before handlers; breaking-change linter (oasdiff) in CI.
- **Security:** CODEOWNERS on crypto/protocol crates; dependency additions require cargo-vet audit; secrets scanning; signed commits for maintainers.
- **Quality:** coverage thresholds on protocol crates (≥ 90% line, mutation testing with cargo-mutants on `auvora-oauth`, `auvora-jose`, `auvora-saml`, `auvora-authz`).
- **Dogfooding:** console and Cloud authenticate via Auvora; `examples/` B2B demo app built only on public SDKs runs in E2E.

---

# 30. Risks

| Risk | Mitigation |
|---|---|
| SAML stack weakness | Sandboxed worker, fuzzing, external review, JVM fallback backend (D14) |
| Breadth dilutes quality | Phase gates; no phase starts before the previous exit; standing gates |
| Embedded ReBAC complexity | Spec + model-compat suite; external adapter fallback always available |
| Wasm sandbox escape | wasmtime with no WASI fs/net, fuel limits, separate process option, external review |
| AGPL adoption friction | Apache SDKs; clear license FAQ; commercial license available |
| Competitor feature drift | Quarterly superset-matrix review (§2) as a standing product ritual |
| Standards drafts change | Drafts behind feature flags, versioned profiles |
| Postgres as the only store at extreme scale | Partitioning, read replicas, cells; Valkey/NATS/ClickHouse accelerators |

---

# 31. Glossary

- **Tenant:** an Auvora customer account.
- **Project:** an issuer and user pool (a "realm" in Keycloak terms).
- **Organization:** a B2B customer inside a project.
- **Principal:** any security subject.
- **Agent:** an autonomous principal acting via delegation.
- **Workload:** an attested runtime with no stored secret.
- **Cell:** a regional, self-contained deployment.
- **Flow:** a server-side authentication state machine.
- **PDP/PEP:** policy decision/enforcement point.
- **AAL/IAL/FAL:** NIST assurance levels.
- **SSF/CAEP/RISC:** shared security signals.

**Final principle:** Auvora should be the one identity platform a company never outgrows. It is built in gated, verifiable phases, embedded first, standards-correct, and covers what every other product covers without copying their insecure shortcuts.
