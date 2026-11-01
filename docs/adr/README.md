# Auvora Architecture Decision Records

Process and template: [ADR-0000](0000-record-architecture-decisions.md). Source plan: [`docs/product/plan.md`](../product/plan.md) v1.0. In ADRs, "plan.md §x" always means that file.
ADR-0001…0060 = Decision Register D01…D60. ADR-0061+ = further decisions from the plan body.

## Strategy and product

| ADR | Title | Status |
|---|---|---|
| [0001](0001-delivery-model-and-quality-gates.md) | Delivery model: multi-year, gated by quality | Accepted |
| [0002](0002-target-markets-and-sequencing.md) | Target markets and sequencing | Accepted |
| [0003](0003-business-model-open-core-and-cloud.md) | Business model: open-core + Auvora Cloud | Accepted |
| [0004](0004-licensing-agpl-apache-commercial-cla.md) | Licensing: AGPL server, Apache SDKs, commercial EE, CLA | Accepted |
| [0061](0061-product-naming-and-namespaces.md) | Product naming and namespaces | Accepted |
| [0062](0062-edition-boundary.md) | Edition boundary (no SSO tax) | Accepted |
| [0063](0063-auvora-cloud-operating-model.md) | Auvora Cloud operating model and launch regions | Accepted |
| [0064](0064-phase-gated-roadmap-and-release-labels.md) | Phase-gated roadmap and release labels | Accepted |
| [0065](0065-product-non-goals.md) | Product non-goals | Accepted |
| [0066](0066-excluded-legacy-grants-and-secure-equivalents.md) | Excluded legacy mechanisms and secure equivalents | Accepted |

## Architecture and runtime

| ADR | Title | Status |
|---|---|---|
| [0005](0005-modular-monolith-single-binary-roles.md) | Modular monolith, single binary with roles | Accepted |
| [0006](0006-minimum-deployment-profile-postgres-only.md) | Minimum deployment profile: binary + PostgreSQL | Accepted |
| [0007](0007-optional-valkey-cache-with-postgres-fallback.md) | Optional Valkey cache with Postgres fallback | Accepted |
| [0008](0008-event-outbox-and-optional-bus.md) | Transactional outbox and optional event bus | Accepted |
| [0009](0009-control-to-data-plane-config-propagation.md) | Control-plane → data-plane config propagation | Accepted |
| [0010](0010-no-bundled-edge-gateway.md) | No bundled edge gateway | Accepted |
| [0019](0019-cell-architecture-multi-region.md) | Cell architecture for multi-region | Accepted |
| [0073](0073-state-stores-and-fail-modes.md) | State stores, consistency and fail modes | Accepted |
| [0074](0074-rate-limiting.md) | Rate limiting | Accepted |
| [0103](0103-outbound-egress-ssrf-protection.md) | Outbound egress and SSRF protection | Accepted |

## Data

| ADR | Title | Status |
|---|---|---|
| [0017](0017-tenancy-isolation-postgres-rls.md) | Tenancy isolation with Postgres RLS | Accepted |
| [0018](0018-identifier-format-uuidv7-prefixed.md) | ID format: UUIDv7 + prefixed external IDs | Accepted |
| [0025](0025-database-migrations-expand-contract.md) | Database migrations: expand/contract | Accepted |
| [0042](0042-search-postgres-fts.md) | Search: Postgres FTS, optional OpenSearch | Accepted |
| [0043](0043-analytics-rollups-and-clickhouse-sink.md) | Analytics: rollups, optional ClickHouse | Accepted |
| [0071](0071-entity-history-time-travel.md) | Entity history (identity time travel) | Accepted |
| [0072](0072-secrets-at-rest-and-pii-tagging.md) | Secrets at rest and PII tagging | Accepted |
| [0109](0109-data-partitioning-and-retention.md) | Data partitioning and retention | Accepted |

## Identity domain model

| ADR | Title | Status |
|---|---|---|
| [0026](0026-tenant-project-organization-hierarchy.md) | Tenant → Project → Organization hierarchy | Accepted |
| [0027](0027-per-project-identity-scope.md) | Per-project identity scope | Accepted |
| [0034](0034-account-linking-and-merge.md) | Account linking and merge | Accepted |
| [0054](0054-deletion-and-retention-of-identities.md) | Deletion and retention of identities | Accepted |
| [0075](0075-identifier-normalization-and-uniqueness.md) | Identifier normalization and uniqueness | Accepted |
| [0076](0076-user-profile-schema-and-metadata.md) | User profile schema and metadata | Accepted |
| [0077](0077-anonymous-users.md) | Anonymous users | Accepted |
| [0088](0088-organizations-model.md) | Organizations model | Accepted |

