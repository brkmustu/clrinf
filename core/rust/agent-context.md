# Rust contributor context

This repository provides generic infrastructure for both modular monoliths and
distributed services. Product behavior belongs under `examples/`, never in core.

- `crates/clrinf-core`: framework-free contracts, validated request context, CQRS and rules.
- `crates/clrinf-adapters`: Axum boundary normalization and process-local memory storage.
- `crates/clrinf-cli`: application scaffolding, CQRS generator, Syn AST linter and MCP server.
- `examples/clrinf-auth`: configurable credentials, RS256/JWKS and authorization service.
- `examples/generic`: one application module exposed through in-process and HTTP composition.
- `examples/clrinf-kampanya`: retained campaign showcase; not a generic infrastructure module.
- `examples/demo-auth`: explicitly enabled showcase identities and policy configuration.

Preserve contract field names from local `contracts/schemas`.
Do not infer error/compensation from event names. Never invent tenant identities,
JWT keys, credentials, permissions or a privileged service-account bypass.

Run `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`
and `cargo fmt --all -- --check`. Core conformance tests require the parent's
`tests/conformance/fixtures` or an explicit `CLRINF_CONFORMANCE_DIR`; absence is
an error, not a skipped test. Private keys belong outside source control.

Memory adapters are not durable and do not make external domain writes atomic.
Any durable adapter must document its real transaction and delivery boundaries.
