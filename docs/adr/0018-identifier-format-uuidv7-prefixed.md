# ADR-0018: ID format: UUIDv7 + prefixed external IDs

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D18, §7.3
- **Related:** ADR-0017, ADR-0019, ADR-0020, ADR-0027, ADR-0075, ADR-0093

## Context

IDs show up in URLs, tokens (`sub`), logs, SDKs, support tickets and cross-cell routing. They need to be index-friendly, globally unique without coordination, non-enumerable, human-recognizable by type, and stable across profile changes. Email or other mutable attributes must never be identity keys.

## Decision

1. **Internal storage:** `uuid` columns holding **UUIDv7** (RFC 9562) values, generated in the application with a CSPRNG for the random bits.
2. **External representation:** `<prefix>_<base32>`, where the body is the 128-bit UUID encoded in Crockford base32, lowercase, 26 chars. Example: `usr_01jbqz6v0f8m3h2k4n5p7r9s1t`.
3. **Registered prefixes (closed list; additions via ADR amendment of this table):**

   | Prefix | Entity | Prefix | Entity |
   |---|---|---|---|
   | `ten` | Tenant | `prj` | Project |
   | `org` | Organization | `mem` | Membership |
   | `usr` | User | `sa` | Service account |
   | `wl` | Workload | `agt` | Agent |
   | `anon` | Anonymous user | `idn` | Identifier |
   | `cred` | Credential | `lid` | Linked identity |
   | `grp` | Group | `rol` | Role |
   | `app` | Application | `cli` | OAuth client |
   | `rs` | Resource server | `con` | Connection |
   | `ses` | Session | `dev` | Device |
   | `gnt` | Grant (consent) | `dlg` | Delegation |
   | `sup` | Support session | `hook` | Hook/action |
   | `pol` | Policy | `key` | Key |
   | `flw` | Flow | `evt` | Event |
   | `inv` | Invitation | `dom` | Domain |
   | `whk` | Webhook endpoint | `job` | Background job |
   | `ak` | API key (ID part) | `env` | Environment link |

4. **Parsing is strict.** The API rejects IDs with the wrong prefix for an endpoint (404, not 400, to avoid an oracle). The prefix is never trusted for authorization.
5. **OIDC `sub`:** for users, `sub` = the external user ID by default (opaque, stable, per project). Optional **pairwise** subjects per sector identifier: `sub = base32(HMAC-SHA256(project_pairwise_key, sector_id || user_uuid))[0..26]`, prefixed `pw_`. `sub` never changes for the life of the principal.
6. **Cell routing hint:** the router uses the project → cell directory (ADR-0019), not ID bits. IDs are therefore portable when a project migrates between cells.
7. **Customer-provided external IDs:** `external_id` (string ≤ 255, unique per project per entity type) is supported for users, orgs and groups to ease migrations and SCIM. It is never used as the primary key.
8. **Secrets aren't IDs.** Tokens, API keys and codes have their own formats (ADR-0029, ADR-0093).

## Rationale

UUIDv7 is time-ordered for B-tree locality and needs no coordination. Prefixes make logs, support and SDK typing unambiguous (the Stripe pattern). Crockford base32 is URL-safe and case-insensitive.

## Alternatives considered

- **UUIDv4:** random index inserts hurt write performance. Rejected.
- **ULID:** equivalent ordering, but not a native Postgres type standard. Rejected in favor of UUIDv7 in `uuid` columns.
- **Snowflake/bigint:** needs coordination and leaks volume. Rejected.
- **Email as key:** mutable, PII. Rejected.

## Consequences

- Positive: efficient indexes; self-describing IDs; strong typing in SDKs.
- Negative: UUIDv7 leaks creation time (millisecond precision). This is accepted, since creation time isn't sensitive for these entities. Pairwise subjects are available for privacy.
- Risks: none significant.

## Fallback / revisit trigger

None.

## Implementation notes

- `auvora-core::id` provides typed IDs (`UserId(Uuid)` etc.) with `Display`/`FromStr`, serde, and sqlx `Type`. Each newtype is generated from the prefix table by a macro.
- P0 deliverable.

## Verification

- Property tests: round-trip encode/decode; parsing rejects wrong prefixes.
- A DB lint ensures primary keys of entity tables are `uuid` with no default except an app-supplied value.
