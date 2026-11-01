# ADR-0102: Mobile and native apps

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §9.1 (First-Party Apps, Native SSO), §13.1 (native mode, attestation), §9.3 (`native` profile), §27 P3/P5
- **Related:** ADR-0024, ADR-0066, ADR-0067, ADR-0084, ADR-0085, ADR-0056, ADR-0032, ADR-0029

## Context

Mobile, desktop and CLI apps want native-feeling login without the insecure ROPC grant, which is excluded by ADR-0066. They also need app-family SSO, device binding and proof that requests come from a genuine app. Platform rules (App Store, Play) and RFC 8252 constrain redirect handling.

## Decision

1. **Default native flow:** Authorization Code + PKCE through the system browser (ASWebAuthenticationSession / Custom Tabs). Redirects use **claimed HTTPS links** (Universal Links/App Links) by default. Private-use schemes (reverse-DNS only) and loopback (any port, RFC 8252 §7.3) are allowed for desktop/CLI. This is enforced by the `native` client profile (ADR-0084).
2. **Native first-party login (no browser):** supported via the Flow API native mode (ADR-0067), aligned with the **OAuth 2.0 for First-Party Applications** draft (`authorization_challenge_endpoint`). It is allowed only for clients marked `first_party = true` with app attestation enabled. It is never available to third-party clients.
3. **App attestation:**
   - Apple App Attest / DeviceCheck and Google Play Integrity.
   - Verified server-side in `auvora-authn` (attestation verification module).
   - Per client: `[clients.<id>.attestation] mode = "required" | "risk_signal" | "off"`. The default is `risk_signal`; `required` is the default for first-party native login.
4. **Native SSO for Mobile Apps:** a device secret is shared across an app family (same team/keychain group). It's issued only when the client opts in, and is bound to the device key.
5. **Token binding:** native SDKs generate a non-exportable device key (Secure Enclave / StrongBox when available) and use **DPoP** for all token requests. Refresh tokens are sender-constrained (ADR-0030).
6. **Passkeys in native apps:** platform credential APIs (ASAuthorization, Credential Manager), with associated-domains / Digital Asset Links verification documented and validated by the CLI (`auvora doctor mobile`).
7. **SDK distribution:** Swift (SPM), Kotlin (Maven Central), React Native (npm), Flutter (pub.dev). They are hand-written and thin (ADR-0024), with secure storage defaults (Keychain with `ThisDeviceOnly`, Android Keystore + EncryptedDataStore).
8. **Desktop and CLI:** loopback redirect or device grant (RFC 8628). The `auvora` CLI is the reference implementation.
9. **Push MFA and the Authenticator app** are covered in ADR-0056.

## Rationale

This meets the ROPC use case (native UX) securely, with attestation and DPoP. The system-browser default maximizes SSO and follows RFC 8252.

## Alternatives considered

- **ROPC for first-party apps:** rejected by ADR-0066.
- **Embedded WebViews:** phishing-prone and blocked by Google for OAuth.
- **A Rust core with UniFFI bindings:** rejected in ADR-0024 in favor of thin native SDKs.

## Consequences

- Positive: a native experience with stronger security than ROPC; app-family SSO.
- Negative: the First-Party Apps draft may change. It's kept behind a versioned profile flag.
- Risk: attestation availability varies (rooted devices, regions without Play Services). That's why `risk_signal` mode is the default.

## Fallback / revisit trigger

If the First-Party Apps draft is abandoned or changes incompatibly, keep the Flow API native mode as Auvora's own documented protocol and track its successor.

## Implementation notes

- P3: Flow API native mode, `native` profile, loopback/claimed-link validation.
- P5: SDK tier 2 (Swift, Kotlin, RN, Flutter), attestation, Native SSO, DPoP in SDKs.

## Verification

- Device-farm E2E (iOS/Android latest 2 major OS versions).
- Attestation negative tests (replayed or invalid attestation objects).
- A redirect-URI rule test matrix.
- Confirmation that ROPC is rejected at the token endpoint (`unsupported_grant_type`).
