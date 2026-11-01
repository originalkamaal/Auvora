# ADR-0048: Documentation site

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §4 D48, §13.5, §29
- **Related:** ADR-0020, ADR-0023, ADR-0024, ADR-0045, ADR-0104, ADR-0107

## Context

Developer experience is a pillar (U4). Docs must be versioned with releases, include API references generated from the source of truth, and contain examples that are proven to work.

## Decision

1. **Tool:** Astro **Starlight** in `docs/site/`, built with pnpm, deployed to `https://docs.auvora.dev` (Cloud-hosted static site; mirrored as a downloadable archive per release for air-gapped users).
2. **Information architecture (Diátaxis):** Tutorials (quickstarts per framework), How-to guides, Reference (API, config, CLI, events, errors, claims), Explanation (concepts, architecture, security model). Separate sections: Self-hosting, Cloud, Migration, Security/Trust.
3. **Generated references** (never hand-written):
   - Admin and Runtime API reference from OpenAPI 3.1 (`crates/auvora-api` generation) rendered with Scalar.
   - Config reference from the `auvora-config` schema (JSON Schema → MDX).
   - CLI reference from `clap` metadata.
   - Event catalog from the CloudEvents JSON Schemas (ADR-0080).
   - Error catalog from the RFC 9457 problem-type registry.
   - Cedar schema and ReBAC model reference from the Auvora entity schema.
4. **Versioning:** the docs version follows the server minor (`/docs/1.2/`); `latest` points to the newest stable. Unreleased features are documented on `next` from `main`.
5. **Tested examples:** every code snippet lives in `examples/` or `docs/site/snippets/`, is compiled and run in CI against `auvora dev` (snippet extraction + test harness), and is embedded by reference. A snippet that fails CI blocks the docs build.
6. **ADRs and threat models** are published under "Architecture" from `docs/adr/` and `docs/threat-models/` (with sensitive threat-model details excluded via front-matter `public: false`).
7. **Search:** Pagefind (static, no third-party service).
8. **Localization:** English source. Community translations are deferred until GA, with Starlight i18n prepared.
9. **Docs as definition of done:** no feature merges without docs (ADR-0107).

## Rationale

Starlight is static, fast, accessible, MDX-capable and keeps tooling in the same JS ecosystem as the frontend.

## Alternatives considered

- **Docusaurus**: viable, but heavier React runtime.
- **mdBook**: Rust-native, but weak for multi-product versioned sites and interactive API refs.
- **Hosted SaaS (Mintlify, ReadMe)**: rejected because of vendor lock-in and the lack of an air-gapped option.

## Consequences

- Positive: docs can't drift from APIs; offline docs bundle.
- Negative: generation pipelines must be maintained.

## Fallback / revisit trigger

If Starlight versioning support becomes insufficient for > 6 concurrent versions, evaluate Docusaurus (new ADR).

## Implementation notes

P0 skeleton + ADR publishing; P1 API/config generation; P3 full quickstarts and public launch.

## Verification

The CI docs job builds the site, runs snippet tests and link checks (lychee), and fails on missing reference pages for any OpenAPI operation.
