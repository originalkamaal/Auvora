# ADR-0069: Drop-in components

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §13.2, §2.1 (Clerk row)
- **Related:** ADR-0021, ADR-0022, ADR-0024, ADR-0067, ADR-0068, ADR-0094

## Context

Developers increasingly expect Clerk-style components that render in their own app. Supporting every frontend framework separately is costly.

## Decision

1. **Two packages over one core:**
   - `@auvora/react`: native React components and hooks.
   - `@auvora/elements`: framework-agnostic **Web Components** (custom elements, Shadow DOM, CSS parts) for Vue, Angular, Svelte, Solid and vanilla.
   Both wrap `@auvora/js` (the Flow API client + session helpers). Thin wrappers (`@auvora/vue`, `@auvora/angular`, `@auvora/svelte`) re-export elements with typed bindings.
2. **Component set:**
   - `SignIn`, `SignUp`, `UserButton`, `UserProfile` (account portal panels), `OrgSwitcher`, `OrgProfile`, `OrgMembers`, `CreateOrg`, `Consent`, `MFASetup`, `PasskeyManager`, `SessionList`, `SupportBanner`, `ConnectedAgents`.
   - Guards: `SignedIn`, `SignedOut`, `Protect` (role/permission).
3. **Behaviour:** components render Flow API `nodes` (ADR-0067). The same node renderer is shared with hosted UI via an internal package `@auvora/flow-ui` (Apache-2.0).
4. **Theming:** the same design token schema as hosted UI (ADR-0068), so a theme is portable between the hosted and embedded modes. Appearance can also be overridden with CSS variables and `::part()`.
5. **Security:**
   - Components run in the customer origin. Session handling uses a BFF cookie when the framework adapter supports server routes (`@auvora/nextjs`, `@auvora/node`). Otherwise it uses in-memory tokens with DPoP keys in non-extractable WebCrypto storage.
   - Refresh tokens are never stored in `localStorage`.
   - Cross-origin calls require the app origin in the project's `allowed_origins` (CORS).
6. **Size budget:** `@auvora/elements` ≤ 60 KB gzip for SignIn + core, enforced in CI.
7. **Accessibility:** WCAG 2.2 AA (ADR-0104).
8. **Phase:** P3 (React + Elements); P5 (framework wrappers alongside tier-2 SDKs).

## Rationale

Web Components cover all frameworks from one codebase. React gets a first-class idiomatic package because it is the largest audience.

## Alternatives considered

- **Per-framework native implementations**: high maintenance.
- **iframe widgets**: poor UX, framing risks, blocked by CSP strategies.

## Consequences

- Positive: broad framework coverage, consistent UX.
- Negative: Shadow DOM styling limits. Mitigated by tokens and parts.
- Risks: SSR hydration issues with custom elements. Mitigation: Next.js uses `@auvora/react`.

## Fallback / revisit trigger

If a major framework's custom-element interop blocks adoption (≥ 10 issues/quarter), add a native wrapper for that framework.

## Implementation notes

Packages live in `packages/`. Storybook + Playwright component tests. Versions are released in lockstep with the server's minor version.

## Verification

- A bundle-size CI check.
- axe tests.
- E2E against the Flow API for every component.
- A lint forbids token persistence in Web Storage.
