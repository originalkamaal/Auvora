# ADR-0061: Product naming and namespaces

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md title, §5.5, §4 D18, D23
- **Related:** ADR-0018, ADR-0023, ADR-0024, ADR-0004

## Context

The v0.2 plan was titled "Universal IAM Platform". Inconsistent names across binaries, packages, env vars, API prefixes and ID prefixes create support burden, and once SDKs are published they are effectively impossible to rename. Namespaces must be reserved before the first public artifact.

## Decision

| Surface | Name |
|---|---|
| Product | **Auvora** (never "Universal IAM Platform") |
| Single binary and CLI | `auvora` (subcommands: `serve`, `dev`, `migrate`, `config`, `authz`, `ssh`, …) |
| Rust crates | `auvora-*` (e.g. `auvora-core`, `auvora-oauth`); EE crates `auvora-ee-*` |
| npm | scope `@auvora/*` (`@auvora/js`, `@auvora/react`, `@auvora/nextjs`, `@auvora/elements`, `@auvora/node`, `@auvora/mcp`) |
| Python | `auvora` (PyPI) |
| Go | `github.com/auvora/auvora-go` |
| JVM | `dev.auvora:*` (Maven Central) |
| .NET | `Auvora.*` (NuGet) |
| Swift / Kotlin | `Auvora` (SPM), `dev.auvora:auvora-android` |
| Container images | `ghcr.io/auvora/auvora`, `ghcr.io/auvora/auvora-ee` |
| Helm chart | `auvora/auvora` |
| Terraform provider | `auvora/auvora` |
| Config file | `auvora.toml`; env override prefix `AUVORA_` with `__` as nesting separator (`AUVORA_DATABASE__URL`). Only bootstrap variables that exist before config is parsed are flat: `AUVORA_CONFIG`, `AUVORA_LOG`, `AUVORA_ROOT_KEY_FILE`, `AUVORA_ROOT_PASSPHRASE`, `AUVORA_LICENSE_FILE` |
| HTTP API prefix | `/v1/…` (admin under `/v1/admin/…`, runtime under `/v1/…`, flows under `/v1/flows/…`) |
| Postgres | schema `auvora`; settings namespace `auvora.*` (e.g. `auvora.project_id`) |
| Event types | `auvora.<entity>.<action>.v<N>` |
| Cookies | `__Host-auvora_session`, `__Host-auvora_csrf` |
| Custom claims namespace | `https://auvora.dev/claims/…` only when a claim cannot be a registered or flat claim |
| External ID prefixes | Defined only in ADR-0018 (single source of truth; e.g. `usr_`, `org_`, `prj_`, `gnt_`, `hook_`). This ADR does not restate the table |

- **Public domain:** the owner fills this in. Until the owner confirms availability and registration, the placeholder is `auvora.dev`, with cloud hosts `{project}.{region}.auvora.cloud`-style patterns kept configurable via `auvora.toml` `[cloud] base_domain`. No code may hard-code the domain; it comes from config.
- GitHub org: `auvora` (claim at P0; fallback `auvora-dev`).

## Rationale

One consistent prefix across all ecosystems improves discoverability and makes secret scanning, log parsing and support easier. Prefixed IDs make leaked identifiers self-describing.

## Alternatives considered

- **Keep "Universal IAM"**: generic and not trademarkable.
- **Different names per ecosystem**: confusing, and it hurts SEO and support.

## Consequences

- Positive: predictable naming; namespaces reserved early.
- Negative: we must squat names across registries in P0.
- Risks: name or trademark conflict. Mitigation: trademark search before beta (0.5).

## Fallback / revisit trigger

A trademark conflict or registry unavailability for `auvora` on ≥ 2 major registries leads to a superseding ADR choosing a qualified name (`auvora-id`).

## Implementation notes

P0: register the GitHub org, npm scope, crates.io names (`auvora`, `auvora-core` placeholders), PyPI, Docker/GHCR, and the domain. Add a `NAMING.md` lint list.

## Verification

A CI lint rejects new crate or package names that don't match the patterns, and rejects hard-coded domain strings outside config defaults and docs.
