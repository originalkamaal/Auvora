# ADR-0068: Hosted UI security model

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §13.3, v0.2 §13.2, §26
- **Related:** ADR-0021, ADR-0067, ADR-0069, ADR-0090, ADR-0104

## Context

The hosted login origin is the most phishing- and XSS-sensitive surface in the product. Customers want deep branding. Competitors allow tenant JavaScript or template logic, which creates XSS and credential-theft risk.

## Decision

1. **No tenant-supplied JavaScript or executable template logic** ever runs in the hosted UI origin.
2. **Customization surface:**
   - Design tokens (colors, typography, radii, spacing, shadows) → CSS variables.
   - Logo and background assets (served by Auvora after re-encoding; SVG is sanitized to a safe subset or rasterized).
   - **Custom CSS**, limited to a sanitized subset: no `url()` outside the asset host, no `@import`, no `expression`. It is applied in a scoped layer.
   - Component **slots** filled with logic-less Markdown/text plus allowlisted elements.
   - Layout presets, per-page templates (logic-less, compiled server-side).
   - Localization overrides (ADR-0104).
3. **Headers on every hosted UI response:**
   - `Content-Security-Policy`: `default-src 'none'; script-src 'self' 'nonce-…' 'strict-dynamic'; style-src 'self' 'nonce-…'; img-src 'self' <asset-host> data:; connect-src 'self' <bot-provider>; frame-src <bot-provider>; frame-ancestors 'none'; form-action 'self' <allowed redirect origins>; base-uri 'none'; require-trusted-types-for 'script'`.
   - `Strict-Transport-Security` (2 years, preload eligible for Auvora domains).
   - `X-Content-Type-Options: nosniff`, `Referrer-Policy: strict-origin-when-cross-origin` (`no-referrer` on flows carrying tokens).
   - `Permissions-Policy` minimal (`publickey-credentials-get=(self)`, `publickey-credentials-create=(self)`).
   - `Cross-Origin-Opener-Policy: same-origin`, `Cross-Origin-Resource-Policy: same-site`.
4. **Framing:** denied by default. A per-app `frame_ancestors` allowlist is permitted only for consent-free embedded flows, and those apps get a mandatory risk warning. Login with passkeys in iframes uses `publickey-credentials-get` delegation only when explicitly allowed.
5. **Trusted Types** are enforced. The React build bans `dangerouslySetInnerHTML` via lint.
6. **Theme versioning:** themes are immutable versions with preview URLs (`?theme_preview=<signed token>`, 1 h TTL), A/B percentage rollout, and one-click rollback.
7. **Third-party scripts:** only bot-protection providers configured in ADR-0037, loaded with SRI where the provider supports it. No analytics scripts. Analytics come from server-side events (ADR-0043).
8. **Customers who need full control** use drop-in components (ADR-0069) or a self-hosted UI in *their* origin.

## Rationale

Strict CSP + Trusted Types + no tenant scripts eliminate whole bug classes. Tokens + CSS + slots satisfy virtually all branding needs.

## Alternatives considered

- **Tenant JS in a sandboxed iframe**: still enables phishing UI and keystroke capture.
- **Full template languages (FreeMarker-style)**: an injection risk.

## Consequences

- Positive: a provably hardened login origin.
- Negative: some exotic customizations require components or a self-hosted UI.
- Risks: CSS-based UI redress. Mitigation: the sanitizer forbids positioning over the credential fields' reserved region and blocks `content:` on inputs.

## Fallback / revisit trigger

None.

## Implementation notes

P1 minimal UI with full headers; P3 theme system. The CSS sanitizer lives in `auvora-flows` (theme module) and is fuzzed.

## Verification

- An automated header test suite runs on every route.
- CSP violation reporting runs in staging.
- The CSS sanitizer is fuzzed with XSS payload corpora.
- An external pentest is scoped to the hosted UI before GA.
