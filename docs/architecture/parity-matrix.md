# Cross-Language Capability Parity Matrix

This matrix distinguishes shared wire-format contracts from runtime feature parity.
`In-memory` adapters are volatile across restarts; an `interface` or `behaviour` is a port definition, not a production adapter. All four languages are part of the monorepo, and no single language is required as a runtime dependency for the others.

| Capability | Rust | C# | TypeScript | Elixir |
|---|---|---|---|---|
| `RequestContext` / `ErrorEnvelope` / `CloudEvent` wire contracts | Core | Core | Core | Core |
| Shared valid/invalid JSON conformance fixtures | Native test | xUnit | Bun | ExUnit |
| Framework-free / in-process usage | Core example | Dispatcher / example | Core example | `CLRINF_MODE=core` |
| HTTP adapter / example | Axum | ASP.NET Core | Bun | Plug |
| Idempotency / outbox port | Yes | Yes | Yes | Behaviour |
| In-memory reference adapter | Yes (volatile) | Persistent adapter preferred | Yes (volatile) | Yes (volatile) |
| Local atomic business data + inbox + outbox | Production adapter needed | SQLite reference | Production adapter needed | Production adapter needed |
| Durable leased outbox / restart recovery | No reference | SQLite | No reference | No reference |
| Live NATS event inspector / streaming bridge | No | No | Inspector | Streaming bridge |
| Compile-time AOT CRUD generator & Smart Override eject | `clrinf-macros` (`#[derive(ClrinfCrud)]`) + `clrinfrs eject/crud-map` | `ClrinfCS.Generators` + `EfCrudStore` | No | No |
| Zero-dependency CLI static AST/import rules (`clrinf verify`) | Yes (`ast.rs`) | Yes (`ast.rs`) | Yes (`ast.rs`) | Yes (`ast.rs`) |
| Language-toolchain architectural AST linter (`clrinf lint`) | Syn (`RUST_ARCH001`) | Roslyn (`ARCH001–ARCH005`) | TS Compiler (`ARCH_TS_001`) | Elixir AST (`ARCH_EX_001–004`) |
| Generic durable Saga orchestrator | No | No | No | No |
| Automatic OpenTelemetry export across all services | Not guaranteed | Not guaranteed | Not guaranteed | Not guaranteed |

## Support Boundaries

This split is intentional: rather than copying superficial database adapters across four runtimes, `clrinf` standardizes canonical wire formats, explicit ports, and a concrete durable C# SQLite/EF Core reference. When production persistence is needed in another language or database, implement its adapter against the same lease, idempotency, and transaction acceptance criteria.

Core fixture tests cover only the sampled JSON wire shapes. JWT verification, broker redelivery, Cedar tenant authorization, high-load backpressure, disk failure, and distributed tracing remain application-level verification responsibilities.

## Verification Entry Points

Root CI runs native test suites per language plus `tests/conformance/fixtures`. Contract generation runs `tests/generated/regenerate.sh` and compiles/exercises all four language outputs via `tests/generated/check.sh`. `docker-compose.test.yml` exercises the live Elixir → NATS → TypeScript Inspector path.
