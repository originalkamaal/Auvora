# ADR-0012: Crypto provider: aws-lc-rs

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D12, §23, §5.6
- **Related:** ADR-0011, ADR-0031, ADR-0059, ADR-0082, ADR-0083, ADR-0047

## Context

The Rust crypto ecosystem offers `ring`, `aws-lc-rs` and the RustCrypto crates. Auvora needs:
- TLS, JOSE (JWS/JWE), WebAuthn (COSE), X.509/CA, AES-GCM envelope encryption, HMAC, HKDF, Argon2, and post-quantum algorithms;
- a FIPS 140-3 validated path for regulated and government customers (ADR-0059).

Mixing providers duplicates audit surface and breaks FIPS claims.

## Decision

1. **aws-lc-rs** is the single cryptographic provider for all primitives it offers:
   - TLS: rustls with the `aws-lc-rs` provider;
   - signatures: ECDSA P-256/P-384, RSA PKCS#1 v1.5/PSS, Ed25519;
   - AEAD: AES-GCM, ChaCha20-Poly1305;
   - HMAC, HKDF, SHA-2;
   - ECDH, RSA-OAEP;
   - ML-KEM (hybrid TLS); ML-DSA when exposed and stable (ADR-0083).
2. **Allowed exceptions** (algorithms aws-lc-rs doesn't provide), each isolated behind `auvora-crypto` and reviewed:
   - **Argon2id:** RustCrypto `argon2` crate (audited, constant-time). Password hashing isn't a FIPS-approved primitive. In FIPS mode, PBKDF2-HMAC-SHA-256 via aws-lc-rs is available for deployments that must use only approved algorithms (ADR-0028).
   - **Legacy hash verification for imports** (bcrypt, scrypt, Firebase-scrypt, MD5/SHA-1 salted): RustCrypto crates, verify-only, never for new hashes.
   - **XML-DSig/XML-Enc:** libxmlsec1 inside the SAML sandbox (ADR-0014). In FIPS mode it is configured with OpenSSL 3 FIPS provider builds.
3. **FIPS build:**
   - Cargo feature `fips` uses `aws-lc-fips-sys`;
   - power-on self-tests run at boot (abort on failure);
   - algorithm allowlist restricted to FIPS-approved (no Ed25519 until approved in the validated module version; no ChaCha20);
   - `/v1/system/info` reports `fips: true`.
   Built and published as `auvora-fips` and `auvora-ee-fips` images (EE).
4. **Forbidden:** OpenSSL linkage in the main process, custom primitives, `ring` (to keep one provider), and JWT libraries that bring their own crypto backends. JOSE is implemented in `auvora-jose` on aws-lc-rs.
5. **Randomness:** all randomness comes from aws-lc-rs `SystemRandom` (OS CSPRNG). `rand::thread_rng` is forbidden for security values (clippy disallowed-methods).
6. **Constant-time comparisons** for secrets use `aws_lc_rs::constant_time::verify_slices_are_equal` or `subtle`.

## Rationale

aws-lc-rs has a FIPS 140-3 validated module, PQ support, is maintained by AWS, and is rustls' default provider. One provider minimizes audit surface and makes FIPS claims credible.

## Alternatives considered

- **ring:** no FIPS validation, limited algorithm set (no RSA keygen, no PQ). Rejected.
- **RustCrypto everywhere:** not FIPS validated; broad but uneven audit coverage. Rejected except for listed gaps.
- **OpenSSL (`openssl` crate):** C attack surface and version fragmentation. Rejected for the main process.

## Consequences

- Positive: FIPS path; PQ readiness; a single dependency to track.
- Negative: aws-lc-rs needs a C toolchain + CMake (and Go for FIPS builds) at build time, which makes cross-compilation heavier.
- Risks: aws-lc-rs API churn. Mitigation: wrap all usage in `auvora-crypto`; nothing else imports aws-lc-rs directly.

## Fallback / revisit trigger

If aws-lc-rs loses active maintenance or FIPS validation lapses without renewal for > 12 months, a superseding ADR picks a replacement behind the same `auvora-crypto` facade.

## Implementation notes

- P0: `auvora-crypto` facade, `auvora-jose` skeleton with fuzz targets, CI build matrix (`default`, `fips`).
- `.deny.toml` bans `ring`, `openssl`, `openssl-sys` outside the SAML worker crate.

## Verification

- `cargo deny check bans` in CI.
- A FIPS-build CI job runs the full test suite with the FIPS feature and asserts the self-tests ran.
- Wycheproof test vectors run for every primitive exposed by `auvora-crypto`.
