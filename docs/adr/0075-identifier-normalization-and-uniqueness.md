# ADR-0075: Identifier normalization and uniqueness

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §7.2 (Identifier), §7.3, §26 (Unicode confusables)
- **Related:** ADR-0018, ADR-0027, ADR-0034, ADR-0035, ADR-0072, ADR-0076

## Context

Login identifiers (email, phone, username) are mutable, user-supplied Unicode strings. Inconsistent normalization causes duplicate accounts, account-takeover via homoglyphs or case tricks, and broken lookups. Never use email as the primary key (v0.2 §5.2).

## Decision

1. **Primary key** is the immutable `principal_id` (ADR-0018). Identifiers are a separate table: `identifiers(project_id, type, value_normalized, value_display, verified_at, primary, principal_id)`.
2. **Types:** `email`, `phone`, `username`, plus `external` (upstream IdP subject, ADR-0034). Each project configures which types are enabled for login and which are required. Default: email required and login-enabled; username and phone disabled.
3. **Uniqueness:** `UNIQUE(project_id, type, value_normalized)` across *active and soft-deleted* principals until hard delete (prevents re-registration takeover during the 30-day window, ADR-0054). Not unique across projects (ADR-0027).
4. **Email normalization:**
   - Trim.
   - Unicode NFC.
   - Lowercase the **domain** via IDNA 2008/UTS #46 to A-label.
   - Lowercase the local part (default `identifiers.email.case_insensitive_local = true`).
   - **No** provider-specific rules (no Gmail dot or `+tag` stripping), because they aren't universally valid.
   - Optional per-project flag `identifiers.email.block_plus_alias` for abuse control.
   - Validation: RFC 5321 length limits, no quoted local parts, no IP-literal domains (by default).
5. **Phone normalization:** E.164 via a libphonenumber-equivalent (`phonenumber` crate) with a project default region. Reject numbers that aren't valid-for-region when `identifiers.phone.strict = true` (default).
6. **Username normalization:**
   - NFKC + case fold (Unicode default caseless matching) for comparison.
   - Allowed characters configurable (default `[a-z0-9._-]`, 3–64 chars, Unicode opt-in).
   - When Unicode is enabled: **UTS #39** mixed-script and confusable *skeleton* checks. Uniqueness is enforced on the skeleton as well (`value_skeleton` unique).
   - Reserved names list (`admin`, `root`, `support`, `auvora`, …, configurable).
7. **Display value** is preserved as entered (post-NFC) for UX. Comparisons always use the normalized value.
8. **Verification:**
   - An identifier is `unverified` until proven via OTP/link or trusted upstream assertion (ADR-0034).
   - Unverified identifiers can't be used for recovery, linking or org domain auto-join.
9. **Change of identifier:**
   - Requires re-authentication (`max_age` 5 min) and verification of the new value.
   - The old value receives a notification with a 72 h revert link (email-change takeover defense).
10. **Lookup storage:** the normalized value is stored with a blind index (ADR-0072) for exact lookup. The plaintext is encrypted when the project enables `identifiers.encrypt_at_rest` (default false for email; true for phone in regulated mode).

## Rationale

Deterministic, standards-based normalization (IDNA, E.164, UTS #39) closes known takeover classes without provider-specific heuristics.

## Alternatives considered

- **Gmail-style dot stripping**: wrong for other providers; surprises users.
- **Case-sensitive emails**: technically RFC-correct, practically harmful.

## Consequences

- Positive: no duplicate or confusable accounts.
- Negative: the strict rules may reject some exotic but valid addresses. A per-project relax flag exists.
- Risks: changing normalization later needs a migration. Mitigation: store `normalization_version`.

## Fallback / revisit trigger

None. Algorithm changes use `normalization_version` + migration under a new ADR.

## Implementation notes

`auvora-core::identifiers` (pure functions, fuzzed). P1. Importers (ADR-0058) run the same normalization and report collisions.

## Verification

- Fuzz + property tests (idempotence: `norm(norm(x)) == norm(x)`).
- A confusable corpus test from the Unicode `confusables.txt`.
- Collision tests in importers.
