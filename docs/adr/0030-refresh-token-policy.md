# ADR-0030: Refresh token policy

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D30, §9.1, §9.4
- **Related:** ADR-0029, ADR-0032, ADR-0033, ADR-0084, ADR-0094, ADR-0091

## Context

Refresh tokens are long-lived bearer credentials and a prime theft target, especially in SPAs and native apps. RFC 9700 requires public-client refresh tokens to be sender-constrained or rotated.

## Decision

1. **Format:** opaque, 256-bit random, prefixed `auv_rt_`, stored as a SHA-256 hash with metadata: `family_id`, `session_id`, `client_id`, scopes, `authorization_details`, `cnf`, `issued_at`, `last_used_at`, `generation`.
2. **Public clients** (SPA, native):
   - **Rotation always on.** Each use issues a new refresh token and invalidates the old one.
   - **Reuse detection:** presenting an already-rotated token revokes the **entire family** and the linked SSO session (configurable: `refresh.reuse_revokes_session = true`). A `refresh_token.reuse_detected` security event (and SSF credential-change) is emitted.
   - **Grace window:** `refresh.reuse_grace_seconds = 0` by default. It can be set up to 30 s to absorb network retries; during the grace window the *same* successor token is returned and nothing new is minted.
   - DPoP binding is recommended and becomes required under the `spa` profile when `dpop_required = true` (ADR-0084).
3. **Confidential clients:**
   - Sender-constrained via DPoP or mTLS **or** client authentication on every refresh.
   - Rotation is optional (`refresh.rotate = false` default for confidential, `true` for public).
4. **Lifetimes (defaults, per client override within bounds):**
   - Idle: **30 days** (`refresh.idle_ttl`), bounds 1 h – 365 d.
   - Absolute: **90 days** (`refresh.absolute_ttl`), bounds 1 h – 730 d.
   - Refresh never extends beyond the SSO session's absolute lifetime when bound to a session (`refresh.bound_to_session = true` default for browser-originated grants). `offline_access` grants survive session logout but not user disable/delete, password reset (configurable) or credential compromise events.
5. **Issuance conditions:** only for authorization_code, device_code, CIBA and token-exchange (when policy allows). **Never** for client_credentials, Supervised Support Access sessions (ADR-0094) or anonymous users (unless explicitly enabled).
6. **Revocation triggers:**
   - user logout (session-bound),
   - RFC 7009 revocation,
   - user disabled/deleted,
   - password change (`refresh.revoke_on_password_change = true`),
   - MFA reset,
   - admin action,
   - client deleted or secret rotated with revoke flag,
   - consent revoked,
   - SSF/CAEP inbound signals.

   Propagation follows ADR-0033.
7. **Scope narrowing** on refresh is allowed. Broadening is never allowed. Audience switching via `resource` follows ADR-0029.

## Rationale

Rotation with family revocation detects theft in public clients. Sender-constraining protects confidential and high-assurance clients. Tying refresh to session lifetime prevents zombie access after logout.

## Alternatives considered

- **Non-rotating long-lived tokens**: theft goes undetected. Rejected.
- **JWT refresh tokens**: no benefit, and harder to revoke. Rejected.
- **Non-zero grace window by default**: weakens replay detection. Rejected; opt-in only.

## Consequences

- Positive: theft detection; bounded lifetime.
- Negative: rotation needs a write per refresh. Mitigated by indexed hash lookup and batched last-used updates.
- Risks and mitigations: false-positive reuse from flaky networks is mitigated by the opt-in grace window and SDK single-flight refresh locking.

## Fallback / revisit trigger

None.

## Implementation notes

P1 rotation and family revocation; P5 DPoP/mTLS binding. Crate `auvora-oauth`. Table `refresh_tokens` partitioned by issue month (ADR-0109).

## Verification

Abuse tests: replay after rotation revokes the family; concurrency test with 2 parallel refreshes; lifetime boundary tests; OIDF refresh tests.