## Cryptography and keys

| ADR | Title | Status |
|---|---|---|
| [0011](0011-key-custody-keyprovider.md) | Key custody via KeyProvider | Accepted |
| [0012](0012-crypto-provider-aws-lc-rs.md) | Crypto provider: aws-lc-rs | Accepted |
| [0031](0031-signing-algorithms-and-key-rotation.md) | Signing algorithms and key rotation | Accepted |
| [0082](0082-key-lifecycle-and-usage-separation.md) | Key lifecycle and usage separation | Accepted |
| [0083](0083-post-quantum-readiness.md) | Post-quantum readiness | Accepted |

## Authentication, sessions and risk

| ADR | Title | Status |
|---|---|---|
| [0028](0028-password-policy-and-hashing.md) | Password policy and hashing | Accepted |
| [0032](0032-session-model.md) | Session model | Accepted |
| [0035](0035-account-enumeration-resistance.md) | Account enumeration resistance | Accepted |
| [0036](0036-communications-providers.md) | Communications providers | Accepted |
| [0037](0037-bot-protection.md) | Bot protection | Accepted |
| [0050](0050-assurance-levels-aal-ial-fal.md) | Assurance levels (AAL/IAL/FAL) | Accepted |
| [0056](0056-push-mfa-and-authenticator-app.md) | Push MFA and Auvora Authenticator | Accepted |
| [0078](0078-account-recovery.md) | Account recovery | Accepted |
| [0087](0087-social-provider-manifests.md) | Social provider manifests | Accepted |
| [0097](0097-risk-engine.md) | Risk engine | Accepted |
| [0098](0098-identity-proofing-connectors.md) | Identity proofing connectors | Accepted |

## OAuth, OIDC and tokens

| ADR | Title | Status |
|---|---|---|
| [0013](0013-own-oauth-oidc-engine.md) | Own OAuth/OIDC engine in Rust | Accepted |
| [0029](0029-access-token-format-and-lifetime.md) | Access token format and lifetime | Accepted |
| [0030](0030-refresh-token-policy.md) | Refresh token policy | Accepted |
| [0033](0033-revocation-propagation-slo.md) | Revocation propagation SLO | Accepted |
| [0051](0051-oidf-certification-targets.md) | OIDF certification targets | Accepted |
| [0052](0052-dynamic-client-registration-and-cimd.md) | Dynamic client registration and CIMD | Accepted |
| [0053](0053-consent-model.md) | Consent model | Accepted |
| [0084](0084-client-policy-profiles.md) | Client policy profiles | Accepted |
| [0085](0085-redirect-uri-and-client-registration-rules.md) | Redirect URI and client registration rules | Accepted |
| [0086](0086-delegation-token-exchange-and-actor-chains.md) | Delegation, token exchange and actor chains | Accepted |

## Federation, provisioning and gateways

| ADR | Title | Status |
|---|---|---|
| [0014](0014-saml-implementation-samael-sandboxed.md) | SAML implementation: samael, sandboxed | Accepted |
| [0089](0089-org-admin-portal.md) | Org Admin Portal | Accepted |
| [0096](0096-shared-signals-ssf-caep-risc.md) | Shared Signals (SSF/CAEP/RISC) | Accepted |
| [0099](0099-provisioning-scim-directories-hr.md) | Provisioning: SCIM, directories, HR sources | Accepted |
| [0100](0100-legacy-and-infrastructure-gateways.md) | Legacy and infrastructure gateways | Accepted |
| [0110](0110-verifiable-credentials-and-openid-federation.md) | Verifiable credentials and OpenID Federation | Accepted |

## Machine, workload and agent identity

| ADR | Title | Status |
|---|---|---|
| [0091](0091-agent-identity-token-vault-and-mcp.md) | Agent identity, Token Vault and MCP | Accepted |
| [0092](0092-workload-identity-and-builtin-ca.md) | Workload identity and built-in CA | Accepted |
| [0093](0093-api-keys.md) | API keys (last resort) | Accepted |

## Authorization, governance and support

