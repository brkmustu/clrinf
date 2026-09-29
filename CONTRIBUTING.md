# Contributing to clrinf

Thank you for contributing to **clrinf**!

## Repository Structure

This is a monorepo. Each language core lives under `core/`:

| Directory | Language | Description |
|-----------|----------|-------------|
| `core/rust/` | Rust | Async CQRS, Cedar ABAC, Axum adapters |
| `core/csharp/` | C# (.NET 10) | Roslyn linters, DI, SQLite adapter |
| `core/typescript/` | TypeScript (Bun) | Functional core, Event Inspector, AST linter |
| `core/elixir/` | Elixir/OTP | BEAM runtime, live streaming bridge |
| `tools/clrinf-codegen/` | Rust CLI | Codegen, topology validation, Meta MCP server |

## Getting Started

1. Clone the repo (no submodules):
   ```bash
   git clone https://github.com/brkmustu/clrinf.git
   ```n2. Install language toolchains as needed (Rust stable, .NET 10, Bun, Elixir/OTP 27).
3. Validate schemas: `clrinf check --schema-dir tools/clrinf-codegen/schemas`

## Commit Style

Follow [Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`, `chore:`, `docs:`.
