# ADR-0028: Password policy and hashing

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D28, §8.1, §8.2, §26
- **Related:** ADR-0035, ADR-0050, ADR-0058, ADR-0074, ADR-0078, ADR-0012

## Context

Passwords remain the most common credential. Auvora needs:
- modern policy (NIST SP 800-63-4),
- safe hashing,
- migration from every competitor's hash formats,
- protection against credential stuffing and hashing-cost DoS,
- breach screening that also works air-gapped.

## Decision

1. **Default policy** (per project, overridable per org and per app with stricter-only overrides):
   - Minimum length **8**; **15** when the password is the only factor permitted for the account (`password.min_length_single_factor = 15`). Maximum length **128** characters (UTF-8, NFKC-normalized before hashing; length counted in Unicode code points).
   - No composition rules and no periodic expiry by default. Orgs may enable `password.max_age_days` where regulation requires it, and the console warns.
   - All printable Unicode and spaces are allowed. Paste is allowed.
   - Blocklists: breached passwords, the user's own identifiers, the project name, and a context-specific word list (configurable).
   - History: `password.history = 0` by default (configurable up to 24).
2. **Hashing:** Argon2id (`argon2` crate). Baseline `m = 19,456 KiB, t = 2, p = 1`. These are tunable per installation (`[password.argon2]`) with a startup benchmark warning if a hash takes < 50 ms or > 1 s.
   - Encoded in PHC string format.
   - An optional **pepper** (HMAC-SHA-256 with a KeyProvider-held key, ADR-0011, key usage `password_pepper`) is applied before Argon2 when `password.pepper_enabled = true`. Default: true for Cloud, false self-hosted.
3. **Rehash on login** whenever parameters, the algorithm or the pepper version differ from current.
4. **Imported hashes** are verified natively and rehashed to Argon2id on first successful login:
   - bcrypt, scrypt, Firebase-scrypt, PBKDF2-SHA1/256/512, Argon2i/d/id;
   - salted SHA-1/256/512 and MD5 (legacy; flagged `weak_import`, forced rehash, reported in posture).

   Lazy migration through the `password-verify` hook covers anything else (ADR-0038, ADR-0058).
5. **Hashing DoS guard:**
   - Hashing runs on a dedicated blocking thread pool (`[password] hash_workers = num_cpus`).
   - A semaphore bounds the queue (`hash_queue_max = 4 × workers`). Overflow or a wait longer than `hash_queue_timeout_ms = 2000` returns HTTP 429 with `Retry-After`.
   - Rate limits (ADR-0074) are applied **before** hashing.
6. **Breach screening:**
   - Online: HIBP Pwned Passwords k-anonymity with padding (`password.breach.online = true`), timeout 300 ms, **fail open with an audit flag** (screening failure must not block login), but **fail closed on registration/change** if `password.breach.required = true`.
   - Offline: a bundled binary fuse filter of the top 100M breached passwords, updated each release (`password.breach.offline = true`). Air-gapped profiles use offline only.
   - On login, a breached password triggers forced reset or step-up per policy (`password.breach.on_login = "force_reset" | "notify" | "step_up"`; default `step_up` then reset).
7. Passwords and hashes never appear in logs, audit or events. They are zeroized in memory after use (`zeroize`).
8. Constant-time verification paths: an unknown user runs a dummy Argon2 hash with identical parameters (ADR-0035).

## Rationale

This aligns with NIST 800-63-4 and OWASP recommendations. Importing all common formats makes migration from any competitor painless. The bounded hashing pool prevents attackers from exhausting CPU.

## Alternatives considered

- **bcrypt default**: 72-byte limit, weaker memory-hardness. Rejected.
- **scrypt default**: Argon2id is the current recommendation. Rejected.
- **Composition rules**: contrary to NIST guidance. Rejected as default; not offered except via a custom CEL rule.

## Consequences

- Positive: modern defaults; migration-friendly; DoS-resistant.
- Negative: Argon2 memory cost limits per-node login throughput, so capacity planning is documented (ADR-0105).
- Risks and mitigations: HIBP outages are handled by the offline filter fallback.

## Fallback / revisit trigger

If OWASP or NIST updates its parameter guidance, adjust the defaults in a minor release via rehash-on-login. If a practical Argon2 weakness is published, write a superseding ADR.

## Implementation notes

P1 in `auvora-authn`; imported hash formats are P3 with the importers. Config section `[password]`; per-project policy keys `password.*` via the Admin API.

## Verification

Unit tests per hash format with known vectors; a timing test for unknown vs known users; a load test proving 429 instead of latency collapse at 3× capacity; a secret-leak scanner on logs in E2E.
