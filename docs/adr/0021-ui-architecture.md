# ADR-0021: UI architecture

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D21, §13, §5.5
- **Related:** ADR-0022, ADR-0067, ADR-0068, ADR-0069, ADR-0089, ADR-0104

## Context

Auvora must serve every integration style the market offers:
- hosted login pages (Auth0 Universal Login, Keycloak themes),
- drop-in components (Clerk),
- fully headless APIs (Ory Kratos),
- self-hosted forks of the login UI.

Competitors often implement these as separate code paths with different security properties and feature gaps; for example, server-rendered templates support features the headless API doesn't. Auvora also needs an admin console, an end-user account portal and an Org Admin Portal for customers' IT administrators. All of them must ship inside the single binary (ADR-0005) without a Node.js runtime in the minimum profile (ADR-0006).

## Decision

1. **One engine, many surfaces.** Every interactive end-user experience is driven by the **Flow API** state machine (ADR-0067). No surface may implement authentication logic of its own.
2. **Surfaces:**

   | Surface | Location | Delivery | License |
   |---|---|---|---|
   | Hosted UI (login, registration, recovery, verification, consent, device, CIBA approval, org-select, step-up, account portal) | `apps/hosted-ui` | Static React SPA, embedded in the binary via `rust-embed`, served from the project's issuer origin | Apache-2.0 (forkable) |
   | Admin console | `apps/console` | Static React SPA embedded in the binary, served by the `control` role at `/console` | AGPL-3.0 |
   | Org Admin Portal | `apps/org-portal` | Static SPA embedded, served by `control` at `/org-portal` with portal-link tokens | AGPL-3.0 |
   | Drop-in components | `packages/react`, `packages/elements` | npm packages rendering in the customer's origin | Apache-2.0 |
   | Headless SDKs | `packages/js` etc. | npm/other registries | Apache-2.0 |

3. **No server-side rendering runtime.** The binary serves static assets plus JSON APIs only. Dynamic per-project branding is delivered as data: theme JSON and localized strings from `GET /v1/flows/ui-config`. It is not delivered as server-generated HTML.
4. **Account portal** is part of `apps/hosted-ui` (route `/account`), so it reuses the theme and Flow API `settings` flows.
5. **Console parity.** The console calls only the public Admin API (`/v1/...`). No private console endpoints exist (plan principle 15).
6. **Custom hosting:** `auvora ui export --project <id>` produces a static bundle of the hosted UI pre-configured for a project, for customers who self-host the UI (ADR-0069).
7. **Visual Flow Designer** (P7) is a console feature that edits flow policies (ADR-0067). It is not a separate engine.
8. **No-JS fallback:** none for interactive flows. Clients without JavaScript use the Device Authorization grant (RFC 8628) or CIBA. This is documented explicitly.

## Rationale

- A single Flow API means hosted, drop-in, headless and self-hosted surfaces share one tested state machine. A feature added once is available everywhere, which is pillar U4.
- Static SPAs keep the binary self-contained and the minimum profile Postgres-only.
- An Apache-licensed hosted UI lets customers fork it without AGPL concerns.

## Alternatives considered

- **Server-rendered templates in Rust (Askama/Tera)**: a second rendering path whose features drift from headless. Rejected.
- **Next.js SSR hosted UI**: requires a Node runtime, contradicting ADR-0006. Rejected.
- **Separate login service**: contradicts ADR-0005. Rejected.

## Consequences

- Positive: feature parity across surfaces; one security review target; small operational footprint.
- Negative: the hosted UI requires JavaScript; the first paint depends on SPA load.
- Risks and mitigations: SPA load time is mitigated by a ≤ 150 KB gzipped initial bundle budget enforced in CI, immutable-cache hashed assets, and the theme fetched in parallel.

## Fallback / revisit trigger

If measured p75 Largest Contentful Paint of the hosted login on a mid-range mobile on 4G exceeds 2.5 s after optimizations, evaluate pre-rendered static shells per locale (still no runtime SSR).

## Implementation notes

- P1: minimal hosted UI (login, registration, recovery, verification).
- P3: full hosted UI, console v1, components.
- P4: Org Admin Portal.
- P7: Flow Designer.
- Crates: `auvora-api` (static serving), `auvora-server`. Config: `[ui] console_enabled = true`, `[ui] hosted_ui_path_prefix = ""`, `AUVORA_UI__CONSOLE_ENABLED`.

## Verification

- CI bundle-size check.
- An OpenAPI usage lint ensuring `apps/console` calls only documented endpoints.
- Playwright E2E running the same flow scenarios against the hosted UI, React components and a headless test client.
