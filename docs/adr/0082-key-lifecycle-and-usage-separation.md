# ADR-0082: Key lifecycle and usage separation

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §17 (v0.2 retained), §23, §6.1 (envelope encryption)
- **Related:** ADR-0011, ADR-0012, ADR-0031, ADR-0072, ADR-0083, ADR-0092, ADR-0096

## Context

Auvora signs and encrypts with many keys: tokens, SAML, cookies, webhooks, DEK wrapping and CA material. Reusing one key across purposes, having no lifecycle states, or relying on manual rotation are recurring causes of IAM incidents. ADR-0011 fixes *where* keys live. This ADR fixes *how* they live.

## Decision

1. **Usage classes.** A key belongs to exactly one class, enforced by type in `auvora-crypto`:
   `token_signing`, `id_token_encryption`, `userinfo_encryption`, `saml_signing`, `saml_encryption`, `cookie_hmac`, `csrf_hmac`, `webhook_signing`, `ssf_signing`, `dek_wrapping` (KEK), `data_encryption` (DEK), `ca_root`, `ca_intermediate`, `audit_checkpoint_signing`, `config_snapshot_signing`.
2. **Scope:**
   - Per project: token, SAML, webhook, SSF, DEK and cookie keys.
   - Per cell or platform: config-snapshot and audit-checkpoint keys, and the platform KEK.
   - Per tenant: optional dedicated KEK (BYOK, EE).
3. **State machine:** `pending → active → retiring → retired → destroyed`.
   - `pending`: published in JWKS for pre-distribution (default 24 h before activation), not used to sign.
   - `active`: exactly one per class/scope/algorithm signs.
   - `retiring`: verify only, still in JWKS, until max token lifetime + 24 h (minimum 7 days).
   - `retired`: not in JWKS, kept for audit verification.
   - `destroyed`: material erased (crypto-shred). Metadata is kept.
4. **Rotation:**
   - Signing keys rotate automatically every 90 days (ADR-0031).
   - Cookie/CSRF HMAC keys every 30 days.
   - DEKs every 365 days or 2^32 encryptions, with lazy re-wrap.
   - KEK rotation per provider policy.
   - **Emergency rotation:** one action, or `auvora keys rotate --emergency`. The old key skips `retiring`, so tokens signed by it fail. An SSF/CAEP event is sent, all sessions are optionally revoked, and an audit plus a mandatory incident note are recorded.
5. **Algorithm pinning:** each key stores its `alg`. Verification uses the key's `alg`, never the token header. `none` and HS* for asymmetric contexts are rejected (ADR-0031).
6. **Material handling:**
   - Private keys never leave the KeyProvider when it is KMS or HSM.
   - With the local keystore, keys are encrypted at rest under the KEK, decrypted in memory only, and zeroized on drop (`zeroize`).
   - Export only as KEK-wrapped backup. Restore drills run every release.
7. **Configuration:** `[keys]` section with `rotation.signing_days`, `rotation.hmac_days`, `prepublish_hours`, `retire_grace_days`, `provider`. Per-project overrides are allowed for rotation periods within [7, 365] days.
8. **Observability:** key age metrics; an alert at 80% of the rotation period if automatic rotation failed; JWKS change events.

## Rationale

Strict separation limits the blast radius. Pre-publication and a retiring grace window eliminate the "kid not found" outages that are common with manual rotation.

## Alternatives considered

- **One signing key per installation.** Rejected: no tenant isolation, and bad for compromise impact.
- **Manual rotation only.** Rejected: operationally unsafe.

## Consequences

- Positive: safe automatic rotation, per-project isolation, crypto-shredding.
- Negative: more keys to manage, with KMS cost per key. Mitigated by local DEK wrapping under one KMS KEK per project.
- Risks: clock or job failure stalls rotation. Mitigated by alerting.

## Fallback / revisit trigger

KMS per-key costs exceeding budget in Cloud triggers a move to the per-cell KEK with per-project DEKs. The class model doesn't change.

## Implementation notes

- Crates: `auvora-crypto` (types, state machine), `auvora-store` (`keys`, `keys_history`), worker rotation job.
- Ships in P1 (local plus AWS KMS). Other providers per ADR-0011.

## Verification

- Type-level tests prevent cross-class use.
- Rotation chaos test (tokens across rotation boundaries).
- Emergency-rotation drill in CI.
- A JWKS conformance test in the OIDF suite.
