# ADR-0078: Account recovery

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §8.1 (Recovery), §8.2 (downgrade protection), §13.1 (`recovery` flow), §26 (abuse cases)
- **Related:** ADR-0028, ADR-0035, ADR-0036, ADR-0050, ADR-0067, ADR-0094, ADR-0097, ADR-0098

## Context

Recovery is the most common way accounts get taken over. Many platforms protect sign-in with passkeys and then let an attacker reset everything through an email link. That turns AAL3 into AAL1. Auvora supports passwords, passkeys, TOTP, push, OTP channels and federation, so recovery has to respect the strongest factor a user has enrolled, and it must work for CIAM, B2B and workforce without dropping any recovery method.

## Decision

1. **Recovery methods** (all available in the open edition; enabled per project, org and app via flow policy, ADR-0067):
   - `recovery_codes`: 10 single-use codes, 12 characters each, Crockford base32. Stored as Argon2id hashes. Shown once. Regenerating them invalidates the whole previous set.
   - `recovery_passkey`: a second registered passkey or security key, labelled as a recovery credential.
   - `verified_channel`: email or phone OTP / magic link to an identifier verified ≥ 7 days ago (configurable via `recovery.channel_min_age`).
   - `admin_assisted`: an org or project admin initiates recovery. Requires the admin to be at AAL2 or higher and to record a reason. The user must re-prove identity: an IDV connector (ADR-0098) when the project requires IAL2; otherwise a verified-channel OTP plus admin attestation.
   - `trusted_contacts` (optional, off by default): k-of-n approval (default 2-of-3) from pre-designated contacts, each confirming with their own AAL2 sign-in. A 24-hour delay applies before it completes.
   - `upstream_idp`: for federated-only users, recovery is delegated to the upstream IdP. Auvora never resets credentials that it does not own.
2. **Stronger-recovery rule (downgrade protection).** A recovery path may restore access only at an assurance level no higher than the level that path proves. When a user has a phishing-resistant authenticator (AAL3-capable) enrolled:
   - The default is `recovery_codes`, `recovery_passkey`, or `admin_assisted` with IDV.
   - `verified_channel` recovery then requires a **cooling-off delay** (default 72 h, `recovery.downgrade_delay`) during which every enrolled channel is notified and any active session can cancel the recovery with one click.
   - Policy can forbid weaker recovery outright (`recovery.allow_downgrade = false`).
3. **After recovery:**
   - All sessions and refresh-token families are revoked, unless the user explicitly keeps them (default: revoke).
   - Every channel receives a CAEP `credential-change` event and a notification.
   - The user is forced to enroll a new credential before they can continue.
   - The risk engine marks the account `recently_recovered` for 7 days, which triggers step-up on sensitive actions.
4. **Abuse limits:** max 5 recovery initiations per identifier per 24 h, and 20 per IP per hour. Responses are enumeration-resistant (ADR-0035). Recovery links and OTPs are single-use, live for 15 min, and are bound to the initiating browser via the flow ID and a cookie. Cross-device completion requires a code shown on the initiating device.
5. **Support staff** can never reset credentials directly. Support-originated recovery is `admin_assisted`, performed inside a Supervised Support Access session (ADR-0094) with a reason and ticket, and it appears on the user's security activity log.
6. **Configuration:** `[projects.<id>.recovery]` keys `methods`, `channel_min_age`, `downgrade_delay`, `allow_downgrade`, `trusted_contacts.{enabled,k,n,delay}`, `revoke_sessions_on_recovery`.

## Rationale

This keeps every recovery method that competitors offer (codes, email/SMS, admin reset, social recovery, IdP delegation) while making sure no method can silently undo phishing-resistant enrollment. That matches NIST SP 800-63-4 guidance on recovery and authenticator binding.

## Alternatives considered

- **Email-link reset as the universal default.** Rejected: it lets an attacker downgrade AAL3 to AAL1.
- **No recovery for passkey users other than another passkey.** Rejected: it locks out real users, especially in CIAM.
- **Trusted contacts on by default.** Rejected: social-engineering risk. It stays opt-in.

## Consequences

- Positive: recovery honours the strongest credential a user holds. Admins and support get a safe, audited path.
- Negative: the cooling-off delay adds friction, so the UX must explain it clearly.
- Risks: support social engineering is mitigated by the ADR-0094 controls and by IDV.

## Fallback / revisit trigger

If more than 2% of recoveries in projects with phishing-resistant MFA are abandoned during the delay, review the default delay. The stronger-recovery rule itself stays.

## Implementation notes

- Crates: `auvora-flows` (recovery flow), `auvora-authn`, `auvora-risk`, `auvora-events`.
- Recovery codes, verified channel and admin-assisted ship in P1/P2. Trusted contacts and IDV-backed recovery ship in P5.

## Verification

- An abuse-case suite covers downgrade attempts, link replay, cross-device phishing and enumeration timing.
- A property test checks that the restored AAL is never greater than the AAL proven by the recovery path.
