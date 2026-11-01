# ADR-0072: Secrets at rest and PII tagging

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §6.1, §17 (v0.2), §23
- **Related:** ADR-0011, ADR-0012, ADR-0028, ADR-0054, ADR-0071, ADR-0079, ADR-0082, ADR-0093

## Context

The database holds reversible secrets (TOTP seeds, upstream tokens, SAML/OIDC connection secrets) and PII. A database or backup leak must not reveal them. Privacy operations (DSAR export, erasure, residency, log redaction) need machine-readable knowledge of which columns contain PII.

## Decision

**Secrets classification:**

| Class | Examples | Storage |
|---|---|---|
| Verifier-only | Passwords, recovery codes, API key secrets, client secrets (Auvora-generated), magic-link/OTP codes | **Hashed**: Argon2id for passwords (ADR-0028); HMAC-SHA-256 with a server pepper (KeyProvider) for high-entropy tokens/API keys/OTP; never reversible |
| Reversible secrets | TOTP/HOTP seeds, upstream refresh/access tokens (Token Vault), connection client secrets, SAML/mTLS private keys not in KMS, webhook signing secrets, SMTP/SMS credentials | **Envelope encryption**: AES-256-GCM with a per-project DEK; DEKs wrapped by a KEK in the KeyProvider (ADR-0011); AAD = `table:column:row_id:project_id` |
| Key material | Signing/encryption keys | Per ADR-0082 (KMS/HSM preferred; local keystore encrypted) |

1. **DEK hierarchy:** KEK (KeyProvider) → per-project DEK → optional **per-principal DEK** for sensitive PII columns. The per-principal DEK enables crypto-shredding on hard delete, including in history and backups.
2. **DEK rotation:** annual by default (`crypto.dek_rotation_days = 365`). Re-encryption is lazy on read/write, with a background sweep. KEK rotation rewraps DEKs only.
3. **PII tagging:**
   - Every column declares `pii: none|direct|indirect|sensitive` and `export: true|false` in schema metadata (a Rust derive + migration comment `COMMENT ON COLUMN … IS 'pii=direct'`).
   - `direct`: email, phone, name, address, username.
   - `indirect`: IP, user agent, device ID, upstream subject.
   - `sensitive`: date of birth, government ID references, biometric-derived data, IDV results.
4. **Uses of tags:**
   - DSAR export (ADR-0079) includes all `export: true` PII.
   - Erasure purges/shreds `direct|sensitive` fields and pseudonymizes `indirect`.
   - Log/trace redaction: tagged fields are never logged in clear (ADR-0040).
   - Residency checks (ADR-0019) block tagged fields from cross-cell sinks.
5. **Sensitive columns** (`sensitive`) are always encrypted with a per-principal DEK.
6. **Search over encrypted/hashed identifiers** uses a keyed blind index (HMAC of the normalized value) where lookup is needed.
7. **Memory hygiene:** secrets use `secrecy`/`zeroize` types and never implement `Debug`/`Display`.

## Rationale

Hash where possible, encrypt where reversibility is needed, and shred per principal for GDPR erasure even in backups. Tags make privacy features mechanical rather than manual.

## Alternatives considered

- **Postgres TDE / pgcrypto only**: doesn't protect against an app-level DB dump or backup leak, and has no per-row shredding.
- **Encrypt everything**: breaks indexing and queries; marginal gain.

## Consequences

- Positive: breach blast radius reduced; GDPR erasure is real.
- Negative: KeyProvider latency on DEK unwrap. Mitigation: an in-memory DEK cache (TTL 10 min, `crypto.dek_cache_ttl`), locked memory where available.
- Risks: a lost KEK means data loss. Mitigation: KEK backup procedures (ADR-0082).

## Fallback / revisit trigger

None.

## Implementation notes

`auvora-crypto` (envelope API), `auvora-store` (column codecs, tag registry). P1.

## Verification

- A schema lint: every column has a `pii` tag.
- A test that no `sensitive` column stores plaintext.
- A log-redaction test over tagged fields.
- Crypto-shredding test: after deletion, a backup restore yields undecryptable PII.
