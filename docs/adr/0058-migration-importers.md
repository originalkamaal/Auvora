# ADR-0058: Migration importers

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D58, §8.1 (hash import), §17 (import/export), §27 (P3, P10)
- **Related:** ADR-0028, ADR-0038, ADR-0070, ADR-0075, ADR-0076, ADR-0088, ADR-0020

## Context

Switching cost is the main barrier to adopting a new IAM. Users must be able to move without forcing password resets, broken MFA, or changed `sub` values that would break downstream apps.

## Decision

1. **Importer framework** (`auvora migrate` CLI + Admin API `POST /v1/projects/{id}/imports`):
   - pluggable *source adapters* producing a canonical **Auvora Import Format (AIF)**: versioned JSON Lines with JSON Schema covering users, identifiers, credentials (hashes, TOTP seeds, WebAuthn credentials where exportable), linked identities, orgs, memberships, roles, groups, apps/clients, connections and metadata;
   - **dry-run** with a validation report, idempotent re-runs keyed by the source ID, resumable batches, a throughput of ≥ 5,000 users/s per worker on reference hardware, and an audit event per batch.
2. **Source adapters (priority order):**

| Phase | Sources |
|---|---|
| P3 | Auth0, Keycloak, Firebase Auth, AWS Cognito, Supabase Auth, Clerk |
| P10 (earlier on demand) | Okta, Zitadel, Ory (Kratos/Hydra), FusionAuth, authentik, generic CSV/LDIF |

3. **Password hashes:** imported verbatim with an algorithm tag. Supported: bcrypt, scrypt, Firebase-modified scrypt (with project signer key/salt separator), PBKDF2-SHA1/256/512, Argon2i/d/id, Keycloak `pbkdf2-sha256`/`argon2`, salted SHA-1/256/512 and MD5 (legacy, flagged). On next successful login, rehash to Argon2id (ADR-0028). Legacy algorithms can be policy-forced to a reset after N days.
4. **Lazy (trickle) migration:** the `password-verify` sync hook (ADR-0038) calls the old system on first login when no local credential exists. On success, Auvora stores an Argon2id hash and marks the user migrated. A cut-over report shows the remaining users.
5. **Identifiers:** `sub` preservation. The original subject is stored as `legacy_sub`, and per-client claim mapping can emit `legacy_sub` as `sub` for clients migrated from the old IdP (opt-in per client, never for new clients). External IDs map to Auvora prefixed IDs (ADR-0018), with a lookup table.
6. **MFA:** TOTP seeds imported encrypted (when the source exports them). WebAuthn credentials are imported when RP ID compatibility holds (same domain/related origins). Otherwise the user re-enrolls with a grace policy.
7. **Clients:** client IDs can be preserved. Secrets are imported hashed or re-issued. Redirect URIs are validated against ADR-0085, and violations are reported.
8. **Security:** import files are treated as untrusted (schema validation, size limits, fuzzed parser). Admin AAL2 is required. Imported data is encrypted in transit and at rest; staging blobs are deleted after completion (max 7 days).

## Rationale

Hash compatibility + lazy migration + `sub` preservation eliminate forced resets and downstream breakage, which are the main reasons migrations fail.

## Alternatives considered

- **CSV-only import**: rejected because it can't carry MFA, linked identities or orgs.
- **Forced resets**: rejected as a poor UX that blocks adoption.

## Consequences

- Positive: low-friction switching from all major competitors (superset goal).
- Negative: maintaining adapters as source export formats change.

## Fallback / revisit trigger

An adapter whose source format breaks is marked "degraded" in docs and fixed within one minor release.

## Implementation notes

Crate `auvora-migrate` (added to §5.5 layout under `crates/`). AIF schema in `docs/site/reference/aif/`. Adapters are tested with recorded fixture exports.

## Verification

Round-trip tests (import → login with original password for every hash algorithm); `sub` preservation tests; fuzzing of the AIF parser; performance benchmark.
