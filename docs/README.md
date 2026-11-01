# Auvora documentation

This is the map of all documentation in the repository. Placement rules are in [ADR-0111](adr/0111-repository-organization-and-file-placement.md). Each kind of content has exactly one home: link to it, never duplicate it.

| Directory | Contains | Status |
|---|---|---|
| [`product/`](product/) | Master plan ([`plan.md`](product/plan.md)), `archive/` (superseded plans), `reviews/` (dated reviews) | Active |
| [`adr/`](adr/README.md) | Architecture Decision Records: why each decision was made (index in `adr/README.md`) | Active |
| `specs/` | Technical specifications: how a subsystem works | Created in P0 |
| `threat-models/` | One threat model per feature (STRIDE + LINDDUN, ADR-0049) | Created in P0 |
| `security/` | Security policies, crypto/PQ guides, review reports | Created in P0 |
| `roadmap/` | Phase status (`STATUS.md`) and exit-criteria tracking | Created in P0 |
| `releases/` | Release notes and phase exit reports | Created at first release |
| `runbooks/` | Operational runbooks (incident, key compromise, DR) | Created in P1 |
| `site/` | Public documentation site (Astro Starlight, ADR-0048) | Created in P3 |

When you create a directory listed as "Created in…", add its `README.md` and change its status here in the same commit.
