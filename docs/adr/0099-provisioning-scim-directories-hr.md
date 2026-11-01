# ADR-0099: Provisioning: SCIM, directories and HR sources

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §17, §12.4, §24 (SCIM, LDAPv3), §27 P4/P7/P8
- **Related:** ADR-0088, ADR-0089, ADR-0016, ADR-0080, ADR-0096, ADR-0100, ADR-0101, ADR-0058, ADR-0103

## Context

Enterprise B2B and workforce customers expect automatic joiner/mover/leaver handling. That means inbound SCIM from their IdP, outbound SCIM to downstream apps, AD/LDAP sync, and HR-driven sourcing. Each source needs consistent mapping, precedence, idempotency and observability. IdPs implement SCIM with known quirks.

## Decision

1. **SCIM 2.0 server** (`auvora-scim`, role `scim` or `all`):
   - Resources: Users, Groups, EnterpriseUser extension, custom schema extensions (from the profile schema, ADR-0076).
   - Operations: `/ServiceProviderConfig`, `/Schemas`, `/ResourceTypes`, PATCH (RFC 7644 §3.5.2), filter (fuzzed hand-written parser; max depth 10; max length 4 KiB), sort, cursor and index pagination (max page 1,000), Bulk (max 1,000 ops, 4 MiB), ETags.
   - Scope: per-**organization** endpoints `/scim/v2/orgs/{org_id}/...` and per-project endpoints for workforce projects.
   - Auth: per-org bearer tokens (hashed, rotatable, expiring; default 1 year) or OAuth client credentials.
   - **Quirk profiles:** `okta`, `entra`, `onelogin`, `jumpcloud`, `google`, `generic`. Each normalizes PATCH path styles, boolean strings and `active` semantics.
   - **Deactivation semantics:** `active=false` → suspend + revoke all sessions/tokens within the ADR-0033 SLO. DELETE → deprovision per org policy (default: remove membership; delete the user only if it has no other memberships, following the ADR-0054 retention).
   - Provisioning log: every request/response is summarized (bodies with PII are redacted) and retained 30 days. It's visible in the Org Admin Portal (ADR-0089).
2. **SCIM client (outbound):** provisions Auvora users and groups into downstream apps, with CEL attribute mapping, reconciliation (full sync daily by default, incremental via outbox events), dry-run, retries (ADR-0080 semantics) and per-target rate limits. Calls go through the egress guard (ADR-0103).
3. **Directory sync (`auvora-directory`):**
   - LDAP/AD incremental sync via USN/DirSync for AD and `modifyTimestamp` for generic LDAP. Default interval 5 min; full reconciliation daily.
   - Pass-through bind (credentials are never stored unless `import_password_hash` is enabled and the directory exposes hashes).
   - Group mapping; optional write-back of selected attributes; multi-domain forests; LDAPS/StartTLS required (plain LDAP only with an explicit `allow_insecure = true` in dev projects).
4. **HR-driven provisioning** (P8):
   - Connector framework with Workday, BambooHR, SuccessFactors, generic REST, and CSV/SFTP connectors.
   - HR is an **authoritative source** with attribute-level **precedence rules**, e.g. `department: hr > scim > admin`.
   - Lifecycle triggers feed workflows (ADR-0101).
5. **Precedence and conflicts:**
   - Every attribute records `source` and `updated_at`. A lower-precedence write to an attribute owned by a higher source is rejected (SCIM 409 with a detail message) or ignored per config.
   - The default precedence order is `hr > directory > scim > federation_jit > admin > self_service`.
6. **SCIM events (draft) and SSF:** changes are emitted via SSF (ADR-0096) and webhooks (ADR-0080).

## Rationale

Matching IdP behavior through quirk profiles is the difference between a "SCIM supported" checkbox and SCIM that actually works. Precedence rules prevent attribute ping-pong between sources. Everything is embedded and stateful in Postgres.

## Alternatives considered

- **A third-party sync engine:** adds a dependency and breaks the single-binary promise.
- **SCIM server only:** leaves workforce and HR gaps versus Okta and Entra.

## Consequences

- Positive: enterprise-grade JML; WorkOS/Okta parity.
- Negative: the quirk profiles need ongoing maintenance, mitigated by recorded-traffic regression tests.
- Risk: mass deprovisioning caused by a misconfigured source. Mitigation: a **safety threshold**. When one sync run would deactivate more than 10% of users (or > 100), it is held for admin approval.

## Fallback / revisit trigger

If a major IdP changes its SCIM behavior, add a versioned quirk profile (`entra@2027`). Never break existing profiles.

## Implementation notes

- P4: SCIM server + quirk profiles + logs.
- P7: LDAP/AD sync.
- P8: SCIM client, HR connectors, precedence UI.
- Config: `[scim]`, `[directory.<id>]`, `[hr.<id>]`.

## Verification

- Okta and Entra SCIM validator suites pass.
- SCIM filter fuzzing ≥ 72 h.
- Recorded-traffic regression tests per quirk profile.
- Safety-threshold integration test.
- Deactivation-to-revocation latency ≤ 5 s p99.
