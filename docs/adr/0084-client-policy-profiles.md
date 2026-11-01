# ADR-0084: Client policy profiles

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §9.3, §9.4, §11.2, §2.1 (Keycloak client policies, FAPI)
- **Related:** ADR-0013, ADR-0029, ADR-0030, ADR-0051, ADR-0052, ADR-0085, ADR-0086, ADR-0091

## Context

OAuth security depends on a coherent combination of settings: PKCE, sender-constraining, request objects, client authentication and algorithms. Exposing dozens of independent toggles produces insecure combinations. Keycloak's client policies and FAPI profiles show the value of named, enforced bundles.

## Decision

1. A **profile** is a named, versioned, enforced rule set evaluated at registration time (configuration validation) **and** at runtime (request validation). A client or resource server has exactly one base profile plus optional add-on constraints. Violations are rejected with `invalid_request` / `invalid_client_metadata` and a precise `error_description`.
2. **Built-in profiles** (immutable, versioned `name@vN`):

| Profile | Enforces |
|---|---|
| `baseline` | Code + PKCE S256, exact redirect match (ADR-0085), `iss` response param, `state`, no tokens in URLs, refresh rotation for public clients |
| `spa` | baseline + public client + DPoP recommended (warn) + refresh rotation with reuse detection + max refresh lifetime 24 h idle |
| `native` | baseline + RFC 8252 redirects (claimed HTTPS, loopback any-port, private-use scheme with reverse-DNS), refresh rotation, optional app attestation |
| `confidential` | baseline + `private_key_jwt` or `tls_client_auth`/`self_signed_tls_client_auth` preferred; `client_secret_*` allowed with warning and max secret age 365 d |
| `m2m` | client_credentials only, strong client auth, audience required (RFC 8707), max AT TTL 1 h |
| `fapi2-security` | FAPI 2.0 Security Profile: PAR required, PKCE, DPoP or mTLS sender-constraining, `private_key_jwt`/mTLS auth, PS256/ES256/EdDSA only, `iss` param, short auth-code lifetime (≤ 60 s), no refresh without sender constraint |
| `fapi2-message-signing` | fapi2-security + JAR (signed request objects), JARM, signed introspection responses (RFC 9701) |
| `mcp` | RFC 9728 resource metadata, RFC 8707 audience required, PKCE, CIMD or DCR client identity, consent per resource/tool scope |
| `agent` | token exchange allowed, `act` claim mandatory on delegated tokens, `may_act` checks, CIBA required for scopes flagged `sensitive`, max delegation depth (ADR-0086) |
| `device` | device grant only, user code entropy ≥ 20 bits with rate limit, polling interval enforcement |
| `legacy-compat` | baseline with relaxed algorithm set (RS256) for old RPs; never relaxes PKCE, exact redirects or the Implicit/ROPC exclusion (ADR-0066) |

3. **Custom profiles:** a project can define profiles that **extend** a built-in profile and only add constraints (monotonic tightening; the validator rejects any relaxation). The CEL condition language is available for extra request checks (ADR-0016).
4. **Assignment:**
   - The default profile per client type is set at project level (`baseline` for web, `spa`, `native`, `m2m`).
   - Org-level and resource-server-level minimum profiles may force a stricter profile on any client accessing that org or API. The effective policy is the union of constraints.
5. **Profile upgrades:** a new built-in version (`@v2`) is opt-in per project, with a dry-run report listing clients that would fail. Old versions are supported for at least 12 months.
6. **Configuration:** `[projects.<id>.oauth.profiles]`; client metadata `auvora_profile`; DCR may request a profile but policy decides (ADR-0052).

## Rationale

Named profiles make the secure combination the default and the only path, give FAPI and MCP compliance in one setting, and match Keycloak's client-policy capability with a simpler model.

## Alternatives considered

- **Free toggles only.** Rejected: insecure combinations.
- **Hard-coded per client type.** Rejected: can't express FAPI/MCP/agent or org-level minimums.

## Consequences

- Positive: certification-friendly; fewer misconfigurations.
- Negative: legacy RPs may need `legacy-compat`, which still has bounded relaxation.

## Fallback / revisit trigger

A new OIDF/IETF profile (e.g. OAuth 2.1 final, FAPI 2.x revision) leads to a new built-in profile version, not an edit.

## Implementation notes

- Crate: `auvora-oauth` (`profiles` module); validator shared by the Admin API and DCR.
- `baseline`, `spa`, `native`, `confidential`, `m2m` and `device` ship in P1–P2. `fapi2-*`, `mcp` and `agent` ship in P5.

## Verification

- Each profile has a negative test matrix.
- The OIDF FAPI 2.0 suite runs against `fapi2-*` (certification post-GA, ADR-0051).
- A monotonicity property test for custom profiles.
