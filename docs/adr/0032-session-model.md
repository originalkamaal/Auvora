# ADR-0032: Session model

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D32, §21, §5.4
- **Related:** ADR-0007, ADR-0030, ADR-0033, ADR-0050, ADR-0073, ADR-0096, ADR-0102

## Context

Sessions back SSO, logout, step-up, device management, risk and revocation. Stateless JWT sessions can't be revoked promptly. Sessions must work with Postgres only (ADR-0006) yet scale with an optional cache.

## Decision

1. **Server-side sessions.** There are two levels:
   - **SSO session** (`ses_`): per browser or device per project. Holds principal, AAL, `amr`, `auth_time`, per-factor timestamps, device, risk, org context and IP/UA history (policy-controlled).
   - **Client sessions:** one per client/application participating in the SSO session. They are the targets for front-/back-channel logout and refresh-token binding.
2. **Browser cookie:**
   - `__Host-auvora_session` (custom domains: `__Host-` too, since the hosted UI is on the issuer origin).
   - The value is a 256-bit random identifier, stored hashed (SHA-256).
   - `Secure; HttpOnly; SameSite=Lax; Path=/`.
   - A separate `__Host-auvora_csrf` token is used for Flow API browser mode.
3. **Session ID rotation** on every authentication-level change: login, step-up, MFA completion and privilege change. This prevents fixation.
4. **Storage:** Postgres `sessions` is authoritative. Valkey (if configured) is a read-through cache with TTL ≤ 30 s and explicit invalidation on revoke (ADR-0033). Last-seen updates are batched every 60 s.
5. **Default lifetimes** (per project, overridable per org and per app with stricter-only overrides):

   | Profile | Idle | Absolute |
   |---|---|---|
   | CIAM (`session.profile = "ciam"`, default) | 14 days | 30 days |
   | Workforce (`"workforce"`) | 8 hours | 12 hours |
   | Admin console / platform | 30 min | 8 hours |

   Remember-me is opt-in per project. When disabled, the cookie is a session cookie (browser-lifetime).
6. **Per-factor freshness:** policies can require factor recency (`max_age`, step-up RFC 9470) per app or action without terminating the session.
7. **Concurrent session policy:** `session.max_concurrent` (default unlimited), with `on_exceed = "revoke_oldest" | "deny" | "prompt"`.
8. **Device binding:**
   - DBSC (Device Bound Session Credentials) when the browser supports it (`session.dbsc = "auto"`), with fallback to the plain cookie.
   - Native apps: device-bound key + DPoP (ADR-0102).
   - Session hijack signals (IP/ASN/UA change beyond policy) feed the risk engine (ADR-0097).
9. **Logout:**
   - RP-initiated (with `id_token_hint` and validated `post_logout_redirect_uri`), front-channel, back-channel (signed `logout+jwt`), SAML SLO.
   - Global sign-out (all sessions of the principal) and per-device sign-out from the account portal.
   - Admin revoke.
10. **Session tree visibility:** users see their sessions/devices; admins see sessions per principal, and per org for org admins.
11. **Native/mobile sessions** are represented by the same SSO session object (`kind = "native"`), with refresh-token binding instead of cookies.

## Rationale

Server-side sessions enable instant revocation, step-up and visibility. The `__Host-` prefix and SameSite defend against cookie injection and CSRF. Two-level sessions model SSO and per-app logout precisely.

## Alternatives considered

- **Stateless JWT session cookies**: can't be revoked within the SLO. Rejected.
- **SameSite=Strict**: breaks top-level IdP redirects. Rejected; Lax plus CSRF tokens is used.
- **Valkey as authoritative store**: violates the Postgres-only profile. Rejected.

## Consequences

- Positive: instant revocation; clear logout semantics; works Postgres-only.
- Negative: a DB read per session validation without a cache. Mitigated by the in-process LRU (≤ 5 s TTL) + Valkey + indexed hash lookup.
- Risks and mitigations: session table growth is handled by time partitioning and expiry purge (ADR-0109).

## Fallback / revisit trigger

If session validation p99 exceeds 10 ms at target load in the Postgres-only profile, add a mandatory in-process cache with notify-based invalidation (still no new dependency).

## Implementation notes

P1 core sessions + RP-initiated logout; P2 front/back-channel logout; P5 DBSC + device binding. Crate `auvora-sessions`. Project config keys `session.*`.

## Verification

Fixation tests; cookie attribute tests; revocation propagation test (ADR-0033); OIDF logout profiles; DBSC browser E2E where available.
