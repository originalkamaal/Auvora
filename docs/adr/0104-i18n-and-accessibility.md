# ADR-0104: Internationalization and accessibility

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §13.3, §15, §27 P3 (exit criteria)
- **Related:** ADR-0021, ADR-0022, ADR-0036, ADR-0068, ADR-0069, ADR-0075, ADR-0107

## Context

Login, registration, recovery and email/SMS messages are the most user-facing parts of the product and must work for every locale and ability. Enterprise and public-sector buyers require WCAG conformance (and EN 301 549 / Section 508 reports). CIAM needs RTL scripts and locale-aware formatting.

## Decision

1. **Message format:** ICU MessageFormat 2-compatible strings (plural, select, gender), stored as JSON bundles per locale with stable message IDs (`login.password.label`). The source locale is `en`.
2. **Locales shipped:** 20 at P3 (en, es, fr, de, it, pt-BR, pt-PT, nl, pl, tr, ru, uk, ar, he, hi, ja, ko, zh-Hans, zh-Hant, id), growing to 40+ by GA. Translation goes through a TMS with community contributions. Machine translation is never shipped without human review for security-relevant messages.
3. **Locale negotiation order:**
   1. `ui_locales` param
   2. the user's saved locale
   3. the org default
   4. `Accept-Language`
   5. the project default
   6. `en`.

   Fallback chain is by language then region (`pt-BR → pt → en`).
4. **Tenant overrides:** per project, org and app, any message ID can be overridden per locale. Overrides are versioned with themes (ADR-0068, ADR-0070). Missing overrides fall back to the shipped text.
5. **Formatting:** dates, numbers and phone numbers are locale-formatted (ICU4X in Rust for emails/SMS; `Intl` in the UI). Server-side messages (emails, SMS, errors) use the same bundles (ADR-0036).
6. **RTL:** full mirroring via CSS logical properties. Every component is tested in `ar` and `he`.
7. **Accessibility target:** **WCAG 2.2 AA** for the hosted UI, account portal, drop-in components, admin console and Org Admin Portal. AAA where feasible for the login flow (contrast, target size).
8. **Accessibility rules:**
   - Radix primitives (ADR-0022).
   - Every input labelled; error messages associated via `aria-describedby`; focus management on step change.
   - No time limits without an extension (OTP expiry shows a resend option).
   - CAPTCHA always offers an accessible alternative (ADR-0037 PoW fallback).
   - Passkey and OTP inputs support autofill (`autocomplete="one-time-code"`, `webauthn`).
   - Theme tokens are validated for contrast at save time (block below 4.5:1 for text).
9. **Conformance reporting:** a VPAT/ACR (WCAG 2.2, EN 301 549, Section 508) is published per major release.

## Rationale

Shared bundles across UI, emails and errors avoid drift. Contrast validation at theme save stops tenants from breaking accessibility through branding.

## Alternatives considered

- **Fluent or gettext:** good tools, but ICU MessageFormat has the broadest TMS and JS/Rust support.
- **AA only for the login flow:** the admin and org portals are also used by enterprise staff with accessibility needs.

## Consequences

- Positive: enterprise and public-sector eligibility; global CIAM.
- Negative: translation upkeep. Mitigations: message IDs are frozen per release, and a CI check flags untranslated strings as warnings, not failures, except for security messages.

## Fallback / revisit trigger

None.

## Implementation notes

- P1: minimal hosted UI uses bundles from day one.
- P3: 20 locales, axe CI, overrides.
- GA: 40+ locales, VPAT.

## Verification

- axe-core shows 0 violations in Playwright for every flow state and every component in light/dark and LTR/RTL (P3 exit criterion).
- A manual screen-reader audit (VoiceOver, NVDA, TalkBack) per major release.
- Contrast validation unit tests.
- A missing-message-ID CI check.
