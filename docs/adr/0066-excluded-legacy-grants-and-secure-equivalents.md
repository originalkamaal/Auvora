# ADR-0066: Excluded legacy mechanisms and secure equivalents

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §2.3, §9.1, §1.2 principle 13
- **Related:** ADR-0013, ADR-0058, ADR-0068, ADR-0069, ADR-0081, ADR-0093, ADR-0094, ADR-0102

## Context

Competitors support mechanisms that RFC 9700 (OAuth Security BCP) and OAuth 2.1 deprecate. These include Implicit and Resource Owner Password Credentials (ROPC), silent impersonation, and tenant JavaScript in login pages. "Superset coverage" must cover the *use cases* without copying the vulnerabilities.

## Decision

The following are **never implemented**, not even behind flags:

| Excluded | Secure equivalent (required to exist) | ADR |
|---|---|---|
| Implicit grant (`response_type=token`, `id_token token`) | Authorization Code + PKCE (+ DPoP); BFF pattern in SDKs | 0013, 0029 |
| ROPC (`grant_type=password`) for apps | OAuth for First-Party Applications (native credential UI via Flow API), Native SSO | 0102, 0067 |
| ROPC for migrations | Lazy migration hook + password-hash import | 0058 |
| `plain` PKCE | S256 only | 0013 |
| Silent user impersonation | Supervised Support Access (`act` claim, consent, banner, audit) | 0094 |
| Arbitrary tenant JS in the login origin | Drop-in components in the customer origin; logic-less templates; Wasm actions server-side | 0068, 0069, 0081 |
| Long-lived shared secrets as the only M2M auth | private_key_jwt, mTLS, workload federation; API keys only as a scoped, expiring last resort | 0092, 0093 |
| JWT `alg: none`, header-chosen algorithms | Per-key pinned algorithm | 0031 |
| Wildcard redirect URIs in production | Exact match; wildcard only in dev projects | 0085 |
| Access tokens in query strings | Header/body only | 0029 |
| SMS as a "phishing-resistant" factor | AAL mapping marks SMS as restricted | 0050 |

**Migration aids:**
- The error response for `grant_type=password` or `response_type=token` includes a docs link explaining the equivalent.
- Importers (ADR-0058) flag clients that used these flows and propose replacements.

## Rationale

Following RFC 9700 and OAuth 2.1 is a security baseline. Offering the equivalent keeps the superset promise.

## Alternatives considered

- **ROPC behind an "unsafe legacy" flag**: rejected; flags get turned on and stay on.
- **Implicit for legacy SPAs**: rejected; Code+PKCE works in every modern browser.

## Consequences

- Positive: a smaller attack surface; easier certification.
- Negative: some legacy migrations need app changes.
- Risks: lost deals for ROPC-dependent legacy apps. Mitigation: the First-Party Apps flow and the LDAP gateway (ADR-0100) for apps that only know username and password.

## Fallback / revisit trigger

None (permanent).

## Implementation notes

`auvora-oauth` rejects these flows with `unsupported_grant_type` / `unsupported_response_type` plus an `error_uri`.

## Verification

- Negative conformance tests assert rejection of each excluded mechanism.
- A lint forbids `"password"` grant handling code in `auvora-oauth`.
