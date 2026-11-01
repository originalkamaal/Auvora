# ADR-0076: User profile schema and metadata

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §7.2 (User), §13.3 (registration forms), review Q45
- **Related:** ADR-0016, ADR-0029, ADR-0067, ADR-0070, ADR-0072, ADR-0075, ADR-0099

## Context

Applications need custom user data. Competitors use either a free-form JSON blob (Auth0 `app_metadata`/`user_metadata`, Clerk public/private/unsafe metadata) or a typed schema (Ory identity schemas, Keycloak user profile). Auvora wants both, safely.

## Decision

1. **Typed profile schema** per project (JSON Schema 2020-12 subset), versioned (ADR-0070, ADR-0071).
   - Standard attributes follow OIDC standard claims (`given_name`, `family_name`, `name`, `picture`, `locale`, `zoneinfo`, `birthdate`, `address`…).
   - Custom attributes have type, format, validation (CEL, ADR-0016), `pii` tag (ADR-0072), and permissions: `user_editable`, `admin_only`, `readonly`.
   - Custom attributes also carry a mapping to claims, SAML attributes and SCIM.
   - Schema changes are additive-or-migrated: removing or retyping a field requires a migration job with dry-run.
2. **Metadata blobs** (untyped JSON), mirroring Clerk/Auth0 semantics:
   - `public_metadata`: readable by frontend SDKs, writable by backend only.
   - `private_metadata`: backend only, never sent to the client.
   - `unsafe_metadata`: user-writable from the frontend; never trusted for authz.
3. **Limits:**
   - Each blob ≤ 8 KB.
   - Typed profile ≤ 64 KB.
   - ≤ 200 custom attributes per project.
   - All limits configurable down, not up, past platform maxima.
4. **Org-scoped attributes:** memberships carry their own typed attribute schema (per project) and metadata blobs (ADR-0088).
5. **Claims emission:** nothing goes into tokens by default except standard claims mapped by scope. Custom attributes enter tokens only via explicit claim mappers (CEL) with a total custom-claim budget of 4 KB per token (`tokens.custom_claims_max_bytes`).
6. **Registration forms** are generated from the schema: multi-step, conditional fields (CEL), progressive profiling (ask for field X after N logins), terms/consent checkboxes linked to versions (ADR-0079).
7. **Search/filter:** typed attributes marked `indexed: true` get a GIN/expression index (≤ 20 indexed custom attributes per project). Metadata blobs aren't indexed by default.
8. **Validation:** server-side, always (UI hints are derived, not trusted).

## Rationale

The typed schema gives validation, forms, SCIM/claims mappings and privacy tagging. Blobs give fast iteration. Together they cover both competitor styles.

## Alternatives considered

- **Blobs only**: no validation, no privacy tagging.
- **Schema only**: too rigid for app-specific data.
- **EAV tables**: poor performance and ergonomics.

## Consequences

- Positive: superset of Ory/Keycloak/Auth0/Clerk models.
- Negative: two mechanisms to document.
- Risks: oversized tokens. Mitigation: the claim budget.

## Fallback / revisit trigger

None.

## Implementation notes

- Storage: `users.profile JSONB` (validated) + `users.public_metadata/private_metadata/unsafe_metadata JSONB`.
- The schema is in `project_profile_schemas` (versioned).
- P1 (standard + blobs), P2 (custom schema + forms).

## Verification

- Schema validation fuzzing.
- A test that `private_metadata` never appears in Flow API, frontend SDK or token responses.
- A token size budget test.
