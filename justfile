# Task runner (ADR-0023 §7). `just` lists recipes; `just check` runs what CI runs.

default:
    @just --list

# One-time setup: install the commit-msg hook (ADR-0112).
setup:
    git config core.hooksPath scripts/git-hooks
    @echo "git hooks installed from scripts/git-hooks"

# Format all Rust code.
fmt:
    cargo fmt --all

# Format check, as CI runs it.
fmt-check:
    cargo fmt --all --check

# Clippy with warnings denied (ADR-0047).
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run all Rust tests.
test:
    cargo test --workspace --all-features

# Licenses, crypto bans, advisories and sources (ADR-0004, ADR-0012, ADR-0046).
deny:
    cargo deny check

# Dependency audits against trusted audit sets (ADR-0046).
vet:
    cargo vet --locked

# Secret scanning over the full git history (ADR-0046).
secrets:
    gitleaks git --no-banner --redact .

# Repository and documentation rules (ADR-0000, ADR-0111).
check-docs:
    scripts/check-adrs
    scripts/check-repo-layout

# Commit messages since origin/main (ADR-0112).
check-commits range="origin/main..HEAD":
    scripts/check-commits {{range}}

# Everything CI runs locally. Run before every commit.
check: fmt-check lint test deny vet secrets check-docs

# Run the auvora binary, e.g. `just run version`.
run *args:
    cargo run --quiet --bin auvora -- {{args}}
