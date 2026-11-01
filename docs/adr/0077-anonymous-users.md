# ADR-0077: Anonymous users

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §2.1 (Firebase/Supabase row), §7.2 (Principal kind `anonymous`), §7.3, §8.1
- **Related:** ADR-0018, ADR-0034, ADR-0054, ADR-0074, ADR-0037, ADR-0026; amended by ADR-0114

## Context

Firebase and Supabase support anonymous/guest users who can later upgrade to permanent accounts (shopping carts, trials, games). Doing this carelessly enables data-merge attacks and abuse such as unlimited account creation.

## Decision

1. **Principal kind** `anonymous`, external ID prefix `anon_`. No identifiers, no credentials except a session-bound device key.
2. **Enabled per project and app** (`anonymous.enabled`, default false).
3. **Creation:** via the Flow API `registration` with `method=anonymous`. Subject to rate limits (ADR-0074) and bot protection (ADR-0037). Default creation limit 5 per IP per hour.
4. **Tokens:**
   - Anonymous principals receive normal sessions and tokens with `amr: ["anon"]`, `acr: "anon"` (below AAL1).
   - The claim `auvora_anon: true`.
   - Resource servers and Cedar policies can deny by this attribute.
   - Never eligible for org membership (except orgs with `allow_anonymous=true`), admin roles, delegation to agents, or support-access targets.
5. **Upgrade (same principal):** attaching a credential or identifier through `settings`/`registration` keeps the same `principal_id`, transitions the kind to `user`, and emits `auvora.user.upgraded_from_anonymous.v1`.
6. **Merge into an existing account:**
   - If the identifier already belongs to a user, the flow requires authentication *as that user* first.
   - Auvora then emits `auvora.user.anonymous_merge_requested.v1` with both IDs. Data merging is the **application's** responsibility via a sync hook (`pre-anonymous-merge`) or webhook. Auvora never auto-merges app data.
   - After merge, the anonymous principal is deleted (hard delete, no 30-day window). Its sessions are revoked.
7. **Lifetime:** inactive anonymous principals are purged after `anonymous.ttl_days` (default 30, max 365). Purge emits events in batches.
8. **Metering:** anonymous principals don't count as MAU until upgraded (Cloud pricing, ADR-0063). They are counted separately.
9. Linking social or enterprise identities to an anonymous principal follows ADR-0034 (proof of control).

## Rationale

This matches Firebase/Supabase ergonomics, with explicit merge semantics and hard limits that prevent account-pollution abuse.

## Alternatives considered

- **Client-side guest IDs only**: no server tokens, so apps can't protect APIs.
- **Auto-merge of app data**: Auvora doesn't own app data; unsafe.

## Consequences

- Positive: closes a feature gap against Firebase and Supabase.
- Negative: an extra principal kind across policy code.
- Risks: abuse for token farming. Mitigation: limits, bot protection, low default TTL, policy denial.

## Fallback / revisit trigger

None.

## Implementation notes

`auvora-core` principal kind; `auvora-flows` anonymous method; purge worker in `worker` role. P2.

## Verification

- Tests: upgrade keeps the ID; merge requires prior auth; anonymous can't obtain admin or delegation; TTL purge.
- An abuse test for creation limits.
