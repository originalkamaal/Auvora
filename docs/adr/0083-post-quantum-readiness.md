# ADR-0083: Post-quantum readiness

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §23 (Post-quantum), §27 P10, §2.2
- **Related:** ADR-0012, ADR-0031, ADR-0082, ADR-0092

## Context

Harvest-now-decrypt-later threatens TLS confidentiality today. Signature forgery by a quantum computer threatens long-lived trust anchors (CA roots, SAML certificates, audit checkpoint signatures) later. NIST published ML-KEM (FIPS 203), ML-DSA (FIPS 204) and SLH-DSA (FIPS 205). JOSE/COSE and X.509 registrations for PQ signatures are still settling, and relying parties will lag.

## Decision

1. **TLS key exchange now:** hybrid `X25519MLKEM768` is enabled by default in rustls with aws-lc-rs (ADR-0012), for inbound and outbound connections, with classical fallback for peers that don't support it.
2. **Signatures: crypto agility, not early switching.**
   - Every signing path goes through the `alg`-pinned key model (ADR-0082), so adding an algorithm is a configuration change, not a code-path change.
   - ML-DSA-44/65/87 and SLH-DSA are implemented behind the feature flag `pq-signatures` as soon as aws-lc-rs exposes them. They're enabled per project and key class once the IANA JOSE/COSE and LAMPS X.509 registrations are final.
   - **Dual publishing:** a project may publish classical and PQ keys at the same time and choose per client (via client metadata `id_token_signed_response_alg` / resource-server config). Classical stays the default until relying-party support is broad.
3. **Long-lived anchors first:**
   - Built-in CA roots (ADR-0092) and audit checkpoint signing keys gain hybrid or PQ options first: a composite or dual signature on checkpoints.
   - Generating a new CA root requires choosing an explicit algorithm, and the console shows a PQ recommendation.
4. **Symmetric:** AES-256-GCM and HMAC-SHA-256/384. 256-bit symmetric keys are the default everywhere (Grover margin).
5. **Inventory:** `auvora keys inventory --crypto` reports algorithms in use per project and class (a cryptographic bill of materials) to support migration planning.
6. **Playbook:** `docs/security/pq-migration.md` (written in P5) describes the stages: hybrid TLS → PQ for internal anchors → dual-publish for OIDC/SAML → PQ default.

## Rationale

Hybrid key exchange is mature and solves the urgent threat. Moving signatures early would break relying parties, so agility plus inventory is the safe path. It keeps every capability and adds PQ without removing classical options.

## Alternatives considered

- **PQ-only signatures now.** Rejected: no RP support.
- **Waiting entirely.** Rejected: TLS harvest risk, and retrofitting agility later is costly.

## Consequences

- Positive: an early, low-risk PQ story; a marketable differentiator.
- Negative: larger PQ signatures and keys (ML-DSA signatures ~2.4–4.6 KB) increase token size. Mitigated by per-client selection.
- Risks: algorithm or standard churn. Mitigated by the feature flag and versioned profiles.

## Fallback / revisit trigger

- If aws-lc-rs lacks a needed PQ primitive 6 months after IANA registration, evaluate a vetted alternative crate for that primitive only, via a new ADR.
- A CNSA 2.0 or regulatory mandate date triggers a change to the default.

## Implementation notes

- Crates: `auvora-crypto`, `auvora-jose`, CA module in `auvora-workload`.
- Hybrid TLS ships in P1. The inventory command ships in P5. PQ signatures ship in P10 (or earlier once standards are final).

## Verification

- A TLS handshake test asserts the hybrid group is negotiated with supporting peers.
- Agility test: adding a mock algorithm needs no change outside `auvora-crypto`.
- The inventory output is golden-tested.
