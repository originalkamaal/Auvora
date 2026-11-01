# ADR-0050: Assurance levels (AAL/IAL/FAL)

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D50, §8.1–8.2, §9.2, §24
- **Related:** ADR-0028, ADR-0032, ADR-0078, ADR-0098, ADR-0097, ADR-0067, ADR-0084

## Context

Policies such as "admins need phishing-resistant MFA" or "this payment API needs step-up" need a common vocabulary of authentication strength that relying parties can request (`acr_values`) and verify (`acr`/`amr`). NIST SP 800-63-4 defines AAL (authentication), IAL (identity proofing) and FAL (federation).

## Decision

1. **Method → AAL and `amr` mapping** (RFC 8176 values; Auvora-registered values where none exist):

| Method | `amr` | Max AAL alone | Phishing-resistant |
|---|---|---|---|
| Password | `pwd` | AAL1 | no |
| Email magic link / email OTP | `otp` + `email` (Auvora) | AAL1 | no |
| SMS / WhatsApp / voice OTP | `sms` / `otp`+`whatsapp` / `tel` | AAL1 (restricted authenticator) | no |
| TOTP/HOTP | `otp` | combines → AAL2 | no |
| Push with number matching | `swk` + `user` | combines → AAL2 | no |
| Synced passkey (UV) | `swk` + `user` + `mfa` (+ Auvora `fido`) | AAL2 | yes |
| Device-bound passkey / security key (UV, attestation trusted) | `hwk` + `user` + `mfa` (+ Auvora `fido`) | AAL3 | yes |
| PIV/CAC smart card, X.509 with key in hardware | `sc` / `hwk` + `pin` | AAL3 | yes |
| Federated login | Upstream `amr`/`acr`, mapped per connection trust config | ≤ upstream, capped by connection's FAL trust | per upstream |

   AAL2 = two distinct factors (one may be the passkey's UV) or a multi-factor authenticator. AAL3 requires a hardware-bound, phishing-resistant, verifier-impersonation-resistant authenticator with UV and the reauthentication limits in 4.

2. **`acr` values** issued by Auvora: `urn:auvora:acr:aal1`, `…:aal2`, `…:aal2:phr` (phishing-resistant), `…:aal3`. Also accepted: NIST-style aliases `http://idmanagement.gov/ns/assurance/aal/{1,2,3}`, and per-project custom acr aliases mapped to one of the above.
3. **IAL:** a user attribute `assurance.ial ∈ {0,1,2,3}` with evidence reference (proofing connector, ADR-0098) and timestamp; exposed via the `verified_claims` (OIDC Identity Assurance) claim when requested and permitted.
4. **Reauthentication limits:** AAL2 session max 30 days absolute / 12 h idle for elevated actions requiring AAL2. AAL3 max 12 h absolute / 15 min idle. Defaults are configurable to be *stricter* only.
5. **FAL:** downstream federation: FAL1 = signed assertions (default), FAL2 = signed + audience/injection protections + encrypted or back-channel delivery (PAR/JAR/JARM or encrypted SAML assertion), FAL3 = FAL2 + holder-of-key (DPoP/mTLS/HoK). Per-client profile (ADR-0084) declares the FAL. Upstream connections declare the trusted FAL and the maximum AAL they may assert.
6. **Policy use:** flow and access policies (ADR-0067, CEL) reference `session.aal`, `session.phishing_resistant`, `user.ial`. Step-up via RFC 9470 `insufficient_user_authentication` challenges and `acr_values`/`max_age`.
7. **Downgrade protection:** once a user enrolls a phishing-resistant method and the policy `require_phr_after_enrollment` (default `true` for admin roles, `false` for CIAM) is set, flows never offer weaker-only paths except governed recovery (ADR-0078).

## Rationale

A single mapping gives consistent tokens, consistent policy conditions and clean NIST/eIDAS reporting, and enables "admin requires AAL3"-style controls.

## Alternatives considered

- **Free-form per-project `acr` only**: rejected as non-interoperable.
- **eIDAS LoA as primary**: rejected because it's region-specific; supported as aliases (`low/substantial/high`) via the custom alias mechanism.

## Consequences

- Positive: precise step-up; auditable assurance.
- Negative: AAL3 conditions (attestation trust) require FIDO MDS3 handling.

## Fallback / revisit trigger

Revisit when NIST publishes a revision of 800-63 or the IETF/OIDF registers new `amr` values that replace Auvora-specific ones.

## Implementation notes

P1 `pwd`/`otp` + aal1; P2 full mapping, step-up, passkeys, `acr_values`; P5 FAL profiles; P8 IAL via proofing. Crates: `auvora-authn`, `auvora-sessions`, `auvora-oauth`.

## Verification

Table-driven tests asserting issued `acr`/`amr` per method combination; OIDF `acr_values` tests; downgrade abuse-case tests.
