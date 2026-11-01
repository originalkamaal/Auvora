# ADR-0060: Interoperability test targets

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D60, §24, §27 (exit criteria P2, P4, P5, P7)
- **Related:** ADR-0051, ADR-0106, ADR-0014, ADR-0099, ADR-0091, ADR-0100, ADR-0064

## Context

Principle #11 requires testing against independent implementations. Phase exit criteria name specific interop targets. The matrix, automation level and accounts must be fixed so that "interop" is measurable.

## Decision

1. **Interop matrix** (`conformance/interop/matrix.toml`, generated into docs):

| Area | Targets | Automation | Phase |
|---|---|---|---|
| Upstream enterprise IdPs (Auvora as SAML SP / OIDC RP) | Okta, Microsoft Entra ID, Google Workspace, ADFS (Windows Server 2022+), Ping (PingOne, PingFederate), OneLogin, JumpCloud, Keycloak, authentik, Zitadel, Shibboleth IdP | Keycloak, authentik, Zitadel, Shibboleth in containers: per-PR/nightly. SaaS IdPs: nightly against dedicated dev tenants. ADFS: weekly Windows VM | P4 |
| SAML IdP (Auvora as IdP) consumers | Salesforce, AWS IAM Identity Center, Google Workspace, Slack, Zoom, Atlassian Cloud, GitHub Enterprise, Microsoft 365 (WS-Fed), ServiceNow, Workday, Zendesk, Box, Dropbox, Tableau, Snowflake, Datadog, PagerDuty, Okta (as SP), Entra (as SP), Shibboleth SP | Containerized SPs nightly; SaaS SPs weekly scripted (Playwright) + release-gating manual checklist | P7 |
| SCIM clients (Auvora as SCIM server) | Okta, Entra ID, OneLogin, JumpCloud, Rippling | Nightly (Okta/Entra) via provisioning test apps; others weekly | P4 |
| OIDC RPs (Auvora as OP) | oidc-client-ts, NextAuth/Auth.js, Spring Security, ASP.NET Core, go-oidc, Authlib, AppAuth iOS/Android, Kubernetes API server, Grafana, Argo CD, HashiCorp Vault, Tailscale (custom OIDC) | Containerized per-PR subset, nightly full | P1–P3 |
| WebAuthn authenticators | Chrome, Safari, Firefox, Edge; iOS/macOS iCloud Keychain, Android/Google Password Manager, Windows Hello, 1Password, Bitwarden, Dashlane; YubiKey 5, Google Titan, SoloKey, Feitian | Virtual authenticator (CDP) per-PR; real devices on a release-gating manual checklist + a device lab in P5 | P2 |
| MCP clients | Claude (Desktop/Code), ChatGPT connectors, VS Code / GitHub Copilot, Cursor, MCP Inspector, official MCP SDKs (TS, Python) | MCP Inspector + SDKs automated nightly; hosted clients release-gating manual | P5 |
| Workload federation | EKS/GKE/AKS/kind projected tokens, AWS STS, GCP, Azure MI, GitHub Actions, GitLab CI, SPIRE | kind + SPIRE per-PR; cloud nightly via test accounts | P5 |
| SSF/CAEP | OpenID SSF interop event participants, Okta/Entra receivers where available | Interop events + nightly against reference implementation | P5 |
| LDAP directories (sync/bind) | Active Directory (2019/2022), OpenLDAP, 389 DS, FreeIPA, Samba AD, JumpCloud LDAP | Containers nightly; AD weekly VM | P7 |
| LDAP gateway consumers / RADIUS clients | Jenkins, GitLab, Nextcloud, Jira DC, sssd; FreeRADIUS client tools, pfSense/OPNsense, OpenVPN, WireGuard (via proxy), Cisco/Fortinet/Palo Alto (lab images where licensable) | Containers nightly; vendor appliances release-gating | P7 |
| Resource-server middleware | All tier 1/2 SDK middlewares against Auvora + one foreign AS (Keycloak) to prove standards-only behavior | Per-PR | P3+ |

2. **Accounts and environments:** developer/test tenants are created for each SaaS target, owned by the project organization, with credentials stored in GitHub Actions environments (OIDC-federated where possible) and rotated quarterly. Where free tiers don't exist, budget for the minimum paid tier is approved before the phase that needs it.
3. **Gating:** each phase exit lists the required green rows. A row is "green" when all automated tests pass for 7 consecutive nightly runs and the manual checklist is signed off for the release.
4. **Results:** published per release (ADR-0051, ADR-0106).
5. **Quirk handling:** vendor quirks are encoded as named compatibility profiles (e.g. `scim.profile = "entra"`), never as silent global relaxations.

## Rationale

This fixes concrete, measurable interop evidence per phase and limits manual work to what can't be automated.

## Alternatives considered

- **Ad-hoc testing at release**: rejected because it's non-repeatable.
- **Only containerized OSS targets**: rejected because enterprise buyers need evidence against Okta/Entra/Google.

## Consequences

- Positive: credible "works with" claims; earlier detection of quirks.
- Negative: cost of SaaS tenants and device lab; test flakiness from external services (mitigated by the 7-day green rule and retries with recorded evidence).

## Fallback / revisit trigger

If a SaaS target withdraws test access, mark the row "manual-only" and document it. Removing a target requires a new ADR.

## Implementation notes

`conformance/interop/` harness with per-target Playwright and API scripts; device lab from P5. Matrix rows are added in the phase listed.

## Verification

CI dashboard of matrix status; the phase-exit checklist references matrix rows; release notes include the matrix snapshot.
