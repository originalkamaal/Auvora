# ADR-0086: Delegation, token exchange and actor chains

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §9.1 (Token Exchange, JWT/SAML bearer), §9.4 (`act` depth), §7.2 (Delegation entity), §11, §16, §10 (Transaction Tokens)
- **Related:** ADR-0013, ADR-0029, ADR-0084, ADR-0091, ADR-0092, ADR-0094, ADR-0039

## Context

Supervised Support Access, AI agents, service-to-service on-behalf-of calls, cross-domain federation and Transaction Tokens all need one consistent delegation model. Inconsistent "impersonation" implementations erase the real actor, and that's the core flaw Auvora avoids (plan §2.3, §16).

## Decision

1. **Single mechanism:** RFC 8693 Token Exchange at the token endpoint (`grant_type=urn:ietf:params:oauth:grant-type:token-exchange`) is the only way to obtain a token where the acting party differs from the subject. There are no proprietary impersonation endpoints.
2. **Delegation vs impersonation:**
   - **Delegation** (default) yields `sub` = subject and `act` = actor (nested for chains). Resource servers always see both.
   - **Impersonation** (no `act`) is **disabled** and cannot be enabled. The need it serves is met by Supervised Support Access (ADR-0094).
3. **Authorization of an exchange.** All of the following must hold:
   - (a) The actor is authenticated (`actor_token`, or client authentication when the client is the actor).
   - (b) A valid **Delegation record** exists (user → agent consent, support session, service trust policy) **or** the `subject_token` carries `may_act` naming the actor.
   - (c) Requested scopes and RAR details are ⊆ subject grant ∩ actor permissions ∩ delegation constraints.
   - (d) The client profile permits exchange (ADR-0084 `agent`, `confidential`, `m2m`).
   - (e) Cedar policy `Action::"token_exchange"` allows it (ADR-0015).
4. **Actor chain rules:**
   - `act` nesting depth defaults to 3 and is configurable up to 5 (`oauth.token_exchange.max_act_depth`).
   - Each hop narrows scope (never widens), and `exp` never exceeds the parent token's `exp`.
   - Cycles (the same actor twice) are rejected.
   - Each hop is audited with the full chain (ADR-0039 `act` chain fields).
5. **Token types accepted:** Auvora access tokens, ID tokens (for subject only, same project), refresh tokens are **not** accepted, external JWTs from trusted issuers (federation trust config), SAML 2.0 assertions (via the SAML worker), and SPIFFE JWT-SVIDs. Issued types: access token (default), ID token (for cross-domain SSO), Transaction Token.
6. **Refresh tokens** are never issued for exchanged tokens by default. They're allowed only for agent delegations whose record grants `offline` and whose profile permits it, with the delegation expiry as an absolute cap.
7. **Assertion grants** (RFC 7523 JWT bearer, RFC 7522 SAML bearer) require an explicit trust configuration: issuer, JWKS/cert, allowed `sub` mapping in CEL, audience = Auvora token endpoint, `jti` replay protection (fail closed, ADR-0073), and max assertion age 5 min.
8. **Transaction Tokens** (draft) are issued via token exchange from an access token at the service-mesh edge. They're short-lived (≤ 60 s), carry `txn`, `purp`, `azd` (requester context) and `sub`, have audience = trust domain, and are signed with a separate key class (an extension of ADR-0082 `token_signing` scoped to the trust domain).
9. **Revocation cascade:** revoking a Delegation record, support session or agent invalidates all derived tokens. Introspection reflects this within ADR-0033 bounds, and an SSF `session-revoked`/`token-claims-change` event is sent.
10. **SDK enforcement:** resource-server middleware exposes `subject`, `actor_chain` and `restrictions`, and it rejects tokens with `act` on endpoints marked `no_delegation`.

## Rationale

One standards-based mechanism for every delegation use case means one place to secure, audit and certify. Keeping `act` everywhere preserves real-actor accountability.

## Alternatives considered

- **Proprietary impersonation API** (Keycloak, Clerk style). Rejected: it substitutes the actor.
- **Separate mechanisms per feature.** Rejected: duplicated security surface.

## Consequences

- Positive: support, agents and microservices share a hardened path; the audit chain is complete.
- Negative: resource servers must understand `act`. Mitigated by SDK middleware.

## Fallback / revisit trigger

Changes in the Transaction Tokens draft are tracked behind a feature flag. A final RFC triggers a profile-version bump.

## Implementation notes

- Crates: `auvora-oauth` (`token_exchange`, `assertion_grants`), `auvora-core` (Delegation), `auvora-agents`, `auvora-support`.
- Basic token exchange ships in P5 (moved before support/agents per the plan §28 roadmap fix). Transaction Tokens ship in P5.

## Verification

- An abuse suite: scope widening, depth overflow, cycle, `exp` extension, refresh smuggling, cross-project subject token, replayed assertion.
- A property test: issued scopes ⊆ intersection.
