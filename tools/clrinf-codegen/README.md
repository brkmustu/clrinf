# clrinf-codegen

The root Rust CLI is the canonical contract generator and manifest-based template
catalog. Existing language-specific CLIs remain available for their own use cases.

```sh
cargo install --path tools/clrinf-codegen
# Inspect federated language workers (C#, Rust, TypeScript)
clrinf-codegen worker check
# Run cross-language architectural linter (Roslyn, Syn, TS AST)
clrinf-codegen lint --lang all
# Scaffold an isolated business rule in all 3 languages or specific language
clrinf-codegen rule new CheckMaxDiscount --lang all --entity Order
# Start Model Context Protocol (MCP) JSON-RPC 2.0 stdio server
clrinf-codegen mcp
# Validate cross-service pub/sub topology, dead events, and evolution chains
clrinf-codegen topology check \
  --schema-dir tools/clrinf-codegen/schemas \
  --upcasters-dir tools/clrinf-codegen/schemas

# Generate CloudEvent publisher envelopes and subscriber shells
clrinf-codegen generate-pubsub \
  --schema-dir tools/clrinf-codegen/schemas \
  --templates-dir tools/clrinf-codegen/templates \
  --lang all \
  --output ./generated-pubsub

# Schema validation & contract generation
clrinf-codegen check --schema-dir tools/clrinf-codegen/schemas
clrinf-codegen generate \
  --schema-dir tools/clrinf-codegen/schemas \
  --templates-dir tools/clrinf-codegen/templates \
  --output ./generated
clrinf-codegen template list --templates-dir /path/to/clrinf/templates
# Create greenfield projects with architecture, paradigm, and dispatcher choices
clrinf-codegen new MyShop --lang csharp --arch clean-cqrs --paradigm fp --dispatcher native
clrinf-codegen new MyRustSvc --template minimal-rust --templates-dir /path/to/clrinf/templates

# Batch generate entire monolith/services from codegen.toml
clrinf-codegen generate-all --config ./codegen.toml --path ./apps/MyCrm

# Project initialization & profiles (minimal, standard, full)
clrinf-codegen init --profile standard --dispatcher native

# Explore built-in official module catalog (Domain Suites & Infrastructure Concerns)
clrinf-codegen catalog

# Module adoption & transplantation into existing projects (.csproj, Cargo.toml, package.json)
# Modes: 'raw' (pure self-contained code, zero external dependencies) or 'wired' (hooked into DI / module tree)
clrinf-codegen module adopt crm \
  --to-project ./apps/BillingService/BillingService.csproj \
  --mode raw

clrinf-codegen module adopt deals \
  --to-project ./apps/BillingService/BillingService.csproj \
  --as Sales \
  --mode raw
```

## Modular Application Lifecycle & Module Adoption

`clrinf-codegen` provides brownfield and greenfield modular application scaffolding and cross-project transplantation:
- **Built-In Module Catalog (`catalog`)**:
  - Official Domain Suites & Features: `crm` (full suite: Deals + Contacts + Activities), `deals`, `contacts`, `activities`.
  - Official Cross-Cutting Concerns: `caching`, `logging`, `transaction`, `auth`, `authz` (Multi-Tenant Cedar), `idempotency`, `outbox`.
- **Profiles (`--profile`)**:
  - `minimal`: Pure domain logic with zero forced cross-cutting concerns (lean/pure).
  - `standard`: Balanced default enabling `logging` and `transaction`.
  - `full`: Enterprise batteries-included enabling all 7 cross-cutting concerns (`caching`, `logging`, `transaction`, `authentication`, `authorization` with Cedar, `idempotency`, `outbox`).
- **Dispatchers (`--dispatcher`)**:
  - `native`: `ClrinfCS.Core.Dispatcher` using compile-time `FrozenDictionary` (zero assembly-scanning, allocation-free).
  - `mediatr`: Standard MediatR bridge.
  - `mediatornet`: Mediator.Net bridge.
- **Module Transplantation (`module adopt`)**:
  - Supports transplanting built-in modules or custom modules from external source projects.
  - Automatically resolves project language from target `.csproj`, `Cargo.toml`, or `package.json`.
  - `--mode raw`: Generates self-contained contracts (`OperationClaim`) with zero mandatory external package dependencies.
  - `--mode wired`: Auto-wires into language-idiomatic DI/module tree (`IServiceCollection`, `pub mod`, barrel export) with Cedar multi-tenant authorization guards.
  - Generates multi-tenant Cedar security policy (`policies/<module>.cedar`) alongside every adopted module.
- **Modular Layout Strategies & Decomposition**:
  - `Containerized Modules`: Configured via `[backend.structure] folder_name = "Modules"` (e.g. `CrmMonolith`). Gathers all domain modules under `Modules/<Module>/` to keep the root directory reserved for infrastructure (`Data/`, `Common/`, `Policies/`).
  - `Direct Bounded Contexts`: Used when `folder_name` is omitted (e.g. `EcommerceMonolith`). Places bounded contexts directly at the project root (`Catalog/`, `Inventory/`, `Orders/`).
  - `Patterns (--pattern)`:
    - `flat` (Default): Decomposes modules into clean, focused flat files (`<Module>Objects.cs`, `<Module>Rules.cs`, `<Module>Handlers.cs`, `<Module>Module.cs`) without deep folder nesting.
    - `basic`: Bundles all module definitions into a single `<Module>Module.cs` file.

## Federated Language Worker Model & Meta MCP

