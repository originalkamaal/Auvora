# ADR-0005: Modular monolith, single binary with roles

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D05, §5.1, §5.5
- **Related:** ADR-0006, ADR-0009, ADR-0010, ADR-0019, ADR-0023, ADR-0100

## Context

v0.2 drew control plane, data plane, gateway and authz as boxes without deciding whether they are services. Microservices from day one would multiply operational cost, contradict the small-deployment principle, and slow development. The owner asked for a single binary "but without compromising features." The system still has to scale its latency-sensitive paths independently.

## Decision

1. Auvora ships as **one binary, `auvora`**, built from a Cargo workspace of modular crates (plan §5.5). The binary includes the server, CLI and dev tooling as subcommands (`auvora serve`, `auvora dev`, `auvora migrate`, `auvora admin …`).
2. The server runs one or more **roles**:

   | Role | Responsibilities |
   |---|---|
   | `edge` | TLS termination, ACME, host→project routing, rate limiting, bot checks |
   | `data` | Flow API, OAuth/OIDC, SAML front-end, sessions, tokens, introspection, AuthZEN |
   | `control` | Admin API, console, config, keys, policy publishing |
   | `worker` | Outbox, webhooks, comms, SCIM client, directory sync, rotation, retention, ACME renewals |
   | `ldap` | LDAP gateway |
   | `radius` | RADIUS server |
   | `proxy` | Identity-aware proxy / forward-auth |
   | `scim` | SCIM server (can be isolated for noisy provisioning traffic) |
   | `node` | Optional node-local SPIFFE Workload API / certificate sidecar (ADR-0092); **not** included in `all` |
   | `all` | Default: every role above except `node` |

   Selection: `auvora serve --roles=data,edge`, config `[server] roles = ["all"]`, env `AUVORA_SERVER__ROLES`.
3. **Module boundaries are enforced in code.** Each domain crate exposes a public service trait; cross-crate access happens only through those traits. No crate reads another crate's tables directly. Each crate owns its schema area (ADR-0017).
4. **Roles never call each other synchronously over the network** in the default design. Coordination happens through Postgres (config versions, outbox) and the optional bus (ADR-0008). The data plane never makes synchronous calls to the control plane (ADR-0009).
5. SAML XML processing runs in a **sandboxed child worker process** spawned by the binary (ADR-0014). This is a process boundary, not a separate product.
6. Splitting a role into a separate artifact or service needs a superseding ADR, backed by profiling evidence (see trigger).
7. Internal gRPC is permitted only between split roles (ADR-0020). It is never exposed as a public API.

## Rationale

A modular monolith keeps one artifact, one version, one upgrade and one config while still scaling horizontally per role. All features remain available in every topology. Strict crate boundaries keep a later split cheap.

## Alternatives considered

- **Microservices from day one** (Ory-style multiple binaries): high ops cost; a feature matrix that varies by which services are deployed. Rejected.
- **Single process without roles:** can't isolate data-plane latency from control-plane batch work. Rejected.
- **Plugin-loaded dynamic libraries:** unsafe ABI in Rust, harder supply-chain verification. Rejected (compile-time modules per ADR-0038 instead).

## Consequences

- Positive: trivial small deployments; consistent behaviour; atomic upgrades; easy local dev.
- Negative: a larger binary; compile times; all roles share a release cadence.
- Risks: module boundaries eroding. Mitigation: a CI dependency-graph lint (crates may depend only on allowed layers) and `pub(crate)` by default.

## Fallback / revisit trigger

Profiling shows a role causing > 10% p99 latency degradation on co-located `data` workloads that can't be fixed by running it in a separate replica set of the same binary. In that case, a superseding ADR may split it into a dedicated artifact. (Plan D05: "Split a role into its own service only when profiling proves contention.")

## Implementation notes

- P0: `auvora-server` wires roles. Each role has its own HTTP listener config (`[server.listeners.<role>]`) so they can bind separate ports or share one.
- Health endpoints per role: `/healthz` (liveness) and `/readyz?role=data`.
- Helm chart (P1) exposes one Deployment per role group with `roles` values. The default is a single Deployment with `all`.

## Verification

- A CI dependency lint (`cargo xtask check-layers`) enforces the crate layering.
- An E2E suite runs in two topologies, `all` and fully split, and both must pass identically.
