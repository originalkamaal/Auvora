# ADR-0113: Brand assets location

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §13.3
- **Related:** ADR-0004, ADR-0021, ADR-0048, ADR-0111 (amends its §2 documentation layout)

## Context

The owner supplied the Auvora logo and wants it in the repository and in the README header. ADR-0111 §2 defines a closed `docs/` layout with no place for brand assets. Several consumers will use the same assets: the root README, the docs site (P3), the hosted UI and console (default theme), and the Authenticator app. Copying the files into each consumer without a single source of truth would cause drift.

## Decision

1. **Amends ADR-0111 §2:** add `docs/brand/` as the single canonical home for brand assets (logos, marks, icons, brand colours) and brand usage guidance. All other ADR-0111 rules still apply.
2. **Files:**
   - Originals are kept unmodified as masters: `logo.jpeg` is the owner-supplied original (1600×1200).
   - Derived variants are named `logo-<variant>.<ext>` (for example, `logo-mark.jpeg` is the tightly cropped mark used in the README). They are regenerated from the master and listed in `docs/brand/README.md` with how they were produced.
   - Vector (`.svg`) and transparent (`.png`) variants are added when available. Once an SVG master exists it becomes the preferred master.
3. **Consumers reference, they don't fork.** The README links `docs/brand/`. Build-time consumers (docs site, hosted UI, console, mobile) copy from `docs/brand/` in their build step rather than committing duplicates. The `@auvora/tokens` package (ADR-0022) may republish the assets as build output.
4. **Licensing:** the logo and name are trademarks of the owner. They aren't covered by AGPL-3.0 or Apache-2.0 grants; `docs/brand/README.md` states the usage terms. This is consistent with ADR-0004.

## Rationale

Brand assets are reference material shared by every surface. `docs/` is the repository's home for shared reference material, and one directory gives one source of truth.

## Alternatives considered

- **`.github/assets/`:** suits README images only. The docs site, UI and mobile app would need duplicates. Rejected.
- **`packages/tokens/assets/`:** ties trademark files to an Apache-2.0 package and only serves JS consumers. Rejected as the master location (republishing as build output is allowed).
- **New top-level `brand/` directory:** expands the closed root set for a small asset folder. Rejected.

## Consequences

- Positive: one canonical location; consistent branding across surfaces.
- Negative: build steps of consumers must copy assets.
- Risks and mitigations: the trademark could be mistaken as open-licensed. Mitigated by explicit usage terms in `docs/brand/README.md` and by ADR-0004.

## Fallback / revisit trigger

If the asset set grows into a full design system (fonts, illustration library, more than 50 files), revisit whether it should move to its own package via a superseding ADR.

## Implementation notes

- Create `docs/brand/README.md` (asset table + usage terms), list `docs/brand/` in `docs/README.md`, and use `logo-mark.jpeg` in the root README header.
- Add "Amended by ADR-0113" to ADR-0111's Related line.

## Verification

`scripts/check-repo-layout` requires `docs/brand/README.md` and its listing in `docs/README.md`. Code review rejects brand files committed outside `docs/brand/` (except generated build output).
