# ADR-0003: Business model: open-core + Auvora Cloud

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §3.2, §4 D03
- **Related:** ADR-0004, ADR-0062, ADR-0063, ADR-0057

## Context

The business model determines repository structure, licensing, contribution terms, where code lives, and whether Auvora must handle billing, metering and tenant signup. It was deferred in v0.2 (ADR 15 there). The owner accepted the recommendation: open-core plus a managed cloud.

## Decision

1. Auvora is **open-core**:
   - an open edition (AGPL, ADR-0004) with every protocol and authentication method (ADR-0062);
   - an **Enterprise Edition (EE)** under a commercial license for large-operator capabilities.
2. **Auvora Cloud** is an operated multi-tenant SaaS, built from the same binary as self-hosting plus EE modules. It is not a fork. Private beta begins after GA 1.0 (ADR-0063).
3. **Revenue streams:**
   - Cloud subscriptions, metered on monthly active users (MAU), M2M token volume, and agent and workload principals;
   - EE self-hosted licenses (per-node or per-MAU tier);
   - support and SLA contracts;
   - dedicated or air-gapped deployments.
4. **Metering is a product subsystem.** It counts MAU, M2M tokens and principals per project from audit/outbox events. It is available in open-source builds as reporting only. Billing integration lives in `ee/`.
5. Cloud-only operational tooling (global control plane, billing, fleet management) lives in a separate private repository. It consumes only public APIs and `ee/` crates.
6. The open edition must never be crippled to drive upsell. A feature belongs in EE only if it passes the boundary test in ADR-0062.

## Rationale

Open source drives adoption and trust for security software. A managed cloud captures developers who don't want to operate infrastructure. EE monetizes large-operator needs without an "SSO tax."

## Alternatives considered

- **Pure community OSS:** no sustainable funding for multi-year security work. Rejected.
- **Cloud-only, source-available:** loses self-hosted and sovereign markets. Rejected.
- **Open-core with SSO/SAML paywalled:** widely criticized, and contradicts the USP. Rejected.

## Consequences

- Positive: a funding path; dogfooding through Cloud; a single codebase.
- Negative: the EE/open boundary needs governance; Cloud operations add compliance load (ADR-0059).
- Risks: community perception of open-core. Mitigation: a published edition-boundary policy (ADR-0062) and a commitment that features never move from open to EE.

## Fallback / revisit trigger

None for the model itself. Pricing and packaging are business decisions outside ADR scope.

## Implementation notes

- `ee/` directory in the monorepo (ADR-0023). Cargo features `ee` gate EE crates. Release builds produce `auvora` (open) and `auvora-ee` images.
- Metering module: `auvora-events` consumers write `usage_daily` rollups (ADR-0043). Config: `[metering] enabled = true` (default true; local only).
- License key verification for EE is offline (signed license file, Ed25519) with no phone-home (ADR-0041). Config: `AUVORA_LICENSE_FILE`.

## Verification

- A CI check confirms that open crates don't depend on `ee/` crates (`cargo deny` bans plus a workspace graph check).
- The open build passes the full conformance suite on its own.
