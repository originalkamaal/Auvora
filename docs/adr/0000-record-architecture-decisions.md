# ADR-0000: Record architecture decisions

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** `docs/product/plan.md` §4 (Decision Register), §28

## Context

Auvora is a multi-year, security-critical product with a very broad scope. The v0.2 plan deferred 16 decisions to "ADRs later," and the owner asked for every decision to be made up front so that nothing remains undecided during development. Decisions need a durable, reviewable, version-controlled home that explains *why* each choice was made, so that future contributors don't re-litigate settled questions or silently drift from them.

## Decision

1. Every architecturally significant decision is recorded as an ADR in `docs/adr/`, named `NNNN-kebab-title.md`.
2. ADR numbers 0001–0060 map 1:1 to Decision Register rows D01–D60 in plan v1.0 §4. ADRs 0061+ record further decisions embedded in the plan's body.
3. Every ADR uses the template below. All sections are mandatory; write "None" when a section doesn't apply.
4. **Statuses:** `Proposed` → `Accepted` → (`Deprecated` | `Superseded by ADR-NNNN`). Accepted ADRs are immutable except for typo fixes and link updates. To change a decision, write a new ADR that supersedes the old one, and update the old one's status line.
5. **Fallback triggers** are pre-agreed revisit conditions. When a trigger fires, a new ADR is written; the old ADR isn't edited.
6. Any PR that contradicts an Accepted ADR is blocked until a superseding ADR is merged.
7. The plan (`plan.md`) and the ADRs must agree. If they conflict, the newer of the two wins, and the other is corrected in the same PR.
8. `docs/adr/README.md` is the index and must be updated in the same PR as any ADR change.

## Template

```markdown
# ADR-NNNN: <Title>

- **Status:** Accepted | Proposed | Deprecated | Superseded by ADR-XXXX
- **Date:** YYYY-MM-DD
- **Deciders:** <names/roles>
- **Plan refs:** plan.md §x, Dxx
- **Related:** ADR-XXXX, ADR-YYYY

## Context
What problem or force requires a decision. Constraints, requirements, standards involved.

## Decision
The decision, stated precisely and normatively (MUST/SHOULD/MAY per RFC 2119 where useful).
Include concrete values (defaults, limits, names) so nothing is left to interpret.

## Rationale
Why this option beats the alternatives for Auvora's goals (superset coverage, embedded-first, security, Postgres-only minimum profile).

## Alternatives considered
- **Option** — why rejected.

## Consequences
- Positive:
- Negative / costs:
- Risks and mitigations:

## Fallback / revisit trigger
Measurable condition that reopens this decision, and the pre-agreed alternative. "None" if permanent.

## Implementation notes
Phase(s), crates/apps affected, configuration keys, migration concerns.

## Verification
How compliance with this ADR is checked (CI rule, test suite, review checklist, metric).
```

## Rationale

Markdown ADRs in the repository are versioned with the code they govern, reviewed in the same PRs, and readable without tools. A 1:1 mapping to the Decision Register keeps the plan and the ADRs traceable.

## Alternatives considered

- **Decisions only in the plan document:** a single large file has no per-decision history and no supersession trail. Rejected.
- **External wiki/Confluence:** drifts from the code and isn't reviewed in PRs. Rejected.
- **Lightweight log without a fixed template:** inconsistent depth, and the "why" gets lost. Rejected.

## Consequences

- Positive: decisions are explicit, searchable and reviewable. New contributors can onboard from the ADR set.
- Negative: maintaining ~110 ADRs takes effort. This is mitigated by the immutability rule (changes go into new ADRs, not edits) and the index.

## Fallback / revisit trigger

None.

## Implementation notes

P0 deliverable. A CI check (P0) verifies that every file in `docs/adr/` is listed in `README.md` and has all mandatory headings.

## Verification

`scripts/check-adrs` in CI: heading presence, status validity, index completeness, and that superseded links resolve.
