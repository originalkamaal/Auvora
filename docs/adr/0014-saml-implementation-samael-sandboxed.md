# ADR-0014: SAML implementation: samael, sandboxed

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D14, §12, §27 (P0 spike, P4 SP, P7 IdP), §30
- **Related:** ADR-0005, ADR-0012, ADR-0089, ADR-0100, ADR-0103, ADR-0106, ADR-0060

## Context

SAML 2.0 relies on XML-DSig and XML-Enc, a historically dangerous area (signature wrapping, XXE, canonicalization bugs, comment injection). v0.2 forbade a bespoke XML security stack and suggested OpenSAML, but OpenSAML is Java/C++ and would add a JVM. Auvora needs SAML SP (P4), SAML IdP (P7) and WS-Federation passive (P7), without compromising the single-binary goal.

## Decision

1. SAML is implemented in the crate **`auvora-saml`**, using **`samael`** for SAML protocol structures and **libxmlsec1** (via samael's `xmlsec` feature) for XML-DSig/Enc.
2. **Process isolation:**
   - All XML parsing, canonicalization, signature verification, decryption and signing run in a **sandboxed child worker process** (`auvora saml-worker`, same binary), spawned and supervised by the `data` role.
   - Communication uses a Unix domain socket with length-prefixed, serde-encoded typed messages.
   - Sandbox: seccomp-bpf allowlist (Linux), no network, read-only filesystem, a memory limit (`[saml.worker] memory_limit = "256MiB"`), a CPU time limit per request (2 s), and a pool of N workers (default `num_cpus/2`, min 2), recycled every 10,000 requests.
   - On macOS/dev the sandbox is best-effort (no seccomp). Production builds on Linux require it.
3. **Strict processing profile (normative):**
   - XML parser: no DTDs, no external entities, no XInclude; max document 256 KiB; max depth 64; max attributes per element 64.
   - Only signed elements are used: after verification, the worker returns **only the verified subtree** (by reference ID resolved by the verifier, never by re-querying the original document). This defeats signature wrapping.
   - Exclusive C14N only. Allowed signature algorithms: RSA-SHA256/384/512, ECDSA-SHA256/384. SHA-1 is rejected by default; per-connection opt-in `allow_sha1 = true` exists for legacy IdPs, flagged in the console, and disabled in FIPS mode.
   - Encryption: AES-GCM preferred; AES-CBC accepted only on decryption for legacy, per-connection opt-in. Key transport: RSA-OAEP.
   - Validation: Destination, Recipient, Audience, InResponseTo, NotBefore/NotOnOrAfter (skew ±60 s), SubjectConfirmation, and Issuer pinned per connection. Assertion ID replay cache uses `put_if_absent` until expiry (ADR-0007).
   - IdP-initiated SSO is off by default and enabled per connection with replay and RelayState protections.
4. **Metadata fetching** (URL refresh) goes through the egress guard (ADR-0103).
5. **Fallback backend:** the `SamlBackend` trait abstracts the worker protocol. An alternative backend based on Shibboleth OpenSAML (JVM sidecar) can be implemented behind the same trait if the trigger fires.

## Rationale

samael + libxmlsec1 reuses a mature C XML-security implementation instead of inventing one, which satisfies the v0.2 principle. Process isolation contains memory-safety risk from the C libraries. Returning only the verified subtree is the strongest structural defense against XSW.

## Alternatives considered

- **OpenSAML JVM sidecar:** most mature, but adds a JVM runtime to every deployment. Kept as the fallback.
- **Pure-Rust XML-DSig implementation:** bespoke XML security, forbidden by the plan's principle. Rejected.
- **Commercial SAML library:** licensing conflicts with AGPL distribution. Rejected.

## Consequences

- Positive: SAML stays in the single binary; strong isolation; a clean fallback path.
- Negative: C dependencies (libxml2, libxmlsec1, OpenSSL for xmlsec) in the build; IPC overhead (~0.2 ms).
- Risks: libxml2 CVEs. Mitigation: sandbox, Renovate tracking, fuzzing of the worker interface, and static linking of pinned versions in release images.

## Fallback / revisit trigger

> 2 unresolved interoperability failures with the top-10 IdPs (ADR-0060), or a critical CVE pattern in libxmlsec1/libxml2 that can't be mitigated by the sandbox. Either one triggers a superseding ADR adopting the OpenSAML JVM sidecar backend behind `SamlBackend`.

## Implementation notes

- P0: spike with interop against Okta, Entra ID, Google Workspace, ADFS and Keycloak (go/no-go report).
- P4: SP. P7: IdP + WS-Fed (WS-Fed reuses the same XML-DSig worker).
- Crate `auvora-saml` (FFI allowed; `unsafe` permitted only here and in other listed FFI crates, ADR-0047).

## Verification

- cargo-fuzz on worker message parsing and on SAML responses (with libFuzzer + structure-aware XML mutators), 72 h clean before P4 exit.
- An XSW test corpus (all known wrapping permutations, comment injection, XXE) must pass in CI.
- Interop matrix (ADR-0060) runs in nightly CI.
- External review of `auvora-saml` before P4 exit.
