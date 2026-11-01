# ADR-0044: Supported platforms

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D44, §5.2, §5.6
- **Related:** ADR-0005, ADR-0006, ADR-0045, ADR-0012, ADR-0022, ADR-0024

## Context

Support commitments must be explicit so that tests, CI matrices and docs are bounded, and so that users know what's guaranteed.

## Decision

### Server

| Item | Supported |
|---|---|
| OS | Linux (glibc ≥ 2.31 and musl static builds), amd64 + arm64 |
| Dev only | macOS (arm64, amd64), Windows via WSL2 |
| PostgreSQL | 16, 17, 18 (and newer majors within 6 months of release). Minimum 16. Required extensions: `pg_trgm`, `pgcrypto` (optional: `hll`) |
| Managed Postgres verified | AWS RDS/Aurora PostgreSQL, Google Cloud SQL/AlloyDB, Azure Database for PostgreSQL Flexible, Neon, Crunchy Bridge |
| Valkey (optional) | 8.x; Redis OSS 7.2 API-compatible accepted |
| NATS (optional) | 2.10+ with JetStream |
| Kubernetes | Latest 3 minor versions (N-2) at release time |
| Packaging | OCI images (distroless, multi-arch), Helm chart, Docker Compose, static binary tarballs, Homebrew (dev), deb/rpm |
| IaC | Terraform/OpenTofu modules (AWS, GCP, Azure) under `deploy/terraform-modules`. **No Ansible** |

### Clients

| Item | Supported |
|---|---|
| Browsers (hosted UI, console) | Last 2 major versions of Chrome, Edge, Firefox, Safari (macOS/iOS), Samsung Internet; Firefox ESR current |
| iOS SDK | iOS 16+ |
| Android SDK | API 26+ (Android 8.0) |
| Node SDK | Active LTS + Maintenance LTS |
| Python SDK | Supported CPython versions (not EOL) |
| Go SDK | Last 2 Go releases |
| Java SDK | 17, 21 LTS (+ newer LTS) |
| .NET SDK | Current LTS versions |

### Support levels

- **Tier 1** (full CI, release-blocking): Linux amd64/arm64, Postgres 16–18, K8s N-2, the browsers above.
- **Tier 2** (CI, non-blocking): musl static builds, managed Postgres variants (nightly).
- Anything not listed is unsupported. Issues there are best-effort.

## Rationale

Postgres 16 brings needed features (logical replication improvements, `pg_stat_io`, better partitioning performance) and is widely available on managed services. Linux-only production keeps the matrix tractable.

## Alternatives considered

- **Postgres 14/15 support**: rejected because they're near EOL during the build-out.
- **Windows Server native**: rejected as low demand with high cost. WSL2 is enough for dev.
- **Ansible playbooks**: rejected as duplicate effort next to Helm/Compose/Terraform.

## Consequences

- Positive: bounded test matrix.
- Negative: users on older Postgres must upgrade.

## Fallback / revisit trigger

Re-evaluated at each major release. Platforms are added on demand when ≥ 3 paying/design-partner requests exist.

## Implementation notes

P0 CI matrix (ADR-0046). Compose + Helm in P1. Terraform modules in P3. A startup check refuses Postgres < 16 with a clear error.

## Verification

CI matrix jobs per Tier 1 platform; Playwright browser matrix; a startup version check covered by a test.
