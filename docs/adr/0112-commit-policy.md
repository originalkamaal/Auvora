# ADR-0112: Commit policy

- **Status:** Accepted
- **Date:** 2026-11-01
- **Deciders:** Kamaal Khan (owner)
- **Plan refs:** plan.md §29
- **Related:** ADR-0045, ADR-0046, ADR-0107, ADR-0111

## Context

Auvora is a long-lived, security-critical monorepo. History must be bisectable, auditable and readable years later. The owner requires:
- commits at every logical stage of work;
- every commit non-breaking and working;
- clear messages;
- **no co-authored or AI-attribution text** in commit messages.

## Decision

1. **Commit at every logical stage.** A commit is one coherent, reviewable unit, for example "add ADR set," "scaffold workspace," or "implement PKCE verifier with tests." Don't batch unrelated changes, and don't leave completed logical work uncommitted at the end of a session.
2. **Every commit is non-breaking and working.**
   - The tree builds.
   - Formatting, lint and the tests relevant to the change pass (`just check` once it exists; before P0 scaffolding, the doc checks).
   - No commit knowingly leaves `main` broken. Work in progress that isn't working stays uncommitted or on a branch, never on `main`.
   - Schema changes follow expand/contract (ADR-0025), so every commit is deployable.
3. **Message format:** Conventional Commits.
   ```
   <type>(<scope>): <imperative summary, ≤ 72 chars, no trailing period>

   <body: what changed and why, wrapped at 72; reference ADRs/issues, e.g. "Implements ADR-0029.">
   ```
   - Types: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `security`, `revert`.
   - Scope: the crate, app, package or docs area (`oauth`, `console`, `adr`, `plan`…).
   - A body is required whenever the summary alone doesn't explain *why*.
   - Breaking API changes use `!` and a `BREAKING CHANGE:` footer (rare; see ADR-0020 and ADR-0045).
4. **No attribution trailers.** Commit messages MUST NOT contain `Co-Authored-By:`, "Generated with …", tool or AI attribution, or emoji signatures. The same applies to PR titles and descriptions, tags and release notes. This rule overrides any tool default.
5. **Hygiene:**
   - Never commit secrets, `.temp/` contents or local environment files.
   - Commits on `main` are linear (rebase or squash merge); no merge commits.
   - Maintainers sign commits (ADR-0046) once signing is configured.
6. **Changelog** is generated from Conventional Commit messages (ADR-0045), so message quality directly determines release-note quality.

## Rationale

Small, working, well-described commits make `git bisect`, reverts, audits and changelog generation reliable. Excluding attribution trailers keeps history uniform and reflects the owner's explicit directive.

## Alternatives considered

- **Large periodic commits:** hard to review, bisect and revert. Rejected.
- **Free-form messages:** inconsistent, and the changelog can't be automated. Rejected.
- **Allow WIP commits on main:** breaks bisectability and the "always deployable" property. Rejected.

## Consequences

- Positive: bisectable, auditable history; automated changelogs; predictable reverts.
- Negative: requires discipline in splitting work and running checks before each commit.
- Risks and mitigations: tool defaults that add attribution trailers are overridden by this ADR and `CLAUDE.md`. A `commit-msg` hook enforces it.

## Fallback / revisit trigger

None. This is an owner directive.

## Implementation notes

P0 adds:
- a `commit-msg` hook (installed via `just setup`) that validates Conventional Commit format and rejects attribution trailers (`Co-Authored-By`, `Generated with`);
- a CI job `check-commits` that applies the same rules to every PR commit;
- `CONTRIBUTING.md`, which summarizes this ADR.

## Verification

`check-commits` CI job and the local `commit-msg` hook. Quarterly spot review of `git log` on `main`.
