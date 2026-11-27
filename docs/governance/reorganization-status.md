# Reorganization Status

## Architectural Boundaries

- `core/rust`, `core/csharp`, `core/typescript`, and `core/elixir` are consolidated directly inside this monorepo (migrated from standalone git submodules).
- Example business domains (`examples/contracts`, `templates/eticaret-showcase`) are strictly separated from core wire-format schemas (`tools/clrinf-codegen/schemas`).
- Core contracts, optional runtime adapters, developer guardrails/codegen CLI, and showcase apps are documented separately.
- Local modular monoliths and zero-lock-in adoption in existing repositories are first-class paths.
- Package and repository licensing is standardized under **Apache-2.0** (`LICENSE`).

## Status

| Phase | Scope | Status |
|---|---|---|
| Monorepo consolidation | `core/{rust,csharp,typescript,elixir}` merged into monorepo; unified CI & conformance gates | Complete |
| Contract & example separation | Core schemas separated from domain examples; generic starter templates | Complete |
| Guardrails & topology CLI | `clrinf.rules.toml`, `verify`, `plan`, `agents sync`, agent hooks, `topology drift/impact/report`, `lean` MCP | Complete |
| Apache-2.0 & English-first launch | Apache-2.0 migration, English-first docs/CLI, Windows release artifact, reusable `action.yml`, language-aware AST scanner | Complete |

## Key Design Decisions

- **No forced persistence engine across all languages:** Common wire formats (`_context`, `_error`, `_envelope`) and explicit ports are shared; C# provides a SQLite persistent Outbox/Inbox reference implementation.
- **No unimplemented generic Saga claims:** Distributed compensation and OpenTelemetry instrumentation remain explicit application integration responsibilities.
- **Single-binary static verification:** `tools/clrinf-codegen` performs schema, topology, drift, and import-rule checks in pure Rust without requiring external language toolchains at verify time.
