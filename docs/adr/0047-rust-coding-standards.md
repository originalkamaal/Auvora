# ADR-0047: Rust coding standards

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D47, §26, §29
- **Related:** ADR-0046, ADR-0012, ADR-0014, ADR-0106, ADR-0107, ADR-0023

## Context

Memory safety is one reason for choosing Rust. It must not be eroded by `unsafe`, panics on the request path, or inconsistent error handling in security code.

## Decision

1. **Toolchain:** stable Rust pinned in `rust-toolchain.toml`. **MSRV = latest stable − 2 minors**, bumped deliberately in a PR. Edition: latest stable edition.
2. **Unsafe:**
   - `#![forbid(unsafe_code)]` in every crate except allowlisted FFI crates: `auvora-saml` (libxmlsec1) and `auvora-crypto` (only if needed beyond the aws-lc-rs safe API).
   - In allowlisted crates, `#![deny(unsafe_op_in_unsafe_fn)]`; every `unsafe` block carries a `// SAFETY:` comment reviewed by a CODEOWNER.
   - `cargo geiger` reports in CI.
3. **Lints** (workspace `Cargo.toml` `[workspace.lints]`): `clippy::pedantic` = warn, CI denies warnings. Also denied: `clippy::unwrap_used`, `clippy::expect_used` (allowed in tests), `clippy::panic`, `clippy::indexing_slicing` in protocol crates, `clippy::arithmetic_side_effects` in `auvora-{jose,oauth,saml,authz,crypto}`, `clippy::dbg_macro`, `clippy::todo`, `rust_2018_idioms`, `missing_docs` on public items of library crates.
4. **Errors:** `thiserror` in libraries, a single `auvora_core::Error` taxonomy mapped to RFC 9457 problem types at the API edge (ADR-0020). No `anyhow` in library crates. Errors never contain secrets. Display strings are safe to log.
5. **Panics:** no panics on request paths. `panic = "unwind"` with a tower layer that converts panics to 500 + audit event; tests assert there are no panic paths reachable from fuzz targets.
6. **Secrets:** types wrapping secrets use `secrecy::SecretBox`/`zeroize` and implement redacted `Debug`. Comparison of MACs, tokens and codes uses `subtle::ConstantTimeEq`. `Serialize` is never derived on secret types.
7. **Async:** Tokio only. CPU-heavy work (Argon2, RSA signing, Wasm, CEL with large inputs) runs in bounded `spawn_blocking` pools or a dedicated rayon pool, never on the async worker threads. No blocking I/O in async.
8. **Parsing:** every untrusted-input parser (JOSE, JSON with size limits, SAML XML, SCIM filter, CEL, CBOR/WebAuthn, LDAP BER, RADIUS, X.509, DPoP) has a cargo-fuzz target and a property-test suite. Recursion depth and size limits are mandatory.
9. **Regex:** only the `regex` crate (linear time). No `fancy-regex`/PCRE for untrusted input.
10. **Time:** a `Clock` trait is injected everywhere. `SystemTime::now()` is banned outside `auvora-core::time` (clippy `disallowed_methods`).
11. **Randomness:** `aws-lc-rs` `SystemRandom` or `rand::rngs::OsRng` only. `thread_rng` is banned for security values.
12. **Logging:** `tracing` with structured fields. Log events use redaction helpers. Fields named `password|secret|token|code|otp|key` are blocked by a custom lint (dylint) unless wrapped in `Redacted`.
13. **Style:** `rustfmt` default + `imports_granularity = "Crate"`; module-per-feature layout; public API documented with examples (doctests run).

## Rationale

These rules turn security expectations into mechanically enforced checks rather than review-dependent conventions.

## Alternatives considered

- **Allow `unsafe` with review**: rejected because forbidding it by default makes exceptions visible.
- **`anyhow` everywhere**: rejected because it loses typed error mapping needed for protocol error responses.

## Consequences

- Positive: a smaller audit surface; consistent error and redaction behavior.
- Negative: pedantic lints slow initial development; some `allow` attributes need justification comments.

## Fallback / revisit trigger

Individual lints can be relaxed per crate via a PR with justification. A blanket change requires a new ADR.

## Implementation notes

P0: workspace lint config, `clippy.toml` (`disallowed-methods`, `disallowed-types`), dylint redaction lint, fuzz scaffolding under `fuzz/`.

## Verification

CI clippy with `-D warnings`; `cargo geiger` diff check; fuzz targets enumerated against the parser registry in `fuzz/README.md`.
