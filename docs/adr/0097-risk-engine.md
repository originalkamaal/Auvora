# ADR-0097: Risk engine

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §22, §8.2, §2.1 (Okta ITP, Entra ID Protection, Stytch rows), §27 P5/P8
- **Related:** ADR-0016, ADR-0037, ADR-0050, ADR-0067, ADR-0074, ADR-0096, ADR-0038, ADR-0079

## Context

Adaptive authentication, bot defense, credential-stuffing defense, ITDR and conditional access all need one place that collects signals and turns them into explainable decisions. Competitors split this across products (Okta ITP, Entra ID Protection, Auth0 Attack Protection, Stytch device fingerprinting). It must run in the Postgres-only profile, add little latency to login, and respect privacy.

## Decision

1. **Crate `auvora-risk`**, evaluated in-process by the Flow API (ADR-0067) at defined checkpoints: `identify`, `pre-credential`, `post-credential`, `pre-token`, `session-refresh`, plus asynchronous evaluation of SSF inputs (ADR-0096).
2. **Signals (v1):**
   - IP reputation: built-in Tor exit list, hosting/VPN ASN list, refreshed daily via the worker; pluggable feeds.
   - Geo: from a MaxMind-format database, optional, user-supplied or the bundled GeoLite-compatible file.
   - Velocity per identifier, IP, /24, ASN and device.
   - Impossible travel: > 900 km/h between consecutive successful logins.
   - New device or browser.
   - Device intelligence: a client-side signal bundle from the SDK, hashed server-side with a per-project salt; raw fingerprints are never stored.
   - Credential-stuffing patterns: failure ratio and distinct-identifier fan-out per IP.
   - Breached-credential hit, bot score (ADR-0037), authenticator strength (ADR-0050), upstream SSF signals, and org/user risk context.
3. **Scoring:** a deterministic **rule engine** (CEL rules with weights) produces `score` 0–100 and `reasons[]`. Built-in rule packs are `baseline` (default on), `strict` and `workforce`. Projects can add custom CEL rules. An optional **ML scorer** is an external sync hook (ADR-0038) whose output is a signal, never the sole decision. This arrives in P8.
4. **Actions** are mapped by policy: `allow`, `challenge` (CAPTCHA/PoW), `require_mfa`, `require_phishing_resistant`, `restrict_session` (reduced scopes/lifetime), `admin_approval`, `deny_and_alert`, `lock_and_notify`.
5. **Defaults (`baseline`):**
   - score ≥ 40 → `require_mfa` if enrolled, otherwise challenge;
   - ≥ 70 → `require_phishing_resistant`, or deny if none is enrolled;
   - ≥ 90 → `deny_and_alert`.
6. **Explainability:** every evaluation writes a `risk.evaluated` event with score, reasons and rule versions (no raw signals that are PII), visible in the console and user security log.
7. **Performance budget:** p99 ≤ 3 ms added per checkpoint. External lookups are never on the synchronous path except opt-in sync hooks with a 300 ms timeout, which fail to the configured mode.
8. **Privacy:** geolocation and device intelligence are per-project toggles (`[risk.signals] geo = true`, `device = true`), disclosed in the privacy notice templates (ADR-0079). Signal data is retained for 30 days, then aggregated.
9. **State:** velocity counters live in Valkey or Postgres (ADR-0073). Counters fail open for scoring, but credential endpoints keep the ADR-0074 limits, which fail closed.
10. **ITDR/posture reports** (dormant accounts, MFA coverage, stale credentials, risky sign-ins) are built on these events. Basic reports are open in P5; advanced reports are EE in P8 (ADR-0101).

## Rationale

Rules are explainable, testable and deterministic, which auditors and admins need. ML is pluggable without making the core opaque. Being embedded keeps the Postgres-only profile complete.

## Alternatives considered

- **A built-in ML model in v1:** needs training data that Auvora doesn't have yet, and its decisions are hard to explain.
- **External risk vendor only:** introduces a hard dependency, and self-hosters would lose the feature.
- **Storing raw device fingerprints:** a privacy liability; hashed signals are sufficient.

## Consequences

- Positive: a superset of attack protection and conditional-access inputs; explainable decisions.
- Negative: rule tuning needs care. Mitigation: shadow mode (`mode = "observe"`) per rule pack.
- Risk: false positives lock out users. Mitigations: never permanently lock (ADR-0074), and a recovery path via ADR-0078.

## Fallback / revisit trigger

If the measured false-positive rate on the `baseline` pack exceeds 1% of legitimate logins in Cloud, retune the pack and ship it in observe mode for one release.

## Implementation notes

- P2: velocity and bot hooks.
- P5: full engine, device intelligence, rule packs.
- P8: ML hook, advanced ITDR.
- Config namespace `[risk]`.

## Verification

- A replayed attack corpus (credential stuffing, impossible travel, Tor) produces the expected actions.
- Latency budget tested in bench (ADR-0105).
- Privacy test: no raw fingerprint fields in DB snapshots.
