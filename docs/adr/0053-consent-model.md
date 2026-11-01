# ADR-0053: Consent model

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D53, §9.2, §11.1–11.2, §13.1, §13.4
- **Related:** ADR-0052, ADR-0067, ADR-0079, ADR-0086, ADR-0091, ADR-0084, ADR-0104

## Context

OAuth consent protects users from over-broad third-party access. Showing consent for first-party apps hurts UX. Agents and RAR need granular, revocable, time-bound approvals. This ADR is about *OAuth authorization consent* (grants). Privacy/marketing consent records are ADR-0079.

## Decision

1. **Client trust classes:**
   - `first_party`: set only by project admins; never via DCR/CIMD. Consent is skipped unless the client requests scopes marked `always_consent` (e.g. `offline_access` for agents) or the RAR type has `requires_consent: true`.
   - `third_party`: consent is always required for any scope not previously granted. This is the default for every new client.
2. **Scope catalog per resource server:** each scope has a `name`, a localized `display_name` and `description`, a `sensitivity` (`low|medium|high`) and flags `always_consent` and `admin_consent_only`. Scopes not in the catalog are rejected (`invalid_scope`), never silently granted. OIDC standard scopes (`openid`, `profile`, `email`, `phone`, `address`, `offline_access`) are built in.
3. **Grant record** (`grants` table): principal, client, resource(s), scopes, RAR `authorization_details` (normalized), `org_id` (if org-scoped), created/updated, `expires_at` (default none for normal grants; **agents default to 90 days**, configurable per delegation), and the source (`user|org_admin|admin|policy`). Incremental consent merges into the existing grant and shows only the delta.
4. **Org-admin consent:** org admins may pre-consent a third-party app for all org members (scopes not marked `user_consent_only`). Users then see no prompt, but grants show "approved by your organization."
5. **Consent UI:**
   - shows the app name, verified-publisher badge or an "unverified" warning (ADR-0052), requested scopes grouped by sensitivity, RAR details rendered from per-type templates, and the redirect host;
   - a high-sensitivity scope requires the session to satisfy the policy's AAL (step-up, ADR-0050);
   - partial consent (unchecking optional scopes) is supported when the client declares `optional_scopes`.
6. **Revocation:** users revoke grants from the account portal; admins from the console/API. Revocation invalidates refresh tokens and token-vault entries derived from the grant within the D33 bound (ADR-0033) and emits a CAEP `session-revoked`/token-claims-change event (ADR-0096).
7. **Prompt semantics:** `prompt=consent` forces the screen. `prompt=none` with a missing grant returns `consent_required`.

## Rationale

This matches the user expectations set by Google/Microsoft, provides the granular and expiring grants that agent delegation (ADR-0091) needs, and the catalog requirement prevents silent scope creep.

## Alternatives considered

- **Consent for all clients**: rejected because it hurts first-party UX without security benefit.
- **Implicit grant of unknown scopes**: rejected as unsafe.

## Consequences

- Positive: an auditable, revocable consent ledger; enterprise pre-consent.
- Negative: resource servers must register scopes (mitigated by CLI/Terraform import).

## Fallback / revisit trigger

None.

## Implementation notes

P2 consent screen + grants; P4 org-admin consent; P5 RAR templates, agent expiries. Crates: `auvora-oauth`, `auvora-flows` (`consent` flow).

## Verification

OIDF consent-related tests; unit tests for incremental consent; an abuse test asserting that a DCR client can never become first-party; revocation propagation test.
