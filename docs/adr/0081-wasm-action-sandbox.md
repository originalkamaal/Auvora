# ADR-0081: Wasm action sandbox

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §14 (tier 4), §5.6 (wasmtime), §27 P8, §30 (Wasm sandbox escape risk)
- **Related:** ADR-0038, ADR-0016, ADR-0068, ADR-0103, ADR-0105

## Context

Auth0 Actions and Zitadel Actions let customers run logic inside auth flows without hosting a service. Auvora covers this need (superset), but running tenant code inside the identity server is the highest-risk extension point. Arbitrary JavaScript in the login origin is forbidden (ADR-0068). Server-side code has to be sandboxed, metered and unable to bypass the core invariants.

## Decision

1. **Runtime:** wasmtime, Component Model (WASI 0.2 worlds). Only Auvora-defined host interfaces are exposed. There is **no WASI filesystem, sockets, clocks beyond a monotonic clock, or environment**.
2. **Host interface `auvora:action@1`:**
   - `context` (read-only, a hook-point-specific view)
   - `result` (add claims / deny with reason / require step-up / set metadata)
   - `http-fetch` (only to the per-action allowlist; through the egress layer, ADR-0103; max 3 calls; 2 s each)
   - `kv` (per-project namespace, 1 MiB, 10k keys)
   - `secrets-get` (named action secrets, envelope-encrypted)
   - `log` (structured, rate-limited)
3. **Resource limits (defaults; `[hooks.wasm]` keys):**
   - fuel 50 M instructions
   - linear memory 64 MiB
   - wall-clock 500 ms per invocation (2 s max via config)
   - module size 10 MiB
   - max 10 actions per hook point
4. **Invariant enforcement:** actions can only *add* data, *deny*, or *require more* (ADR-0038). Returned claims pass the reserved-claim blocklist (`iss`, `sub`, `aud`, `exp`, `iat`, `nbf`, `jti`, `cnf`, `act`, `acr`, `amr`, `auth_time`, `sid`) and the claim size budget.
5. **Failure mode** is configurable per action: `fail_closed` (default for pre-token and pre-login) or `fail_open`. Timeouts and traps are logged and counted. 5 consecutive failures trip a circuit breaker for 60 s.
6. **Isolation tiers:**
   - Default: in-process instance pool with pre-instantiation and pooling allocator.
   - `isolation = "process"` runs actions in a separate sandboxed worker process (seccomp on Linux). This is mandatory on Auvora Cloud.
7. **Authoring:** SDKs for TypeScript (via componentize-js), Rust, Go (TinyGo) and Python (componentize-py). Templates come from `auvora actions new`. Local test runner: `auvora actions test`.
8. **Deployment:** actions are versioned config objects (ADR-0070). Each is content-addressed with an SHA-256 module hash. Modules are AOT-compiled and cached per node. Updates can roll out by percentage with a shadow-mode option.
9. **Supply chain:** module upload requires the `Developer` role or higher. An optional signature requirement (`require_signed_modules`) uses cosign keyless or a project key.
10. **Phase:** P8 GA. A private preview behind a feature flag from P5 is allowed.

## Rationale

Wasm gives deterministic metering and capability-based security in-process without a Node or V8 runtime (embedded-first, ADR-0006). The Component Model gives typed interfaces across languages.

## Alternatives considered

- **Embedded V8/Deno.** Rejected: large attack surface, a polyglot runtime, weaker metering.
- **Lua.** Rejected: weaker sandboxing guarantees and ecosystem.
- **External functions only (sync hooks).** Kept as tier 2, but it isn't sufficient for customers who don't want to host services.

## Consequences

- Positive: Auth0-Actions parity with stronger isolation.
- Negative: authoring toolchains vary in maturity by language.
- Risks: wasmtime CVEs. Mitigated by tracking wasmtime releases within 7 days, process isolation in Cloud, and external review before GA.

## Fallback / revisit trigger

A wasmtime sandbox-escape CVE without a patch within 14 days forces process isolation everywhere until it's fixed. If the external review finds a design flaw, GA is delayed.

## Implementation notes

- Crate: `auvora-hooks` (`wasm` module).
- Metrics: invocation latency, fuel used, failures per action.

## Verification

- A fuzzed host interface.
- An escape test suite (memory, fuel, time, egress bypass, reserved-claim overwrite).
- An external sandbox review before P8 exit.
