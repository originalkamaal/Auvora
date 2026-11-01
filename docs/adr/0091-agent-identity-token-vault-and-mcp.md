# ADR-0091: Agent identity, Token Vault and MCP

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §11 (all), §1.1 U3, §2.1 (Auth0 AI agents, Logto), §2.2 (#2), §9.1 (CIBA), §9.2 (RFC 9728, CIMD), §9.3 (`mcp`, `agent` profiles), §27 P5
- **Related:** ADR-0084, ADR-0086, ADR-0052, ADR-0053, ADR-0087, ADR-0092, ADR-0072, ADR-0096, ADR-0101, ADR-0015

## Context

AI agents act autonomously, often on behalf of humans, calling tools (MCP servers) and third-party APIs. Treating them as OAuth clients or service accounts loses ownership, delegation, consent and accountability. Agent identity is a headline differentiator for Auvora and is delivered at GA (P5).

## Decision

### 1. Principal model

- `kind = agent`, external ID prefix `agt_`.
- Fields:
  - `owner` (user | org | service account): required, transferable, with audit.
  - `publisher` (verified or unverified; same verification ladder as ADR-0085 third-party clients).
  - `version`, `description`.
  - `declared_capabilities[]` (tool and resource identifiers).
  - `risk_tier` (`low` | `medium` | `high`), set by an admin and defaulting to `medium`.
  - `allowed_resources[]` (RFC 8707 audiences).
  - `max_delegation_depth` (≤ ADR-0086 project max).
  - `attestation` (optional: workload federation binding per ADR-0092, or a signed agent manifest JWS from the publisher).
  - `status` (active/suspended/revoked).
- An agent authenticates as an OAuth client via `private_key_jwt`, mTLS, workload federation or CIMD. **Agent ≠ client**: one agent may own several clients (per environment), and each client links to exactly one agent.

### 2. Delegation records

- `delegation(subject → agent)`:
  - Scopes, RAR `authorization_details` (types registered per resource server).
  - Resource constraints, rate limits (`max_calls_per_hour`), spend limits (`max_amount` for RAR payment-like types), and time windows.
  - Expiry (default 30 days, max 365) and `offline` flag.
  - `approval_mode` per scope or action: `none` | `ciba` | `ciba_per_call`.
- Created via the `consent` flow (ADR-0053) with an agent-specific consent screen that shows owner, publisher verification, capabilities and constraints.
- Revocable from the account portal ("Connected agents"), the org admin console and the API.

### 3. Runtime flows

- **On-behalf-of:** the agent exchanges the user's delegation for an access token (ADR-0086). `sub` = user, `act.sub` = agent, scopes narrowed.
- **Human-in-the-loop:**
  - For scopes or actions flagged `sensitive` (resource-server scope metadata), the `agent` profile (ADR-0084) requires CIBA.
  - The user receives a push, email or in-app approval showing the RAR details (binding message ≤ 120 chars plus a structured view).
  - Default approval TTL is 5 min. Approval yields a single-use token (`ciba_per_call`) or a short window (`ciba`, default 15 min).
- **Agent → agent:** nested `act`, with depth capped per ADR-0086.
- **Autonomous agents** (no human subject): the agent's own principal is `sub`, using its own scopes. The same audit and policy apply.

### 4. Token Vault (third-party API access)

- Stores upstream OAuth tokens obtained via social or enterprise connections (ADR-0087 `token_storage = vault`) or via a dedicated "connect account" flow (incremental scopes at Google, Slack, GitHub, Microsoft, Salesforce, Atlassian, Notion, etc., using the provider manifests).
- Upstream refresh tokens are envelope-encrypted (ADR-0072) with a dedicated DEK class and are **never returned**.
- The agent calls `POST /v1/token-vault/exchange` (an RFC 8693 token exchange with `requested_token_type=urn:auvora:upstream-access-token` and `resource=<provider>`). Auvora validates the delegation (which must include `vault:<provider>:<scopes>`), refreshes upstream if needed, and returns a short-lived upstream access token (≤ its upstream TTL, capped at 1 h).
- Every vault exchange is audited with the provider, scopes and agent chain.
- If upstream revocation is detected (`invalid_grant`), the user is notified to reconnect.

### 5. MCP authorization

- **Auvora as the authorization server for MCP servers:**
  - Resource servers flagged `mcp = true` get hosted RFC 9728 Protected Resource Metadata (`/.well-known/oauth-protected-resource/{resource-path}`), or the MCP server serves it with Auvora's SDK.
  - Audience binding via RFC 8707 is required. PKCE is required.
  - Tool-level scopes (`tools:<name>`) and resource scopes are auto-registered from the MCP server's declared tool list via `auvora mcp sync`.
- **MCP clients:**
  - Client ID Metadata Documents are preferred.
  - DCR is allowed when project policy permits (ADR-0052, off by default; for MCP projects the `mcp_open_dcr` preset uses rate limits and an unverified-publisher label).
  - The consent screen is per tool or resource.
- **SDK:** `@auvora/mcp` (TypeScript), `auvora-mcp` (Python, Go, Rust) middleware. It validates tokens (JWKS, DPoP), checks tool scopes per call, emits `WWW-Authenticate` with `resource_metadata` and step-up challenges (RFC 9470), and enforces `act` restrictions.
- **Auvora MCP server** (plan §11.4):
  - Official MCP server exposing the Admin API as tools.
  - Read-only by default. Write tools require the `admin:write` delegation plus CIBA approval per call (with AAL2+), and are audited with the agent chain.
  - Shipped as `auvora mcp serve` (stdio and streamable HTTP).

### 6. Kill switch and propagation

- Revoke an agent (globally), a delegation (per user), a tool scope (per resource), or an agent version.
- Introspection reflects the change in ≤ 5 s (ADR-0033). An SSF `session-revoked` / CAEP `token-claims-change` event is sent to subscribed resource servers (ADR-0096).
- JWT access tokens issued to agents have a default TTL of **5 min**.

### 7. Governance

- Agent inventory: owner, last activity, delegations count, risk tier, unverified publishers.
- Per-agent activity timeline.
- Anomaly signals into the risk engine (ADR-0097): tool-call bursts, new resource, scope expansion requests, off-hours activity.
- Periodic delegation reviews (EE, ADR-0101).
- On owner leaving (JML), agents are transferred or suspended.

### 8. Configuration

`[projects.<id>.agents]` keys:
- `enabled`
- `default_delegation_ttl`, `max_delegation_ttl`
- `require_attestation_for_tier.high = true`
- `token_ttl`
- `ciba_channels`
- `vault.enabled`, `vault.providers`

## Rationale

This is the union of Auth0 for AI Agents (Token Vault, async authorization), Logto and Stytch MCP auth, and Entra agent identities, built on standards (RFC 8693, CIBA, RAR, RFC 9728, RFC 8707, CIMD) with real-actor accountability.

## Alternatives considered

- **Agents as plain OAuth clients.** Rejected: no owner, delegation or human approval.
- **Giving agents user refresh tokens.** Rejected: unbounded, unaccountable access.
- **A proprietary MCP auth scheme.** Rejected: the MCP specification mandates the OAuth-based flow.

## Consequences

- Positive: a leading differentiator; safe agent adoption for customers.
- Negative: the MCP specification is evolving quickly, so profiles need versioning.
- Risks: Token Vault is a high-value target. Mitigations: a dedicated DEK class, HSM/KMS KEK, per-exchange audit, anomaly detection and per-provider kill switch.

## Fallback / revisit trigger

- Any change in the MCP authorization specification leads to a new `mcp@vN` profile (ADR-0084).
- If CIMD isn't adopted by major MCP clients within 12 months, make DCR the primary path, with stronger rate limits.

## Implementation notes

- Crates: `auvora-agents` (principal, delegation, vault, MCP), `auvora-oauth` (CIBA, token exchange, RFC 9728), `auvora-flows` (agent consent, CIBA approval), `packages/mcp`.
- Ships in P5 (GA). MCP interop with at least 3 major MCP clients is a GA exit criterion.

## Verification

- An abuse suite: delegation escalation, vault exfiltration attempts, CIBA approval replay, depth overflow, tool-scope bypass, revoked-agent token use.
- MCP conformance and interop tests.
- External review of Token Vault before GA.
