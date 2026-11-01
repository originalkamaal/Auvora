# ADR-0022: Frontend stack

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D22, §5.6, §13
- **Related:** ADR-0021, ADR-0023, ADR-0068, ADR-0069, ADR-0104

## Context

Auvora ships three SPAs (hosted UI, admin console, Org Admin Portal) and a component library for customers. These need:
- a single, accessible, themeable stack,
- strong typing against the generated API client,
- a long maintenance horizon.

## Decision

| Concern | Choice |
|---|---|
| Language | TypeScript (strict mode, `noUncheckedIndexedAccess`) |
| UI library | React (current major) |
| Build | Vite |
| Routing | TanStack Router (type-safe routes) |
| Server state | TanStack Query |
| Forms | React Hook Form + Zod schemas generated from the Flow API node descriptors |
| Primitives | Radix UI |
| Styling | Tailwind CSS driven by design tokens (Style Dictionary → CSS variables) |
| Icons | Lucide (tree-shaken) |
| i18n | ICU MessageFormat via FormatJS (ADR-0104) |
| Tables/virtualization | TanStack Table + TanStack Virtual (console) |
| Charts (console analytics) | A single lightweight library chosen in P3 behind an internal `Chart` wrapper |
| Unit tests | Vitest + Testing Library |
| E2E | Playwright (Chromium, WebKit, Firefox) |
| Accessibility checks | axe-core in unit and E2E |
| Lint/format | ESLint (typescript-eslint strict) + Prettier |
| Package manager | pnpm workspaces (ADR-0023) |
| API client | Generated from OpenAPI 3.1 (ADR-0020, ADR-0024) into `packages/api-client` |
| Web Components | `packages/elements` built with Lit, sharing design tokens with React components |

Rules:
1. No CSS-in-JS runtime. Styling is Tailwind + CSS variables only, so it stays compatible with a strict CSP (ADR-0068).
2. No `dangerouslySetInnerHTML` in hosted UI or components. A lint rule enforces this.
3. No third-party scripts or analytics in the hosted UI. Fonts are self-hosted.
4. Dependency additions to `apps/hosted-ui` and the published packages require security review (ADR-0046).
5. Browser support: the last 2 versions of evergreen browsers (ADR-0044).

## Rationale

- React has the largest ecosystem and is what customers already use for drop-in components.
- Radix provides accessible primitives (ADR-0104).
- TanStack gives type safety end-to-end.
- Tailwind + tokens give theming without runtime cost.
- Lit is the smallest mature option for framework-agnostic Web Components.

## Alternatives considered

- **Vue / Svelte / Solid**: smaller ecosystem for drop-in customer components. Rejected as primary; Web Components cover those frameworks.
- **Next.js**: needs a server runtime. Rejected (ADR-0021).
- **MUI / Chakra**: heavy and harder to theme deeply. Rejected.
- **Stencil for Web Components**: heavier toolchain than Lit. Rejected.

## Consequences

- Positive: one stack across all frontends; strict CSP compatibility; generated types.
- Negative: React major upgrades must be tracked; Lit adds a second component runtime.
- Risks and mitigations: npm supply chain is mitigated by a lockfile, provenance checks, `pnpm audit` and Renovate with review (ADR-0046).

## Fallback / revisit trigger

If React stops being maintained or its licensing becomes restrictive, or TanStack Router is abandoned (no release for 12 months), write a superseding ADR.

## Implementation notes

P1 sets up `apps/hosted-ui` with the stack; P3 delivers console and components. Shared packages: `packages/ui` (internal design system), `packages/tokens`.

## Verification

- CI: `tsc --noEmit`, ESLint (including no-dangerous-HTML and no-inline-style rules), Vitest, Playwright, axe (zero violations), bundle-size budget.
