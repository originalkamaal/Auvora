# ADR-0037: Bot protection

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D37, §22, §8.2
- **Related:** ADR-0035, ADR-0036, ADR-0074, ADR-0097, ADR-0068, ADR-0103

## Context

Signup abuse, credential stuffing, SMS pumping and OTP/email bombing are driven by bots. CAPTCHA vendors differ in privacy, cost and availability (e.g. air-gapped installs can't call third parties). Always-on CAPTCHAs hurt conversion.

## Decision

1. **Risk-triggered challenges.** The risk engine (ADR-0097) computes a bot score per flow step. A challenge is shown only when `score ≥ bot.challenge_threshold` (default 0.5), or when policy forces one (e.g. `bot.always_on_registration = false` default).
2. **Pluggable challenge providers** (per project):
   - Cloudflare Turnstile (**default recommendation** in Cloud),
   - hCaptcha,
   - Google reCAPTCHA Enterprise,
   - **built-in proof-of-work (PoW)**: no third party, works air-gapped. A client-side SHA-256 puzzle with server-issued, HMAC-signed, single-use challenges; difficulty scales with risk (target 0.5–3 s on mid-range mobile); WCAG-friendly (no visual puzzle).
3. **Enforcement points:** registration, login (after N failures or at a risk threshold), recovery, OTP/magic-link send, invitation acceptance, the device code verification page and DCR (open registration mode).
4. **Verification:**
   - Server-side token verification via the provider API (through the egress guard, ADR-0103), with a 1 s timeout.
   - **Fail mode:** `bot.fail_mode = "pow"` (default). If the vendor is unreachable, the built-in PoW challenge is issued instead, so protection never silently fails open or blocks legitimate users.
5. **Signals contributed:** IP reputation, ASN class (hosting/residential/Tor), velocity, headless-browser indicators from the challenge provider, user-agent consistency, and failed-attempt counters (ADR-0074).
6. **Privacy:** the provider is disclosed in the hosted UI privacy text. Challenge scripts load only on the pages and steps that need them. CSP allowlists are generated per project based on the configured provider (ADR-0068).
7. **Headless/API integration:**
   - The Flow API returns a `challenge` node `{provider, site_key, pow_params}`. SDKs and drop-in components render it.
   - Native apps can use app attestation (App Attest / Play Integrity, ADR-0102) in place of a challenge.
8. **Metrics:** challenge rate, solve rate and blocked attempts per project are shown in console analytics (ADR-0043).

## Rationale

Risk-triggered challenges minimize friction. Multiple vendors cover different customer preferences. The built-in PoW guarantees protection in air-gapped deployments and during vendor outages, with no compromise.

## Alternatives considered

- **Always-on CAPTCHA**: conversion loss. Rejected as default.
- **Single vendor**: lock-in, and unavailable air-gapped. Rejected.
- **Fail open on vendor outage**: an attack window. Rejected.

## Consequences

- Positive: protection everywhere, including offline; low friction.
- Negative: PoW costs CPU on low-end devices. Mitigated by difficulty capping and the attestation alternative.
- Risks and mitigations: PoW farms are covered by combining PoW with rate limits and velocity signals.

## Fallback / revisit trigger

If a provider is deprecated or changes terms adversely, remove it from defaults (minor release).

## Implementation notes

P2 (Turnstile, hCaptcha, PoW); P5 reCAPTCHA Enterprise + attestation integration. Crate `auvora-risk`. Config keys `bot.*` per project.

## Verification

Tests: vendor-unreachable falls back to PoW; replayed challenge token rejected; PoW single-use; CSP includes only the configured vendor; E2E with test keys.
