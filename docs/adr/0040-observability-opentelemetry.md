# ADR-0040: Observability: OpenTelemetry

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D40, §5.6, §23 (v0.2 §23 measure list), §25
- **Related:** ADR-0005, ADR-0033, ADR-0039, ADR-0041, ADR-0105, ADR-0072

## Context

Operators need traces, metrics and logs to run Auvora reliably. v0.2 named Prometheus, Grafana and Loki/OpenSearch, but the product must not dictate a backend stack. Observability data must not leak PII or secrets.

## Decision

1. **Instrumentation:**
   - `tracing` + `tracing-opentelemetry` with OTLP export (gRPC or HTTP) for **traces, metrics and logs**.
   - OpenTelemetry semantic conventions (HTTP, DB, messaging) plus Auvora conventions prefixed `auvora.*` (`auvora.project_id`, `auvora.flow.type`, `auvora.grant_type`, `auvora.hook.point`).
2. **Metrics:**
   - A Prometheus scrape endpoint at `/metrics` on the internal admin listener (`[observability] metrics_listen = "127.0.0.1:9464"`), plus OTLP metrics push.
   - Core metric families:
     - `auvora_http_request_duration_seconds`
     - `auvora_login_total{method,outcome}`
     - `auvora_token_issued_total{grant}`
     - `auvora_password_hash_queue_depth`
     - `auvora_revocation_propagation_seconds`
     - `auvora_config_propagation_seconds`
     - `auvora_hook_duration_seconds`
     - `auvora_outbox_lag_seconds`
     - `auvora_db_pool_*`
     - `auvora_cache_hit_ratio`
     - `auvora_comms_delivery_total`
     - `auvora_authz_check_duration_seconds`
   - **Cardinality rule:** `project_id` labels are **off** by default (`metrics.per_project_labels = false`) and enabled only for small installs; per-project usage goes to analytics (ADR-0043).
3. **Logs:**
   - Structured JSON to stdout by default (`[observability] log_format = "json" | "pretty"`), with OTLP log export optional.
   - Levels are controlled by `AUVORA_LOG` (EnvFilter syntax).
4. **Redaction:**
   - A `Redacted<T>` type and the `#[sensitive]` field attribute make secrets unloggable. Spans never record tokens, passwords, OTPs, cookies, `Authorization` headers or secret query params.
   - PII-tagged fields (ADR-0072) are hashed or omitted unless `observability.log_pii = true` (dev projects only).
5. **Health endpoints** (internal listener): `/livez`, `/readyz` (DB reachable, migrations compatible, config cache warm, key provider reachable) and `/startupz`.
6. **Trace propagation:**
   - W3C Trace Context in and out, including hooks/webhooks (`traceparent` header).
   - Audit events carry `trace_id` (ADR-0039).
   - Incoming trace context from the public internet is **not trusted for sampling decisions** by default (`observability.trust_incoming_trace = false`); a new root span is created and the external context is linked.
7. **Sampling:** parent-based ratio sampling (default 0.05) with tail-sampling guidance; errors are always exported when the collector supports it.
8. **Shipped artifacts (not runtime dependencies):** Grafana dashboards (JSON), Prometheus alert rules, and an example OpenTelemetry Collector config in `deploy/observability/`. Loki/OpenSearch/Tempo/Jaeger aren't bundled.
9. **Per-tenant observability for Cloud customers:** log/audit streaming via ADR-0039 sinks; operational metrics per project in console analytics.

## Rationale

OpenTelemetry is vendor-neutral and keeps the binary backend-agnostic. Built-in redaction at the type level prevents the most common identity-system leak: tokens in logs.

## Alternatives considered

- **Bundled Prometheus/Grafana/Loki stack**: violates the minimal profile. Rejected; provided as artifacts.
- **Per-project metric labels by default**: cardinality explosion at 100k projects. Rejected.

## Consequences

- Positive: works with any backend; safe by construction.
- Negative: operators must bring a backend. Compose/Helm examples include optional ones.
- Risks and mitigations: missing redaction on new fields is mitigated by a CI log scanner in E2E and the `Redacted<T>` lint.

## Fallback / revisit trigger

None.

## Implementation notes

P0: tracing setup, redaction types, health endpoints. P1: core metrics. P3: dashboards and alert rules. Crates: all, with shared setup in `auvora-server`. Config `[observability]`, env `AUVORA_OBSERVABILITY__OTLP_ENDPOINT`, `OTEL_*` standard env vars honored.

## Verification

E2E log/trace scanner for secrets and PII; a metric cardinality test with 10k projects; health endpoint tests; dashboard JSON validated in CI.
