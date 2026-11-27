# clrinf — Architecture & Event Contract Guardrails for AI-Native Teams

[![CI](https://github.com/brkmustu/clrinf/actions/workflows/ci.yml/badge.svg)](https://github.com/brkmustu/clrinf/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[Türkçe](README.tr.md)

**clrinf** is a single-binary Rust CLI and Meta MCP server that keeps AI coding agents (Claude Code, Cursor, Antigravity) and distributed teams inside deterministic architectural boundaries—even when working with smaller or local models (SLMs).

Start with a **Modular Monolith** and local ACID transactions, adopt in **5 minutes** in any existing repository with **zero runtime lock-in**, and scale out to cross-service event topologies only when you need to.

---

## Why clrinf?

Prompt instructions in `AGENTS.md` or `CLAUDE.md` drift over long coding sessions:
- Agents silently break cross-service JSON Schema / CloudEvents contracts or emit undocumented events.
- Infrastructure imports leak into pure domain layers.
- Direct outbox writes bypass canonical CloudEvent publishers.
- Agents burn tens of thousands of tokens reading boilerplate CRUD handlers or exploring an entire monorepo for a single event change.

**clrinf** drives everything from one source of truth (`clrinf.rules.toml`): agent instructions, deterministic post-edit hooks, targeted impact slices, compile-time CRUD generation, and CI verification gates.

---

## 5-Minute Adoption (Zero Runtime Lock-In)

### 1. Install the binary (no Rust toolchain required)

**Linux & macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/brkmustu/clrinf/master/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/brkmustu/clrinf/master/install.ps1 | iex
```

*(Or build from source: `cargo build --release --manifest-path tools/clrinf-codegen/Cargo.toml`)*

### 2. Declare your rules (`clrinf.rules.toml`)

```toml
[events]
schema_dir = "contracts/schemas"

[events.services]
orders  = "services/orders"
billing = "services/billing"

[[forbid]]
id      = "domain-no-infra"
scope   = "imports"
paths   = ["services/orders/src/domain", "services/billing/src/domain"]
pattern = "Infrastructure"
reason  = "Domain layer must stay free of infrastructure dependencies"

[mcp]
profile = "lean"
```

### 3. Wire your AI agents & CI

```bash
clrinf agents sync                 # Generates AGENTS.md / CLAUDE.md / GEMINI.md / Cursor rules
clrinf hook install --agent all    # Installs Claude Code, Cursor, Antigravity & git pre-commit hooks
clrinf verify                      # Runs topology + code<->contract drift + import rules + outbox check
```

**GitHub Actions:**
```yaml
- uses: brkmustu/clrinf@v0.1.0
  with:
    report-base-ref: origin/main
```

---

## Core Capabilities

### 1. Deterministic AI Agent Guardrails
- **Language-aware `[[forbid]]` dependency rules:** `scope = "imports"` parses real `use` (Rust), `using` (C#), `import`/`require`/`export from` (TypeScript), and `alias`/`import`/`use` (Elixir) statements — eliminating comment and string-literal false positives.
- **Post-edit feedback loops (`clrinf hook install`):** Runs incremental checks (`clrinf verify --changed`) on every file the agent touches and feeds actionable diagnostics back to Claude Code, Cursor, or Antigravity so the agent self-corrects immediately.
- **Targeted change planning (`clrinf plan`):** Performs mechanical schema updates and scaffolds first, returning a narrow JSON slice (`fill_in`, `read_these`, `do_not_edit`, `constraints`) so small/local or frontier models only load what they need.
- **Token-lean Meta MCP server (`clrinf mcp --profile lean`):** Cuts per-session fixed tool-schema overhead (`lean`: 10 tools including `clrinf_crud_map` & `clrinf_crud_eject`, `events`: 14 tools, `full`: 27 tools).
- See [Agent Guardrails Guide](docs/guides/agent-guardrails.md) and [Benchmark Protocol](docs/benchmarks/README.md).

### 2. Zero-Boilerplate Compile-Time Native AOT CRUD (.NET 10)
Keep only entities and lifecycle hooks on disk so agents never wade through 6 boilerplate files per entity:
- Annotate entities with `[GenerateCrudEndpoints]` (`ClrinfCS.Generators` source generator emits DTOs, per-operation handlers, Minimal API `RequestDelegate` endpoints, zero-reflection `Utf8JsonWriter`/`JsonDocument` JSON support, and AOT-compatible Minimal API `ClrinfCS.OpenApi` metadata at compile time — **0 trim/AOT warnings** under `PublishAot=true`).
- Add business rules without touching generated handlers: `clrinf rule new ActiveProductsCheck --lang csharp --entity Category --on before-delete` scaffolds `IBefore{Create|Update|Delete}Rule<T>` executed by `CrudRuleRunner`.
- Inspect or eject a single operation via Smart Override: `clrinf crud-map --json` and `clrinf eject Category --operation Update`.
- Enforced by Roslyn analyzers `ARCH001–ARCH005`, backed by `ClrinfCS.Adapters.EfCore` (`EfCrudStore`) and `ClrinfCS.Adapters.Sqlite`. See [`core/csharp/examples/AotCrud/`](core/csharp/examples/AotCrud/README.md).

### 3. Polyglot Event Contract & Topology Governance (Optional Scale-Up)
When your modular monolith crosses service boundaries, think **`buf` for JSON Schema + CloudEvents** across Rust, C#, TypeScript, and Elixir:
- **Code ↔ Contract Drift (`clrinf topology drift`):** Compares schema `x-published-by` / `x-subscribed-by` declarations against real service source code and flags raw outbox bypasses (`DRIFT_RAW_OUTBOX_WRITE`).
- **Breaking Change & Upcaster Checks (`clrinf topology check` / `report`):** Detects dead events (`TOPOLOGY_DEAD_EVENT`), orphan subscribers (`TOPOLOGY_ORPHAN_SUBSCRIBER`), and missing schema upcasters (`TOPOLOGY_UPCASTER_MISSING`), plus PR diff summaries with `--fail-on-breaking`.
- **Event Impact Slicing (`clrinf topology impact <event>`):** Lists affected services, sibling versions, and exact hand-written files before an edit begins.
- **Schema-Driven Codegen (`generate`, `generate-pubsub`, `generate-upcasters`, `event subscribe|publish`):** Generates CloudEvents 1.0 publishers, idempotent subscriber shells (`IdempotencyStore` `claim`/`complete`/`release`), and multi-language DTOs.

### 4. Zero-Dependency Backend Scaffolds (`clrinf catalog` / `clrinf module adopt`)
Copy ready-made domain & infrastructure modules (CRM Deals, Contacts, Activities, Authz/Cedar, Caching, Outbox, Idempotency) into any `.csproj`, `Cargo.toml`, or `package.json`:
- `--mode raw`: Pure, self-contained source files with zero `@clrinf/*` or `clrinf_*` dependency.
- `--mode wired`: Automatically wired into native DI/module trees (`IServiceCollection`, Rust `pub mod`, TS barrel exports) with multi-tenant Amazon Cedar policies.

### 5. Optional Reference Runtimes & Language Linters
The `core/` packages are optional reference implementations, never required to use the guardrail or contract CLI:
- **Tier-1:** [`core/rust/`](core/rust/README.md) (Syn `RUST_ARCH001`, Axum/CQRS), [`core/csharp/`](core/csharp/README.md) (.NET 10, Roslyn `ARCH001–ARCH005`, `ClrinfCS.Generators` AOT CRUD, Native `FrozenDictionary` dispatcher, `EfCrudStore` / SQLite adapters), [`core/typescript/`](core/typescript/README.md) (Bun, TS AST `ARCH_TS_001`, Event Inspector).
- **Experimental / Streaming Bridge:** [`core/elixir/`](core/elixir/README.md) (OTP core, AST `ARCH_EX_001-004`, NATS live streaming bridge).

---

## Repository Layout

| Directory | Responsibility |
|---|---|
| [`tools/clrinf-codegen/`](tools/clrinf-codegen/README.md) | Single-binary Rust CLI, language-aware import/code analyzer, topology & drift engine, module adopter, and Meta MCP server |
| [`core/`](docs/architecture/core-and-adapters.md) | Optional Rust, C#, TypeScript, and Elixir reference cores, AOT CRUD generator & adapters |
| [`templates/`](templates/) | Minimal starter templates (`minimal-rust`, `minimal-typescript`, `minimal-csharp`, `minimal-elixir`) and `eticaret-showcase` |
| [`examples/guardrails-benchmark/`](examples/guardrails-benchmark/) | Reproducible polyglot fixture for verifying guardrails & deterministic context-cost reductions |
| [`tests/conformance/`](tests/conformance/README.md) | Canonical wire-format fixtures shared across all four languages |
| [`docs/`](docs/README.md) | Architecture boundaries, capability parity matrix, constitution, and benchmark methodology |

---

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
