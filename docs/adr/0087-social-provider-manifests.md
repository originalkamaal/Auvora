# ADR-0087: Social provider manifests

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §8.3, §12.1, §2.1 (Casdoor/Auth0 breadth)
- **Related:** ADR-0016, ADR-0034, ADR-0075, ADR-0103, ADR-0072

## Context

The plan commits to 30+ maintained social providers plus generic OIDC and OAuth 2.0 connectors. Hand-coding each provider doesn't scale and leads to inconsistent security checks. Providers have quirks: Apple's `form_post` and name-only-on-first-login, GitHub's non-OIDC userinfo and email endpoint, Microsoft's multi-tenant `iss`, X's OAuth 2.0 PKCE, and WeChat's non-standard parameters.

## Decision

1. **Declarative manifests.** Each provider is a versioned YAML manifest in `crates/auvora-federation/providers/<id>.yaml`, compiled into the binary and validated against a JSON Schema at build time. Fields:
   - `id`, `display_name`, `protocol` (`oidc` | `oauth2`)
   - `discovery_url` or explicit endpoints
   - `default_scopes`, `pkce` (`required`/`supported`/`unsupported`), `response_mode`
   - `issuer_validation` (`exact` | `template`, e.g. Microsoft `https://login.microsoftonline.com/{tid}/v2.0` with a tenant allowlist)
   - `userinfo` (endpoint plus extra calls, e.g. GitHub `/user/emails`)
   - `claim_mapping` (CEL expressions → canonical claims `sub`, `email`, `email_verified`, `name`, `picture`, `locale`)
   - `email_verified_semantics` (`trusted` | `untrusted` | `conditional` expression)
   - `token_storage` (`none` | `vault`)
   - `quirks` (enumerated, from a fixed list implemented in Rust, e.g. `apple_first_login_name`, `apple_client_secret_jwt`, `github_emails_api`, `wechat_params`)
   - `branding` (logo, colours, button text per locale)
   - `docs_url`
2. **Security checks are not manifest-controlled.** State, nonce, PKCE (when supported), issuer, audience, signature verification and `email_verified` handling are enforced by core code. A manifest can declare capabilities but can never disable a check. `email_verified_semantics = untrusted` prevents auto-linking (ADR-0034).
3. **Custom manifests:** projects can upload their own manifests (same schema, no new quirks) via API/Terraform. The generic OIDC and generic OAuth 2.0 + userinfo connectors are simply "inline manifests."
4. **Provider tokens:** upstream access and refresh tokens are stored only if `token_storage = vault` and the project enables it. They're envelope-encrypted (ADR-0072) and shared with the Token Vault (ADR-0091). Revocation on unlink is attempted when the provider supports it.
5. **Testing:** every built-in manifest ships with recorded fixtures (sanitized discovery documents, token responses, userinfo) and a contract test. A nightly job runs live smoke tests against providers with test accounts where available. Failures open issues automatically.
6. **Outage handling:** a per-provider circuit breaker. When it's open, the hosted UI hides or disables the button with a message and alternate methods remain.
7. **Initial maintained set (P2):** Google, Apple, Microsoft (personal + Entra), GitHub, GitLab, Facebook, LinkedIn, X, Discord, Slack, Amazon, plus generic OIDC/OAuth2. The rest of the plan §8.3 list follows in P3–P5.

## Rationale

Data-driven providers make breadth cheap while keeping security centralized. Community contributions become reviewable YAML plus fixtures rather than code.

## Alternatives considered

- **Code-per-provider.** Rejected: slow, and inconsistent.
- **Fully generic only.** Rejected: poor DX and missing quirk handling.

## Consequences

- Positive: rapid provider expansion with consistent security.
- Negative: novel quirks need a core release.

## Fallback / revisit trigger

If more than 20% of new providers need new quirks, review the manifest expressiveness.

## Implementation notes

- Crate: `auvora-federation`. Config: `[projects.<id>.connections.social.<provider>]` with `client_id`, `client_secret` (or key ref), `scopes`, `allowed_tenants`.

## Verification

- Manifest schema validation in CI.
- Contract tests per manifest.
- A security test that a manifest cannot disable state, nonce or issuer checks.
