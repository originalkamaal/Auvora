# ADR-0100: Legacy and infrastructure gateways

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §19, §2.1 (authentik, Kanidm, Oathkeeper, Authelia, Teleport rows), §5.1, §27 P7/P10
- **Related:** ADR-0005, ADR-0015, ADR-0056, ADR-0062, ADR-0092, ADR-0099, ADR-0012

## Context

Many enterprises run applications that can't speak OIDC or SAML: LDAP-only apps, VPNs and Wi-Fi (RADIUS), Windows desktops (Kerberos), internal web apps with no auth (which need a proxy), and Linux/SSH fleets. authentik, Kanidm, Authelia, Ory Oathkeeper and Teleport each cover part of this. The owner's direction is to cover all of it without extra products.

## Decision

Each gateway is a **role** of the single `auvora` binary (ADR-0005). It's enabled with `--roles=...` and backed by the same principals, policies and audit. Gateways are stateless apart from caches, and are horizontally scalable.

| Gateway | Role / crate | Decision details | Phase | Edition |
|---|---|---|---|---|
| LDAP gateway | `ldap` / `auvora-ldap-gw` | LDAPv3 server: Bind (simple over TLS only, SASL EXTERNAL), Search, Compare, RootDSE, paged results. **Read-only**: no Add/Modify. Per-application *virtual DIT* (base DN, attribute map, visible groups via CEL). Service accounts bind with app passwords (ADR-0093). MFA at bind time via push (ADR-0056) or an OTP suffix (`password+123456`), per app. LDAPS on 636; StartTLS on 389; plaintext refused | P7 | Open |
| RADIUS server | `radius` / `auvora-radius` | PAP and EAP-TTLS/PAP, EAP-TLS (certificates from the built-in CA, ADR-0092). RadSec (RFC 6614) is preferred. Per-NAS client secrets (≥ 32 bytes). MFA via push or OTP challenge (Access-Challenge). Vendor attribute mapping per NAS profile | P7 | Open |
| Kerberos/SPNEGO | `data` / `auvora-directory` | Acceptor only, using an AD service keytab (stored encrypted, ADR-0072). Used as a login method for desktop SSO. No KDC | P7 | Open |
| Identity-aware proxy | `proxy` / `auvora-proxy` | Reverse proxy (HTTP/1.1, HTTP/2, WebSocket) **and** a forward-auth endpoint (`/v1/forward-auth`) for nginx `auth_request`, Traefik ForwardAuth, Caddy `forward_auth` and Envoy ext_authz (HTTP and gRPC). Per-route policies in Cedar (ADR-0015). Injects a signed JWT (`X-Auvora-Assertion`) and optional identity headers. Session cookie per proxied domain | P7 | Open |
| SSH CA | `data` / `auvora-workload` | Issues short-lived OpenSSH user certs (default 8 h) after OIDC device or browser login via `auvora ssh`, and host certs for registered hosts. Principals mapped by CEL. KRL publication | P10 | Open |
| NSS/PAM module | separate repo `auvora-unix` | Linux login with Auvora identities. Offline credential cache (hashed, TTL 7 days). Group mapping. MFA via PAM conversation | P10 | Open |
| Browser remote access | `ee/` | RDP/SSH/VNC via browser through the proxy role. Out of scope: session recording (plan §2.4) | P10 | EE |

**Common rules:**
1. Every gateway authentication emits the same audit events as the Flow API, and is subject to the risk engine (ADR-0097) and the rate limits (ADR-0074).
2. Gateways read config through the data-plane cache (ADR-0009). They never call the control plane synchronously.
3. Protocol parsers (LDAP BER, RADIUS, EAP, Kerberos ASN.1) are fuzzed and `forbid(unsafe_code)`. TLS is rustls/aws-lc-rs (ADR-0012).
4. Defaults are disabled. Each gateway requires explicit enablement per project: `[gateways.ldap] enabled = false`.

## Rationale

Covering legacy protocols inside the same identity, policy and audit model is the core of the superset USP, and it avoids "outpost" sprawl. Read-only LDAP and acceptor-only Kerberos keep the attack surface bounded while still meeting real needs.

## Alternatives considered

- **Separate outpost binaries (authentik model):** more artifacts to operate. Roles give the same isolation when needed.
- **A writable LDAP server / full KDC:** a large attack surface for little demand. Writes go through the Admin API or SCIM.
- **Leave these to partner products:** contradicts the owner's directive.

## Consequences

- Positive: one system covers VPN, Wi-Fi, legacy apps, desktops and servers.
- Negative: there are many protocols to maintain. Dedicated fuzzers and interop suites per gateway mitigate this.
- Risk: weak legacy credentials (LDAP simple bind). Mitigations: TLS mandatory, app passwords instead of primary passwords, MFA options and rate limits.

## Fallback / revisit trigger

If a gateway has fewer than 5% of Cloud tenants enabling it 12 months after release and carries a high maintenance cost, move it to EE or to community maintenance through a superseding ADR. It isn't removed.

## Implementation notes

- Crates per §5.5. Helm values expose roles as separate Deployments.
- NAS and LDAP app profiles are managed via config-as-code (ADR-0070).

## Verification

- P7 exit criteria: RADIUS with ≥ 3 VPN vendors, LDAP gateway with ≥ 5 legacy apps.
- Envoy, nginx, Traefik and Caddy forward-auth integration tests.
- Fuzzing ≥ 24 h per parser.
- SSH cert validity and KRL tests (P10).
