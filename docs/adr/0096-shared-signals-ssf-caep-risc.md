# ADR-0096: Shared Signals (SSF/CAEP/RISC)

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §21, §22, §24 (Signals), §27 P5
- **Related:** ADR-0033, ADR-0032, ADR-0080, ADR-0097, ADR-0103, ADR-0091, ADR-0082

## Context

Short-lived tokens and session revocation inside Auvora (ADR-0033) don't help applications that hold their own sessions, or upstream and downstream systems (EDR, MDM, other IdPs) that learn about risk first. The OpenID Shared Signals Framework (SSF), with its CAEP and RISC event profiles carried as Security Event Tokens (RFC 8417), is the standard way to exchange these signals continuously. Enterprise buyers (IPSA profile) and continuous-access-evaluation scenarios require it.

## Decision

1. Auvora implements an **SSF Transmitter** and an **SSF Receiver** in `auvora-events`.
2. **Transmitter:**
   - Publishes `/.well-known/ssf-configuration` per project.
   - Stream management API: create, read, update, delete streams; status; subject add/remove; verification events.
   - **Delivery:** push (RFC 8935) and poll (RFC 8936). Push is the default.
   - SETs are signed with a dedicated `ssf-signing` key usage (ADR-0082). `typ=secevent+jwt`.
   - **Events emitted:**
     - CAEP: `session-revoked`, `credential-change`, `assurance-level-change`, `token-claims-change`, `device-compliance-change`.
     - RISC: `account-disabled`, `account-enabled`, `account-purged`, `credential-compromise`, `identifier-changed`, `identifier-recycled`, `sessions-revoked`.
     - Auvora extensions under `https://schemas.auvora.dev/secevent/` for agent kill switch (ADR-0091) and delegation revoked.
   - **Subjects:** `iss_sub`, `email`, `opaque`, `complex` (user + session + device). Pairwise subjects are honored per receiver client.
   - **Receivers are OAuth clients** with scope `ssf:stream`. A stream is scoped to one project and, optionally, one organization.
3. **Receiver:**
   - Subscribes to upstream transmitters (enterprise IdPs, EDR/MDM vendors, other Auvora projects).
   - Maps inbound events through CEL rules (ADR-0016) to actions: revoke sessions, step-up on next request, disable user, raise risk score (ADR-0097), notify admins.
   - Verifies the issuer, the JWKS (fetched via the egress guard, ADR-0103), `aud` and `jti` replay (fail closed, ADR-0073).
4. **Delivery semantics:** at-least-once with exponential backoff (initial 1 s, max 10 min, retained 72 h) and per-stream ordering by subject. A stream is paused after 24 h of continuous failure, and admins are notified.
5. **Latency SLO:** the event is enqueued in the outbox in the same transaction as the triggering change. Transmit p99 is ≤ 5 s after commit in-region, consistent with ADR-0033.
6. **Config:** `[ssf] enabled = true`, `[ssf.transmitter] max_streams_per_project = 100`, `[ssf.receiver] enabled = true`.

## Rationale

Using the standard instead of proprietary webhooks for security signals follows the "standards first" principle. It is required for IPSA and makes Auvora interoperable with Okta, Entra, Google, Apple, EDR and MDM vendors. The outbox (ADR-0008) already guarantees transactional emission.

## Alternatives considered

- **Webhooks only (ADR-0080):** proprietary formats would push every receiver to write custom integration code. Webhooks still exist for general events.
- **Receiver only:** Auvora couldn't drive continuous access in downstream apps.
- **Waiting for the IPSA phase (P8):** too late, because the agent kill switch and session revocation are GA features.

## Consequences

- Positive: continuous access evaluation across apps; an enterprise differentiator; agent kill-switch propagation.
- Negative: more key usages and stream state to operate.
- Risk: a malicious or compromised transmitter could trigger mass revocations. Mitigations: per-receiver action allowlist, a rate cap on destructive actions (default 1,000 disables/h per stream, then pause + alert), and admin approval for `account-disabled` mapping in workforce projects.

## Fallback / revisit trigger

If the SSF/CAEP/RISC final specifications change event types incompatibly, support both versions behind stream-level `events_supported` negotiation for ≥ 12 months.

## Implementation notes

- Phase P5 (transmitter + receiver). Phase P8: IPSA interop and SCIM events alignment (ADR-0099).
- Crates: `auvora-events`, `auvora-sessions`, `auvora-risk`.
- SDK middleware ships an SSF receiver helper for resource servers.

## Verification

- Interop with ≥ 2 independent SSF implementations before GA.
- Replay and signature negative tests.
- Chaos test: the receiver endpoint is down for 24 h, the stream pauses, and it resumes without loss within the 72 h retention.
- Latency SLO in the benchmark suite (ADR-0105).