| ADR | Title | Status |
|---|---|---|
| [0015](0015-layered-authorization-architecture.md) | Layered authorization architecture | Accepted |
| [0016](0016-cel-for-expressions.md) | CEL for expressions | Accepted |
| [0055](0055-control-plane-admin-roles.md) | Control-plane admin roles | Accepted |
| [0057](0057-cloud-staff-access-via-supervised-support.md) | Cloud staff access only via Supervised Support Access | Accepted |
| [0094](0094-supervised-support-access.md) | Supervised Support Access | Accepted |
| [0101](0101-governance-and-privileged-access.md) | Governance and privileged access | Accepted |

## Experience layer and extensibility

| ADR | Title | Status |
|---|---|---|
| [0021](0021-ui-architecture.md) | UI architecture | Accepted |
| [0022](0022-frontend-stack.md) | Frontend stack | Accepted |
| [0038](0038-extensibility-tiers-and-hook-points.md) | Extensibility tiers and hook points | Accepted |
| [0067](0067-flow-api-single-engine.md) | Flow API as the single engine | Accepted |
| [0068](0068-hosted-ui-security-model.md) | Hosted UI security model | Accepted |
| [0069](0069-drop-in-components.md) | Drop-in components | Accepted |
| [0070](0070-config-as-code-and-environment-promotion.md) | Config-as-code and environment promotion | Accepted |
| [0080](0080-event-envelope-and-webhook-delivery.md) | Event envelope and webhook delivery | Accepted |
| [0081](0081-wasm-action-sandbox.md) | Wasm action sandbox | Accepted |
| [0090](0090-custom-domains-and-tls.md) | Custom domains and TLS | Accepted |
| [0102](0102-mobile-and-native-apps.md) | Mobile and native apps | Accepted |
| [0104](0104-i18n-and-accessibility.md) | Internationalization and accessibility | Accepted |

## APIs and developer ecosystem

| ADR | Title | Status |
|---|---|---|
| [0020](0020-api-conventions-rest-openapi.md) | API conventions: REST + OpenAPI 3.1 | Accepted |
| [0023](0023-monorepo-layout.md) | Monorepo layout | Accepted |
| [0024](0024-sdk-strategy-and-tiers.md) | SDK strategy and tiers | Accepted |
| [0048](0048-documentation-site.md) | Documentation site | Accepted |
| [0058](0058-migration-importers.md) | Migration importers | Accepted |

## Audit, privacy and compliance

| ADR | Title | Status |
|---|---|---|
| [0039](0039-audit-tamper-evidence.md) | Audit tamper evidence | Accepted |
| [0059](0059-compliance-targets.md) | Compliance targets | Accepted |
| [0079](0079-privacy-dsar-and-consent-records.md) | Privacy, DSAR and consent records | Accepted |
| [0095](0095-customer-audit-log-api.md) | Customer Audit Log API | Accepted |

## Operations

| ADR | Title | Status |
|---|---|---|
| [0040](0040-observability-opentelemetry.md) | Observability: OpenTelemetry | Accepted |
| [0041](0041-no-default-telemetry.md) | No default telemetry | Accepted |
| [0044](0044-supported-platforms.md) | Supported platforms | Accepted |
| [0045](0045-release-and-support-policy.md) | Release and support policy | Accepted |
| [0105](0105-nfr-targets-and-benchmarks.md) | NFR targets and benchmarks | Accepted |

## Security engineering and process

| ADR | Title | Status |
|---|---|---|
| [0046](0046-ci-and-supply-chain-security.md) | CI and supply-chain security | Accepted |
| [0047](0047-rust-coding-standards.md) | Rust coding standards | Accepted |
| [0049](0049-threat-modelling-method.md) | Threat modelling method | Accepted |
| [0060](0060-interoperability-test-targets.md) | Interoperability test targets | Accepted |
| [0106](0106-testing-and-conformance-strategy.md) | Testing and conformance strategy | Accepted |
| [0107](0107-engineering-process-and-definition-of-done.md) | Engineering process and Definition of Done | Accepted |
| [0108](0108-vulnerability-disclosure-and-advisories.md) | Vulnerability disclosure and advisories | Accepted |
| [0111](0111-repository-organization-and-file-placement.md) | Repository organization and file placement | Accepted |
| [0112](0112-commit-policy.md) | Commit policy | Accepted |
