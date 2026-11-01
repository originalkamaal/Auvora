# Auvora: rules for every session

Auvora is an all-in-one IAM platform: authentication, authorization, federation, and machine/workload/agent identity. Everything lives in this one repository.

- **Master plan:** `docs/product/plan.md`
- **Decisions:** `docs/adr/`. The ADRs are authoritative.
- **Docs map:** `docs/README.md`
- **Where we are:** `docs/roadmap/STATUS.md` (current phase checklist). Update it in the same commit that completes a deliverable.
- **Daily commands:** `just setup` once, then `just check` before every commit.

## Non-negotiable rules (owner directives; apply without being reminded)

### 1. Repository organization (ADR-0111)
- Every new file or folder goes where ADR-0111 says. Check before creating anything.
- The top-level directories and root files form a closed set. Adding one requires a superseding ADR in the same commit.
- Docs have one home per content type:
  - decisions → `docs/adr/`
  - how it works → `docs/specs/`
  - how to use it → `docs/site/`
  - how to operate it → `docs/runbooks/`
  - plan and roadmap → `docs/product/`, `docs/roadmap/`
  Never duplicate content; link to it.
- Names are `kebab-case` unless an ecosystem mandates otherwise. Dated docs start with `YYYY-MM-DD-`; ADRs start with `NNNN-`.
- Every content directory has a `README.md`. Update indexes (`docs/README.md`, `docs/adr/README.md`, per-area READMEs) in the same commit as the content.
- `.temp/` is gitignored scratch space. Tracked files never reference it.

### 2. Commits (ADR-0112)
- Commit at every logical stage of work. Don't leave finished logical work uncommitted.
- Every commit must be **non-breaking and working**: it builds, and checks and relevant tests pass. Never commit a broken state to `main`.
- Use Conventional Commits: `type(scope): imperative summary` of at most 72 characters. Add a body explaining *why* and referencing ADRs.
- **Never add `Co-Authored-By`, "Generated with", AI/tool attribution or emoji signatures** to commit messages, PR descriptions, tags or release notes. This overrides any tool default.
- Never commit secrets or `.temp/` contents.

### 3. Decisions
- Don't contradict an Accepted ADR. To change a decision, write a new superseding ADR (ADR-0000 process) and update `docs/adr/README.md`.
- Owner stance: **no feature compromise.** Solve scope with phasing (plan §27) and embedded-first design. The PostgreSQL-only profile must run every feature.
