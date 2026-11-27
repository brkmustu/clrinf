# Changelog

All notable changes are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [Unreleased]

### Added
- `clrinf lsp`: pure-Rust LSP 3.17 stdio diagnostics server publishing `verify` violations on `textDocument/didOpen` and `textDocument/didSave`
- `clrinf doctor --fix`: automatic remediation for stale agent instruction blocks (`clrinf agents sync`) and missing hooks
- MCP server enrichment: `resources/list` & `resources/read` (`clrinf://rules`, `clrinf://health`, `clrinf://topology`), `clrinf_doctor` tool, and `format: "sarif"` option on `clrinf_verify`
- `clrinf doctor [--json]`: environment & guardrails health check across `clrinf.rules.toml`, agent instruction sync, installed hooks, schemas, `verify`, and federated workers
- SARIF 2.1.0 output via `clrinf verify --format sarif --output <path>` and `action.yml` `sarif-file` input for GitHub Code Scanning annotations
- Brownfield `@clrinf:publishes <event>` and `@clrinf:handles <event>` comment annotations integrated into `drift.rs` (including `DRIFT_UNKNOWN_ANNOTATED_EVENT`) and `impact.rs`
- `clrinf init --guardrails [--ci]`: one-command brownfield onboarding wizard detecting schemas, domain folders, and sibling modules
- `[[boundaries]]` cross-module import isolation rules (`RULE_MODULE_BOUNDARY_VIOLATION`) with configurable `public` contract segments
- `[generated] protect = true` incremental protection against hand-editing generated files (`RULE_GENERATED_FILE_MODIFIED`)
- Translated 4 key entry guides (`agent-guardrails.md`, `benchmarks/README.md`, `parity-matrix.md`, `existing-project.md`) to English

### Fixed
- Resolved all Rust `clippy --all-targets -- -D warnings` lints in `tools/clrinf-codegen`
- Upgraded `SQLitePCLRaw.bundle_e_sqlite3` to `2.1.13` in `CrmMonolith.csproj` to clear `NU1903`

## [0.1.0] - 2026-11-27

### Added
- Migrated repository and language package manifests (`Cargo.toml`, `package.json`, `mix.exs`, `.csproj`) from BSL 1.1 to Apache License 2.0
- Language-aware pure-Rust AST lexer & structural import analyzer (`ast.rs`) with `ForbidScope` (`raw`, `code`, `imports`) and Elixir scanning support
- English-first CLI output and documentation (`README.md` + `README.tr.md`), Windows `x86_64-pc-windows-msvc` release packaging, and reusable GitHub Action (`action.yml`)
- Reproducible polyglot guardrails & AOT CRUD context-cost benchmark fixture (`examples/guardrails-benchmark/`, `scripts/bench-guardrails.{ps1,sh}`)
- AOT CRUD source generator (`ClrinfCS.Generators`): `[GenerateCrudEndpoints]`, per-operation handlers/endpoints, Native & Mediator.Net dispatchers, Smart Override, `CLRCRUD001-003`
- CRUD runtime contracts (`ClrinfCS.Core`): `CrudOperations`, `CrudQuery`/`PagedResult<T>`, `ICrudStore`, `IBefore{Create|Update|Delete}Rule<T>`, `CrudRuleRunner`
- EF Core CRUD adapter (`ClrinfCS.Adapters.EfCore`) with tenant-scoped reads/writes and 409 conflict mapping
- `clrinfcs eject` and `clrinfcs crud-map`; `clrinf-codegen eject|crud-map` proxies and `clrinf_crud_eject` / `clrinf_crud_map` MCP tools
- `clrinf add entity --pattern aot`, `clrinf rule new --on before-create|before-update|before-delete`
- `ARCH005` hook placement linter and `ARCH002` hook-interface support
- `examples/AotCrud` compile-time CRUD reference with a delete hook
- Contract-first multi-language framework (Rust, C#, TypeScript, Elixir)
- `clrinf-codegen` CLI: schema validation, multi-language codegen, topology validation, Meta MCP server
- Cross-service Pub/Sub topology validation (TOPOLOGY_DEAD_EVENT, TOPOLOGY_ORPHAN_SUBSCRIBER, TOPOLOGY_UPCASTER_MISSING)
- Idiomatic business rule engines per language
- Static architecture linters: Roslyn (ARCH001-ARCH004), Syn AST (RUST_ARCH001), TS Compiler API (ARCH_TS_001)
- Module catalog and adoption engine (`clrinf catalog`, `clrinf module adopt`)
- Cedar ABAC multi-tenant authorization
- Meta MCP Gateway (JSON-RPC 2.0 stdio) for AI coding agents
- Elixir live streaming bridge adapter
- Conformance test suite for cross-language wire format validation
