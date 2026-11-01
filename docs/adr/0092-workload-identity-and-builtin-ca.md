# ADR-0092: Workload identity and built-in CA

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §10, §5.6 (Workload row), §27 P5, §2.1 (Entra workload identity federation)
- **Related:** ADR-0011, ADR-0082, ADR-0083, ADR-0086, ADR-0091, ADR-0093, ADR-0016, ADR-0103

## Context

Machine identity must not rely on long-lived shared secrets. Workloads already hold platform-issued identity: Kubernetes service-account tokens, cloud instance identity, CI OIDC tokens and SPIFFE SVIDs. Auvora must federate those, and also provide its own short-lived certificates without requiring SPIRE or an external PKI (embedded-first).

## Decision

1. **Concepts** (plan §10): *service account* = an Auvora-managed principal with credentials (`sa_`); *workload* = an externally attested principal with no stored secret (`wl_`). An OAuth client is a protocol artifact linked to either.
2. **Workload federation.** A trust configuration per source contains: `issuer`, a key source (JWKS URL via the egress guard, a static JWKS, or a cloud API), `audience`, a **CEL trust policy** over incoming claims, a principal mapping (static `wl_` ID or a template), allowed audiences and scopes, and max token TTL (default 15 min, max 1 h). Sources:
   - **Kubernetes:** projected SA tokens. Issuer and JWKS per cluster, or a TokenReview API call for clusters without public OIDC.
   - **AWS:** signed `sts:GetCallerIdentity` request (presigned-request pattern), verified by calling STS through egress; plus IAM Roles Anywhere-compatible X.509.
   - **GCP:** identity tokens (`https://accounts.google.com`).
   - **Azure:** managed identity tokens (Entra issuer, tenant-pinned).
   - **CI:** GitHub Actions, GitLab CI, Bitbucket Pipelines, CircleCI, Buildkite, Terraform Cloud OIDC.
   - **SPIFFE:** JWT-SVID (trust-bundle endpoint) and X.509-SVID (mTLS).
   - **Generic OIDC issuer.**
   Exchange happens via RFC 8693 (`subject_token_type=urn:ietf:params:oauth:token-type:jwt`) or RFC 7523 JWT bearer. Replay protection on `jti` is applied where present; otherwise tokens are bound by short max age (≤ 10 min).
3. **Built-in CA:**
   - A per-project (or per-tenant, configurable) CA hierarchy: offline-capable root → online issuing intermediate.
   - Keys live in the KeyProvider (`ca_root`, `ca_intermediate`, ADR-0082). The root can be KMS/HSM-backed.
   - Importing an existing enterprise intermediate (signed by the customer's PKI) is supported.
   - Issues:
     - mTLS client certificates for service accounts and workloads (default TTL 24 h, max 30 days; SAN URI `spiffe://{trust-domain}/...` or `urn:auvora:{principal}`).
     - X.509-SVIDs compatible with the SPIFFE Workload API.
     - Server certificates for internal services (optional).
     - SSH certificates later (ADR-0100).
   - Revocation via CRL (published every 1 h) and OCSP responder (built-in). Short lifetimes are the primary control.
   - Issuance endpoints: an ACME-compatible endpoint (RFC 8555, internal device/service enrolment), a REST CSR endpoint, and a SPIFFE Workload API node mode (`auvora serve --roles=node`, an optional node-local sidecar; named `node` to avoid confusion with AI *Agent* principals, ADR-0091).
4. **SPIRE interop:** SPIRE can federate with Auvora's trust bundle (SPIFFE Federation bundle endpoint), and Auvora can trust SPIRE bundles. SPIRE is **optional**.
5. **Transaction Tokens:** per ADR-0086 for internal call chains.
6. **Lifecycle and governance:**
   - Required `owner` and `purpose`; environment tags.
   - `last_used_at`, credential age, rotation policy (keys max age default 365 d; alert at 80%).
   - Unused identity detection (default 90 d → flagged, 180 d → auto-disable if policy is enabled).
   - Inventory API; risk signals (new ASN, burst, scope anomaly).
7. **`auvora-pg`:** a helper library and Postgres extension-free SQL functions that map validated JWT claims into `SET LOCAL` session settings for application RLS (plan §10).
8. **Configuration:** `[projects.<id>.workload]` with `trust.<name>.{source,issuer,audience,policy,mapping,max_ttl}`, `ca.{mode=project|tenant,root_provider,cert_ttl,max_ttl,crl_interval}`.

## Rationale

Federation removes stored secrets for every major platform, and the embedded CA gives SPIFFE-grade mTLS without running SPIRE, while still interoperating with it. That covers the whole machine-identity space in one binary.

## Alternatives considered

- **Require SPIRE.** Rejected: dependency sprawl. It's kept as an optional peer.
- **Use an external PKI only.** Rejected: not embedded-first. Import is supported.

## Consequences

- Positive: zero-secret machine authentication across clouds, Kubernetes and CI.
- Negative: CA operation brings responsibility (root ceremony docs, HSM guidance).
- Risks: a permissive CEL trust policy (e.g. any repo). Mitigated by console lint warnings for missing `repository`/`namespace` conditions and templates per source.

## Fallback / revisit trigger

If the AWS signed-STS pattern changes, switch to AWS IAM OIDC (outbound identity federation) as the primary path.

## Implementation notes

- Crate: `auvora-workload` (federation, CA, Workload API, `auvora-pg`). The `node` role (`--roles=node`) is part of the same binary; it is added to the ADR-0005 role list.
- Ships in P5.

## Verification

- Trust-policy negative tests per source (wrong repo, branch, namespace, audience).
- A CA test suite (path validation with OpenSSL and rustls-webpki, CRL/OCSP).
- SPIFFE interop with SPIRE in CI.
