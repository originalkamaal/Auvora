# ADR-0067: Flow API as the single engine

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §13.1, §13.2, §1.1 U4
- **Related:** ADR-0016, ADR-0021, ADR-0038, ADR-0068, ADR-0069, ADR-0102

## Context

Developers integrate via hosted UI, drop-in components, headless SDKs or a self-hosted UI. Competitors often have one well-tested path and weaker secondary paths. Auvora promises every surface with equal security (USP U4).

## Decision

1. All interactive authentication logic lives in **`auvora-flows`**, a server-side state machine. Every UI surface is a client of the public **Flow API**. No surface has private endpoints.
2. **Flow types:** `login`, `registration`, `recovery`, `verification`, `settings`, `consent`, `device`, `ciba-approval`, `org-select`, `step-up`, `link-account`, `logout`.
3. **API:**
   - `POST /v1/flows/{type}` creates a flow (with `return_to`, `client_id`, `login_challenge`, `org`, `ui_locales`).
   - `GET /v1/flows/{type}/{flow_id}` returns `{id, type, state, expires_at, nodes[], messages[], csrf_token, ui: {action, method}}`.
   - `POST /v1/flows/{type}/{flow_id}` with `{node_group, values}` returns the next state, a `redirect`, or `completed` + session.
4. **Nodes** are UI-agnostic typed descriptors: `input` (with type, autocomplete, validation), `passkey` (WebAuthn options JSON), `otp`, `qr`, `choice`, `redirect` (social/enterprise), `captcha`, `consent_scopes`, `info`, `script-free hidden`. Messages carry stable IDs (`auvora.msg.<n>`) for i18n.
5. **Flow policy** (CEL, ADR-0016) per project → app → org decides the steps, factors, conditions, progressive profiling and terms acceptance. Policies are versioned (ADR-0070, ADR-0071).
6. **Modes:**
   - Browser: cookie + CSRF token, `SameSite=Lax`.
   - Native: flow bound to a client-generated key (DPoP-style proof per request) + app attestation where configured (ADR-0102).
   - Neither mode exposes session tokens to page JS in browser mode.
7. **Lifetimes:**
   - Flow TTL is 15 minutes by default (`flows.ttl`, 5–60 minutes).
   - Max transitions per flow: 50.
   - Flows are single-device; cross-device continuation only via explicit magic-link/QR nodes.
8. **State storage:** flow state in Postgres (`flows` table, UNLOGGED option) or Valkey (ADR-0073), with HMAC-integrity. A flow ID is a 128-bit random value plus a prefix (`flw_`).
9. Hooks (ADR-0038) and the risk engine (ADR-0097) run inside transitions. They can add requirements or deny, never skip steps.
10. The **visual Flow Designer** (P7) edits the same policy model. It doesn't create a second engine.

## Rationale

One engine means one place to test, fuzz, audit and certify, so security parity across surfaces is structural.

## Alternatives considered

- **Server-rendered hosted pages with separate headless APIs** (classic Keycloak): two code paths.
- **Client-side orchestration**: leaks security decisions to the client.

## Consequences

- Positive: parity; hosted UI as dogfood of the headless API.
- Negative: API design must be expressive enough for every UI.
- Risks: node schema churn. Mitigation: versioned node schema, additive-only after 0.5.

## Fallback / revisit trigger

None. Node schema changes need an ADR after 1.0.

## Implementation notes

- P1: login, registration, recovery, verification, settings, logout.
- P2: consent, device, step-up, link-account.
- P4: org-select.
- P5: ciba-approval.
The JSON Schema for flows is published in OpenAPI.

## Verification

- Property tests over the state machine (no transition reaches `completed` without required factors).
- Hosted UI E2E runs exclusively against the public Flow API.
- A route lint ensures `apps/hosted-ui` uses no non-public endpoint.
