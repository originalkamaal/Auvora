# ADR-0110: Verifiable credentials and OpenID Federation

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §8.1 (EUDI wallet), §9.1 (pre-authorized code), §12.5, §24 (Credentials), §27 P10
- **Related:** ADR-0013, ADR-0031, ADR-0050, ADR-0083, ADR-0082, ADR-0098, ADR-0103, ADR-0002

## Context

EU regulation (eIDAS 2 and the EUDI Wallet) and similar national programs are making verifiable credentials (VCs) a mainstream identity channel. Large federations (education, research, health, government) need automatic trust establishment beyond bilateral metadata exchange. Few IAM products cover both, so this is a frontier superset item. The data model and crypto choices must not block it later.

## Decision

1. **Roles Auvora will implement** (P10; the data model is prepared from P1):
   - **Verifier (RP)** via **OID4VP 1.0**: accepts presentations of SD-JWT VC and ISO mdoc (18013-5/-7) as a login method, a step-up, or identity proofing (feeds `ial` via ADR-0098). Supports DCQL queries, presentation via same-device and cross-device (QR) flows, and the EUDI ARF high-assurance profile.
   - **Issuer** via **OID4VCI 1.0**: issues SD-JWT VCs (and later mdoc) from Auvora identity data. Supports the authorization-code and **pre-authorized code** flows; credential configurations defined per project; status lists (Token Status List) for revocation.
   - **Holder/wallet:** **not** implemented (out of scope; the user's wallet app is the holder).
2. **Crypto and format:**
   - SD-JWT per the IETF RFC (`auvora-jose` adds selective disclosure).
   - Key binding (KB-JWT).
   - ES256 by default; algorithm policy as in ADR-0031.
   - Issuer keys use a separate key usage `vc-issuance` (ADR-0082).
   - The PQ path follows ADR-0083.
3. **Trust:** verifier trust anchors are configurable per project: EU trusted lists (LOTL), OpenID Federation trust chains, X.509 IACA roots for mdoc, or static issuer allowlists. All fetches use the egress guard (ADR-0103).
4. **OpenID Federation 1.0:**
   - Auvora acts as a **leaf entity** (OP and RP) publishing `/.well-known/openid-federation` entity configurations.
   - It can act as an **intermediate/trust anchor** (EE) for operators running a federation.
   - Trust-chain resolution with metadata policy application, caching (TTL ≤ entity statement `exp`), and automatic client registration (explicit and automatic modes).
5. **Privacy:**
   - Request the minimal claims (DCQL).
   - Store only the claims accepted into the profile with `pii: sensitive` tags; never store raw presentations beyond a 24 h debug window (off by default).
   - Unlinkability: batch-issued credentials with per-presentation keys where the format supports it.
6. **Feature flags:** `[vc] verifier = false`, `issuer = false`, `[openid_federation] enabled = false` until certified profiles exist. Draft-version profiles are versioned (`oid4vp@1.0`).

## Rationale

The verifier is the most valuable part (eIDAS mandates relying-party acceptance). Being an issuer turns Auvora tenants into credential sources. Reusing the JOSE stack and key model keeps the effort incremental. OpenID Federation shares trust-chain machinery with wallet trust.

## Alternatives considered

- **Out of scope:** leaves the EU public sector and regulated CIAM unaddressed. Rejected.
- **W3C VC Data Model with JSON-LD/Data Integrity proofs:** heavier and less aligned with EUDI ARF. It can be added via a format adapter if demand appears.
- **Build a holder wallet:** a different product category.

## Consequences

- Positive: EUDI readiness; a frontier differentiator; IAL2+ without document scanning.
- Negative: fast-moving ecosystem profiles (ARF versions, HAIP). Mitigated with versioned profiles and feature flags.

## Fallback / revisit trigger

If an EUDI ARF or HAIP profile requires mdoc issuance before SD-JWT VC adoption, prioritize mdoc issuance within P10.

## Implementation notes

- P1–P5: principal model supports `verified_claims` and an `ial` source of `vc` (ADR-0098); `auvora-jose` designed for SD-JWT.
- P10: verifier, then issuer, then OpenID Federation.
- Module in `auvora-federation` (federation) and a new `auvora-vc` crate (added to the §5.5 layout at P10).

## Verification

- OIDF OID4VP/OID4VCI conformance suites.
- EUDI reference wallet interop.
- mdoc reader test vectors.
- OpenID Federation interop events.
- Negative tests: tampered disclosures, wrong KB-JWT audience/nonce, revoked status entries.
