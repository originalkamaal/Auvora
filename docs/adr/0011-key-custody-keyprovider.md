# ADR-0011: Key custody via KeyProvider

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D11, §6.1, §23, v0.2 §17
- **Related:** ADR-0006, ADR-0012, ADR-0031, ADR-0072, ADR-0082, ADR-0083, ADR-0092

## Context

Auvora holds high-value keys: token signing, SAML signing and encryption, CA roots and intermediates, webhook signing, cookie HMAC, and key-encryption keys (KEKs) wrapping data-encryption keys (DEKs). v0.2 named OpenBao plus KMS/HSM, but didn't define an abstraction or a minimum-profile option. Requiring OpenBao violates ADR-0006, while HSM/KMS custody is mandatory for high-assurance and FIPS deployments.

## Decision

1. All private-key and KEK operations go through the **`KeyProvider` trait** in `auvora-crypto`:
   - `generate(spec) -> KeyHandle`
   - `sign(handle, alg, digest_or_msg)`
   - `decrypt(handle, alg, ct)` (for JWE/SAML)
   - `wrap(kek, dek)` / `unwrap(kek, wrapped)`
   - `public_key(handle)`
   - `destroy(handle)`
   - `capabilities()` (algorithms, exportability, FIPS)

   Application code never handles raw private-key bytes, except inside the `local` provider.
2. **Providers:**

   | Provider | Phase | Notes |
   |---|---|---|
   | `local` | P1 | Default. Keys stored in Postgres `keys` table, encrypted under the installation **root KEK** (AES-256-GCM). Root KEK from `AUVORA_ROOT_KEY_FILE` (32 random bytes) or `AUVORA_ROOT_PASSPHRASE` → Argon2id-derived. Root KEK held only in memory, zeroized on drop. |
   | `aws-kms` | P1 | Asymmetric KMS keys for signing; symmetric for wrap. |
   | `gcp-kms` | P7 | |
   | `azure-keyvault` | P7 | Including Managed HSM. |
   | `vault-transit` | P7 | HashiCorp Vault / OpenBao Transit engine. |
   | `pkcs11` | P7 | HSMs (Thales Luna, Entrust nShield, Utimaco, YubiHSM 2, CloudHSM, SoftHSM for tests). |

3. Providers are **configurable per key purpose and per project**. For example, signing keys can live in HSM while DEK wrapping uses AWS KMS. Config: `[keys.providers.<name>]` + `[keys.purpose_map] token_signing = "<name>"`. Project-level overrides are an EE feature for HSM. KMS per-project assignment is open.
4. **Key hierarchy:**
   - root KEK (provider-held);
   - per-project KEK (provider-held or wrapped by root);
   - per-project DEKs (wrapped, stored in Postgres; ADR-0072);
   - signing and encryption keys (provider-held, or wrapped by the project KEK for `local`).
5. **Exportability:** HSM/KMS keys are non-exportable. `local` keys are exportable only in wrapped form, for backup (ADR-0082).
6. **Signing throughput:** remote KMS/HSM signing is latency-bound. Auvora mitigates this by caching only *public* keys, using connection pooling, and documenting throughput per provider. Private-key caching outside the provider is forbidden.
7. **Startup safety:** the `local` provider refuses to start in non-dev mode without a root key. A dev mode with an ephemeral root key prints a red warning and sets `auvora_insecure_dev_keys=1`.

## Rationale

One trait gives zero-dependency defaults (embedded-first) and enterprise-grade custody without code-path forks. Per-purpose mapping lets operators put only the highest-value keys in expensive HSMs.

## Alternatives considered

- **OpenBao required:** violates ADR-0006. Rejected; supported as a provider.
- **Keys in config files/env:** weak custody and audit. Rejected (the plan forbids it).
- **Only cloud KMS:** blocks on-prem and air-gapped installs. Rejected.

## Consequences

- Positive: a uniform API; FIPS/HSM readiness; clear backup semantics.
- Negative: remote signing adds latency (KMS ~5–20 ms per op).
- Risks: KMS throttling at high token rates. Mitigation: documented per-provider limits; operators can use the `local` provider with an HSM-wrapped project KEK as a hybrid.

## Fallback / revisit trigger

None.

## Implementation notes

- Crate `auvora-crypto::keyprovider`. All operations are audited (`key.used` is sampled; `key.created/rotated/destroyed` always).
- Key lifecycle states and usage separation are defined in ADR-0082.

## Verification

- A provider conformance test suite runs against `local`, SoftHSM (PKCS#11) and LocalStack/AWS KMS in CI.
- Static lint: no crate outside `auvora-crypto` may depend on private-key types (enforced by `check-layers`).
