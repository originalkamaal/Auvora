# Security policy

Auvora is security software, and we take vulnerability reports seriously. The full process is in [ADR-0108](docs/adr/0108-vulnerability-disclosure-and-advisories.md).

## Reporting a vulnerability

**Do not open a public issue.** Report privately through GitHub's **"Report a vulnerability"** button (private vulnerability reporting) on this repository's Security tab.

Please include the affected component and version or commit, reproduction steps, impact, and any suggested fix.

## What to expect

| Step | Target |
|---|---|
| Acknowledgement | 2 business days |
| Triage and severity assessment | 5 business days |
| Fix for critical issues | 7 days |
| Fix for high / medium / low | 30 / 90 / next release |

We coordinate disclosure with you, publish a GitHub Security Advisory with a CVE where applicable, and credit reporters who want to be credited.

## Supported versions

Auvora is pre-release (Phase 0). Until the first release, only the `main` branch receives fixes. After that, the support policy in [ADR-0045](docs/adr/0045-release-and-support-policy.md) applies.

## Bug bounty

A bug bounty programme opens with the public beta (0.5).
