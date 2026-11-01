# ADR-0035: Account enumeration resistance

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D35, §26, §27 P2 exit criteria
- **Related:** ADR-0028, ADR-0034, ADR-0037, ADR-0067, ADR-0074, ADR-0097

## Context

Login, registration, recovery and identifier-first flows can reveal whether an account exists through response content, status codes, redirects or timing. Enumeration fuels credential stuffing, phishing and privacy harm. Some products deliberately disclose existence for UX, and customers must be able to choose.

## Decision

1. **Default: resistant** (`enumeration.mode = "resistant"` per project).
   - **Login (password):** identical error ("Invalid credentials") and status for unknown users, wrong passwords and disabled accounts. For unknown users a dummy Argon2id verification runs with identical parameters (ADR-0028).
   - **Identifier-first:**
     - The next step shown is computed from policy, not from account existence. For unknown identifiers the UI proceeds to a plausible step (password or passkey prompt; OTP "sent" message).
     - Home realm discovery by **domain** (not by account) is allowed, since it reveals org configuration, not users.
   - **Registration:** "Check your email to continue" for both new and existing addresses. Existing addresses receive an "account already exists, sign in or reset" email instead of a verification email.
   - **Recovery / magic link / OTP:** always "If an account exists, we sent a message." Delivery timing is decoupled through the outbox (ADR-0008), so response timing is independent of delivery.
   - **Passkey conditional UI:** discoverable credentials avoid identifier disclosure. `allowCredentials` lists are never returned for unknown users; a deterministic fake list is derived from an HMAC of the identifier (key usage `enumeration_hmac`) so responses look stable.
2. **Timing:**
   - Server responses for these endpoints are padded to a per-endpoint floor (`enumeration.min_response_ms`, default 250 ms for credential endpoints) with jitter.
   - CI verifies p95 timing difference ≤ 5 ms between existing and non-existing identifiers on the reference setup.
3. **Opt-out:** `enumeration.mode = "disclosing"` enables explicit messages (e.g. "No account found, sign up?"). The console shows a security warning, and it is recorded in the audit log. Organization-level policy can't loosen a project's resistant setting.
4. **Admin and SCIM APIs** are authenticated and not subject to this ADR. The Admin API still requires authorization to look up users.
5. **Rate limits and bot protection** apply per identifier and per IP on all these endpoints (ADR-0074, ADR-0037), so probing is throttled even in disclosing mode.

## Rationale

Resistance by default protects users. Making it a project setting serves CIAM products that prioritize conversion. Timing padding plus dummy hashing addresses side channels, which are the usual gap.

## Alternatives considered

- **Disclosing by default**: insecure default. Rejected.
- **Resistant with no opt-out**: some products legitimately prefer disclosure. Rejected; we meet the need with an explicit, audited opt-out.

## Consequences

- Positive: closes enumeration via content and timing.
- Negative: slightly slower credential endpoints (response floor); more complex email logic.
- Risks and mitigations: UX confusion for existing users trying to register is mitigated by the "account exists" email with a sign-in link.

## Fallback / revisit trigger

None.

## Implementation notes

P1 login/recovery/registration resistance; P2 identifier-first, passkeys, OTP; timing test in CI from P2. Crates `auvora-flows`, `auvora-authn`.

## Verification

Statistical timing test (10k samples) in CI; response-diff tests across existing and non-existing identifiers for every flow; abuse-suite entry "account enumeration".
