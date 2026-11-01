# Data classification and PII tagging

- **Status:** Draft
- **Date:** 2026-11-01
- **Governing ADRs:** [ADR-0072](../adr/0072-secrets-at-rest-and-pii-tagging.md) (authoritative), [ADR-0039](../adr/0039-audit-tamper-evidence.md) §7, [ADR-0040](../adr/0040-observability-opentelemetry.md), [ADR-0054](../adr/0054-deletion-and-retention-of-identities.md), [ADR-0079](../adr/0079-privacy-dsar-and-consent-records.md), [ADR-0109](../adr/0109-data-partitioning-and-retention.md), [ADR-0019](../adr/0019-cell-architecture-multi-region.md)

This document turns ADR-0072 into the tagging rules every table and struct must follow. If the two disagree, the ADR wins.

## 1. Tags

Every persisted column, and every field of a struct that crosses a log, event or export boundary, carries these tags.

| Tag | Values | Default | Meaning |
|---|---|---|---|
| `pii` | `none` \| `direct` \| `indirect` \| `sensitive` | **none of them; a tag is mandatory** | Privacy class (§3) |
| `export` | `true` \| `false` | `true` if `pii ≠ none`, else `false` | Included in DSAR export (ADR-0079) |
| `secret` | `none` \| `verifier` \| `reversible` \| `key` | `none` | Secret class and storage (§2) |
| `audit` | `allowed` \| `denied` | `denied` | May appear in the `audit_event_pii` sidecar (ADR-0054 §4 `audit_allowed`) |

## 2. Secret classes (ADR-0072)

| `secret` | Examples | Storage | Never |
|---|---|---|---|
| `verifier` | Passwords, recovery codes, API-key secrets, Auvora-generated client secrets, OTP/magic-link codes, refresh tokens, auth codes | Argon2id for passwords (ADR-0028); HMAC-SHA-256 with a KeyProvider pepper for high-entropy values | Reversible storage, logging, export |
| `reversible` | TOTP/HOTP seeds, Token Vault upstream tokens, connection client secrets, webhook signing secrets, SMTP/SMS credentials, non-KMS private keys | AES-256-GCM envelope encryption, per-project DEK wrapped by the KEK; AAD = `table:column:row_id:project_id` | Plaintext at rest, logging, export |
| `key` | Signing/encryption keys, DEKs, KEK | ADR-0011 / ADR-0082 (KMS/HSM preferred) | Leaving the KeyProvider unwrapped |

In memory, secrets use `secrecy`/`zeroize` types with no `Debug` or `Display`.

## 3. PII classes

| `pii` | Examples | At rest | Logs/traces (ADR-0040) | On erasure (ADR-0054) | Cross-cell sinks (ADR-0019) |
|---|---|---|---|---|---|
| `none` | IDs, timestamps, config flags | Plain | Allowed | Kept | Allowed |
| `direct` | Email, phone, name, address, username | Plain or blind-indexed; encrypted where ADR-0072 requires it | **Omitted or hashed** unless `observability.log_pii = true` (dev projects only) | Purged / crypto-shredded | Blocked |
| `indirect` | IP, user agent, device ID, upstream `sub` | Plain | Hashed | Pseudonymized | Blocked |
| `sensitive` | Date of birth, government-ID references, biometric-derived data, IDV results | **Always** encrypted with the per-principal DEK | Never | Crypto-shredded | Blocked |

Lookups on encrypted or hashed identifiers use a keyed blind index: the HMAC of the normalized value (ADR-0075).

## 4. Declaring tags

**Rust:** an `auvora-store` derive registers every field in the tag registry. Untagged fields fail compilation.

```rust
#[derive(Record)]
#[record(table = "users")]
struct UserRow {
    #[tag(pii = "none")]                                 id: Uuid,
    #[tag(pii = "direct", export = true, audit = "allowed")] primary_email: Option<String>,
    #[tag(pii = "sensitive")]                             birth_date: Option<Encrypted<Date>>,
}

#[derive(Record)]
#[record(table = "totp_credentials")]
struct TotpRow {
    #[tag(pii = "none", secret = "reversible")] seed: Encrypted<SecretBytes>,
}
```

**SQL:** every migration adds a column comment with the same tags, in the form `key=value` separated by `;`.

```sql
COMMENT ON COLUMN users.primary_email IS 'pii=direct;export=true;audit=allowed';
COMMENT ON COLUMN totp_credentials.seed IS 'pii=none;secret=reversible';
```

**Verification:** a schema lint (P1, `auvora-store` tests) fails if any column lacks a `pii=` comment, or if the comment disagrees with the Rust registry.

## 5. Erasure (canonical scheme, ADR-0039 §7)

1. PII never enters the hashed audit event body. `audit=allowed` fields go to `audit_event_pii`, encrypted with the subject's per-principal DEK. The chain covers `HMAC-SHA256(project_audit_pii_key, canonical_pii)`.
2. Purge (after the 30-day soft delete, D54):
   - destroy the per-principal DEK;
   - delete the sidecar rows;
   - purge `direct`/`sensitive` columns and pseudonymize `indirect` ones;
   - tombstone `_history` rows (ADR-0071).
3. Chain verification still succeeds after purge, and backups become unreadable for that principal.
4. Legal hold blocks purge (ADR-0109 §3).

## 6. Retention

Retention is per table family as defined in [ADR-0109](../adr/0109-data-partitioning-and-retention.md) §1. It is configured with `[retention.<family>] days = N` and is capped at those defaults in OSS. Tags don't change retention. Erasure (§5) applies within any retention window.

## 7. Classification of major entities (plan §7)

| Entity | Key fields → `pii` / `secret` | Notes |
|---|---|---|
| Tenant | name → `direct` (if a person's name), billing contact → `direct` | Billing data lives in the private `cloud/` systems (ADR-0063) |
| Project, Application, Resource server, Policy, Hook | config → `none`; hook/webhook secrets → `reversible` | Integrity-critical rather than private |
| User | id → `none`; profile names/address → `direct`; birth date, IDV results → `sensitive`; `public_metadata`/`private_metadata`/`unsafe_metadata` → `direct` (conservative default) | Profile-schema fields declare their own tag (ADR-0076) |
| Identifier | email/phone/username → `direct` (blind index for lookup) | `audit=allowed` for email at event time |
| Credential | password verifier, recovery codes → `verifier`; TOTP seed → `reversible`; passkey public key → `none`; passkey AAGUID → `indirect` | |
| Linked identity | upstream `sub` → `indirect`; upstream claims → `direct` | |
| Organization | name, domains → `none` (business data) | Member lists are PII via Membership |
| Membership, Group, Role | links → `none`; membership attributes → declared per attribute | |
| OAuth client | `client_id` → `none`; client secret → `verifier` (Auvora-generated) or `reversible` (upstream) | |
| Session, Device | IP, user agent, device ID → `indirect`; session/refresh secrets → `verifier` | |
| Grant, Delegation | scopes/RAR → `none`; subject/actor IDs → `none` | Shown in the account portal (ADR-0079) |
| Support session | ticket reference, reason text → `direct` (may contain PII) | Customer-visible ledger (ADR-0094) |
| Agent, Service account, Workload | config → `none`; credentials → `verifier`/`key` | Owner reference → `none` |
| Token Vault entry | upstream tokens → `reversible` + per-principal DEK | Shredded on user erasure |
| Key | material → `key` | ADR-0082 |
| Audit event | body → `none` by construction; PII only in the sidecar | ADR-0039 §7 |
| Risk evaluation | IP, device signals → `indirect` (hashed signals only) | ADR-0097; 30-day retention |
