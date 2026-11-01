# ADR-0034: Account linking and merge

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D34, §7.3, §8.3, §26
- **Related:** ADR-0027, ADR-0035, ADR-0075, ADR-0077, ADR-0087, ADR-0071

## Context

Linking social, enterprise or LDAP identities to existing accounts is a top account-takeover vector:
- pre-account-takeover,
- unverified emails asserted by IdPs (the "nOAuth" class),
- email recycling.

Products that auto-link by email are vulnerable; products that never link frustrate users.

## Decision

1. **Never silent linking by default.** When an upstream identity's identifier matches an existing principal, Auvora runs the `link-account` flow (ADR-0067). The user must **prove control of the existing account** by any factor already enrolled on it (password, passkey, OTP to a verified identifier), satisfying at least the AAL configured by `linking.min_aal` (default AAL1; AAL2 if the existing account has MFA).
2. **Trusted auto-link (opt-in):** an admin may mark a connection `trust_verified_email = true`. Auto-link then happens only if **all** are true:
   - the upstream asserts the email as verified (`email_verified = true` for OIDC, or a mapped SAML attribute),
   - the upstream is authoritative for the domain (e.g. a Google Workspace/Entra tenant matching a **verified org domain**, ADR-0088), or the provider is on the built-in list of providers that verify email ownership (manifest flag, ADR-0087),
   - the existing account's email is verified,
   - the existing account has no phishing-resistant MFA (otherwise proof is still required).

   Entra "unverified email" claims (`email` without `xms_edov`) are never trusted.
3. **Pre-account-takeover defense:**
   - A password registration on an unverified email doesn't block a later legitimate federated signup.
   - The unverified account is quarantined, and the verified identity wins after proof (configurable `linking.unverified_conflict = "prefer_verified"`).
   - Unverified accounts can't receive invitations or org memberships.
4. **Unlinking:** users can unlink an identity if at least one other login method remains. Admins can unlink anything. Unlink revokes upstream tokens stored in the Token Vault (ADR-0091).
5. **Merge** (two existing principals into one):
   - Admin- or user-initiated.
   - The user must prove control of **both**; admins require approval if `merge.requires_approval = true`.
   - The survivor keeps its `principal_id`. The other becomes `merged_into` (a tombstone, so its `sub` resolves to the survivor for relying parties via introspection/userinfo for 90 days).
   - Memberships, credentials and linked identities move over; conflicts are resolved by the explicit choice of the person performing the merge.
   - Merge is **reversible for 30 days** using the entity history (ADR-0071).
   - It emits `principal.merged` (webhook + SSF).
6. **Collisions without email** (phone, username) follow the same proof-of-control rule.
7. **Audit:** every link, unlink and merge is a security-class audit event with actor, both subjects and the proof method used.

## Rationale

Proof of control is the only universally safe linking rule. Opt-in trusted auto-link restores frictionless UX where the upstream is genuinely authoritative.

## Alternatives considered

- **Auto-link on matching email (common default elsewhere)**: account takeover risk. Rejected.
- **Never link**: duplicate accounts and poor UX. Rejected.

## Consequences

- Positive: closes nOAuth and pre-ATO classes; flexible for enterprise domains.
- Negative: an extra step for users on first social login with an existing account.
- Risks and mitigations: user confusion is reduced by clear hosted-UI copy and telemetry on link-flow abandonment.

## Fallback / revisit trigger

None.

## Implementation notes

P2 linking (with social); P4 domain-authoritative auto-link; P3 merge UI. Crates `auvora-federation`, `auvora-flows`.

## Verification

Abuse tests: pre-ATO scenario, unverified-email IdP, Entra unverified email, email recycling, merge reversal; E2E link flows.
