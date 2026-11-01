# ADR-0056: Push MFA and Auvora Authenticator

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D56, §8.1–8.2, §27 (P7)
- **Related:** ADR-0050, ADR-0078, ADR-0097, ADR-0102, ADR-0036, ADR-0091, ADR-0062

## Context

Push approval is expected by workforce buyers (Okta Verify, Microsoft Authenticator) and is needed for CIBA approvals (agents, call centers). Customers with their own mobile apps want push inside *their* app. Push is vulnerable to MFA fatigue unless hardened.

## Decision

1. **Two delivery surfaces, one protocol:**
   - **Push SDK** in the native iOS/Android, React Native and Flutter SDKs (ADR-0102), embeddable in customer apps;
   - **Auvora Authenticator**, a first-party iOS/Android app (open source, Apache-2.0, in a separate repo `auvora-authenticator`), with multi-account support (many projects/installations), TOTP storage with encrypted backup, push approval, passkey guidance and CIBA approvals.
2. **Enrollment:** the device generates a P-256 key pair in the Secure Enclave/StrongBox (hardware-backed when available; the device is marked `hardware_backed`). Enrollment via QR or deep link carries a one-time enrollment token (5 min TTL). It includes app attestation (App Attest / Play Integrity) and is registered as a `push_device` credential.
3. **Challenge protocol:**
   - the server sends a data-only push (APNs/FCM) containing just a challenge ID. The app fetches challenge details over TLS with a device-key-signed request (no PII in the push payload).
   - Details shown: app/client name, location (city/country from IP, if enabled), time, and for CIBA/RAR the rendered `binding_message`/authorization details.
   - **Number matching is mandatory:** the login screen shows a 2-digit number and the user types it in the app. **Biometric/device-PIN unlock (UV) is required** to approve. The approval is a signature over (challenge ID, number, nonce, server origin).
   - Challenge TTL 120 s; one pending challenge per user per client.
4. **Fatigue controls:** max 3 push requests per user per 10 min (configurable down only); "Deny and report" → per policy, locks push for the user, raises a risk event (ADR-0097) and notifies admins; denial streaks increase the risk score.
5. **Assurance:** push with number matching + UV + hardware key = multi-factor authenticator → contributes to **AAL2**. It is **not phishing-resistant** and never satisfies `phr` policies (ADR-0050).
6. **Push providers:** APNs (token-based auth) and FCM HTTP v1, with credentials per project for SDK-embedded push (customer's own app IDs) and Auvora-operated credentials for the Authenticator app. Self-hosted installations using Auvora Authenticator route through a **push relay** (`relay.auvora.dev`) that receives only an opaque device token + challenge ID. Air-gapped installations use the SDK path with their own app, or TOTP.
7. **Offline fallback:** the Authenticator shows a TOTP for the same account when push is unreachable.
8. **Recovery:** lost-device flows follow ADR-0078. A new device enrollment invalidates old push devices when `single_push_device = true` (default false).

## Rationale

Supporting both an embedded SDK and a first-party app covers CIAM, B2B and workforce needs (superset goal) with modern anti-fatigue protections.

## Alternatives considered

- **SDK only, no app**: rejected because workforce buyers expect a standalone authenticator.
- **Third-party authenticator integrations only (Duo)**: rejected as a dependency. They're still supported via federation/hooks.
- **SMS-like plain approve/deny without number matching**: rejected (MFA fatigue).

## Consequences

- Positive: complete MFA portfolio; native CIBA UX for agents.
- Negative: two app-store releases to maintain; relay service to operate.
- Risk: relay as a dependency. It's optional, holds no PII, and has a documented alternative.

## Fallback / revisit trigger

If app-store policy or relay economics make the Authenticator unviable, keep the SDK + TOTP path and publish the Authenticator as source-only (new ADR).

## Implementation notes

P7 (SDK push + Authenticator v1 + relay). CIBA approvals via the SDK from P5 (in-app only). Crates: `auvora-authn` (push credential), `auvora-comms` (APNs/FCM). Config: `authn.push.max_requests_per_10m`, `authn.push.challenge_ttl_s`, `push.relay.enabled`.

## Verification

Abuse tests (push bombing, replay of approval signature, approval on a different number); mobile E2E (Detox/XCUITest/Espresso); relay payload inspection test (no PII).
