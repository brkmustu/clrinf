# Security Policy

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Report via [GitHub private vulnerability reporting](https://github.com/brkmustu/clrinf/security/advisories/new).

Include: description, reproduction steps, and potential impact.
We aim to respond within 48 hours and release a fix within 14 days.

## Known Security Boundaries

- `clrinf-codegen` processes schema JSON — use only trusted schema directories.
- The MCP stdio server exposes codegen and filesystem operations to AI agents. Run only in trusted environments.
- Cedar policy evaluation uses the [`cedar-policy`](https://github.com/cedar-policy/cedar) crate — keep it updated.