`clrinf-codegen` acts as the **central orchestrator and Meta MCP Gateway** for the polyglot ecosystem:
- **Explicit Prerequisites**: C# code generation/linting delegates to C# (.NET in `core/csharp`), Rust to `clrinf-cli` (in `core/rust`), and TypeScript to Bun (in `core/typescript`).
- **Worker Discovery**: Searches system `PATH` first, falling back to local workspace checkouts. Missing tools report clear, actionable installation instructions.
- **Universal Meta MCP**: Exposes `clrinf_list_catalog`, `clrinf_adopt_module`, `clrinf_module_manage`, `clrinf_add_domain_module`, `clrinf_add_entity`, `clrinf_inspect_ecosystem`, `clrinf_lint_architecture`, `clrinf_scaffold_rule`, `clrinf_generate_contracts`, `clrinf_validate_schemas`, `clrinf_validate_topology`, `clrinf_generate_pubsub`, and `clrinf_new_project` via JSON-RPC 2.0 stdio for autonomous AI agents.
- **Deterministic Schema Authority**: `validator.rs` remains the single canonical authority for JSON Schema validation before delegating to workers.

## Cross-Service Pub/Sub & Topology Validation

Distributed event-driven systems require strict mechanical safety guarantees:
- **Publisher Generation (100% Mechanical)**: Stamps canonical CloudEvent envelopes (`specversion: "1.0"`, `id`, `source`, `type`, `time`, `tenantid`, `correlationid`, `causationid`) and enqueues atomically into `OutboxStore`.
- **Subscriber Shell Generation ("Kabuk")**: Handles deserialization, event filtering, and idempotency protection (`claim`, `complete`, `release`). Handlers deliberately fail until domain reactions are implemented.
- **Topology Verification**:
  - `TOPOLOGY_DEAD_EVENT` (Warning): Detects events published by services but never consumed.
  - `TOPOLOGY_ORPHAN_SUBSCRIBER` (Error): Detects events subscribed by a service with no known producer.
  - `TOPOLOGY_UPCASTER_MISSING` (Error): Enforces upcaster migration chains across evolving event versions (e.g. `v1` to `v2`).
- **Transport Agnostic**: Generates code against `OutboxStore` and `IdempotencyStore` abstractions, avoiding hardcoded transport bindings (e.g. NATS, Kafka).


Assets are explicit, not embedded. The installed binary never consults its build
checkout. Defaults refer to the current directory's `tools/clrinf-codegen/schemas`,
`tools/clrinf-codegen/templates`, and `templates`; outside a checkout, supply the
absolute asset paths above. Missing assets are errors, not fabricated catalog
entries. `new` requires an explicit template choice.

## Models and validation

`generate --lang rust|csharp|typescript|elixir` writes one language; omitting
`--lang` writes four independent language subdirectories. Generate core and
application contracts into separate output roots. Inputs are sorted, traversal
errors and symlinks fail, generated filename/type collisions fail, and repeated
generation from the same inputs is deterministic. Generation does not delete
unrelated existing files; use a fresh output directory when removing schemas.

`validate` checks Draft JSON Schema metadata and unique `$id`/message identities.
`check` also checks the supported model subset and generated-name collisions.
Neither command certifies runtime adapters, executes application tests, enforces
the architecture constitution, or compares historical schema versions.

Supported model shapes are object roots, required/optional properties, string,
integer, number, boolean, homogeneous arrays (including nested arrays and object
items), and JSON objects. Nested objects are deliberately generic JSON values,
not recursively generated named classes. Original JSON property names are kept
in every language. `x-domain` is optional namespace metadata, defaults to
`common`, and must be an identifier; it is not a framework dependency.

Constraints such as `enum`, `const`, bounds, patterns and `additionalProperties`
remain **schema validation rules**, not generated DTO validation. Run JSON Schema
validation at the boundary; deserializing a DTO is not schema validation.
Integer formats select fixed-width target types; otherwise Rust/C# use signed
64-bit integers. TypeScript uses `number`, so applications requiring values beyond
its exact-integer range must choose an explicit string-based wire contract.
String `uuid` and `date-time` use `Guid` and `DateTimeOffset` in C#, strings in
other targets. Date/time serialization may normalize equivalent representations.

References, composition (`allOf`/`oneOf`/`anyOf`), nullable/type unions, tuples,
boolean schemas, conditional/dependent schemas and custom raw-code type
overrides fail clearly instead of silently producing false types.
Titles must start uppercase and identifiers must not collide with support files.

## Template manifests

Each catalog entry has `clrinf-template.json` with `name` (directory name),
`description`, `language`, `kind`, `capabilities` (string array), and `quickstart`
(command string array). The CLI copies files and prints commands; it never
executes manifest commands. Catalog order is stable. Scaffolding rejects
traversal names, symlinks, nonempty targets and targets inside the source.

`generate-slice` remains a compatibility command for **legacy adapter sketches**,
not runnable application generation. Its handler/aggregate stubs explicitly
fail until business behavior is implemented. Its Axum, Cedar and .NET assumptions
are not core requirements. Prefer `new --template minimal-<language>` for a
runnable, no-infrastructure starter.

## Regression coverage

```sh
cargo test --manifest-path tools/clrinf-codegen/Cargo.toml
```

Native tests cover exact four-language golden output, deterministic generation
from an unrelated working directory with explicit assets, malformed/unsupported
schemas, duplicate identities, missing inputs and symlink/traversal rejection.
The shared conformance fixtures are also validated against the canonical core
JSON Schemas. `tests/generated` provides target-language compilation and wire
round-trip coverage; generated models are intentionally not full validators.
