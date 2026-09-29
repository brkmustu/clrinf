# Current implementation context

The package name is `@clrinf/core`. Its default and `/core` exports are side-effect-free reusable contracts, ports, functional Result type (`Result<T, E>`), and functional business rules engine (`pipeRules`, `Rule<TCtx>`, `RuleViolation`). `/adapters` contains process-local memory implementations. `/inspector` exports an explicit startup function and isolated engine; `src/server.ts` starts only when executed as main.

Contracts: `Context`, `ErrorEnvelope`, `CloudEventEnvelope`, `parseContext`, `parseError`, `parseEvent`, `createEvent`, `createError`, and `contextFromEvent`. The validator rejects non-JSON data, bad RFC3339 times, and non-boolean flags. Shared fixture files are read from the parent or `CLRINF_CONFORMANCE_DIR`; never duplicate them into this submodule to hide a missing contract dependency.

Functional Rule Engine:
- `Result<T, E>` (`Ok<T>`, `Err<E>`) provides zero-dependency monad error handling.
- `pipeRules(...rules)` evaluates sequential business rules short-circuiting on first failure without throwing raw exceptions.
- `Rule<TCtx>` is a pure function: `(context: TCtx, requestContext?: Context) => Result<void, RuleViolation> | Promise<Result<void, RuleViolation>>`.
- Anti-pattern: Never use raw `throw` in business rules or services. Always return `Result<void, RuleViolation>` or canonical `ErrorEnvelope`.
- Anti-pattern: Do not introduce C#-style reflection or class-based Mediator/Dispatcher to TypeScript. Keep rules and pipelines as pure functional compositions.

Reliability: token-owned idempotency claims; tenant/operation/request scoping; completed-result TTL; no automatic expiry of active claims. Bounded memory outbox with expiring ownership leases, attempts, explicit retry/acknowledge, and a sequential at-least-once dispatcher. No transactional database adapter, durable inbox, automatic lease renewal, dead-letter policy, or distributed-lock guarantee is implemented here.

Inspector: optional NATS Core observer, bounded HTTP/WebSocket ingestion, local-only replay, document simulation, explicit failure/compensation flags, tenant-separated waterfall groups, optional shared token authentication, loopback default, explicit unsafe development override. NATS availability is mandatory when configured, and readiness exposes connection status. This is not a production auth or observability platform.

Architectural Linter (`/linter` & `bun run lint:arch`):
- Deterministic AST analysis using TypeScript compiler API (`ts.createSourceFile`).
- `ARCH_TS_001`: Forbids raw `throw` inside `*.rules.ts` or `*.service.ts` files; requires `Result<void, RuleViolation>`.
- `ARCH_TS_003`: Forbids class-based `Dispatcher`, `Mediator`, or `RulePipeline`.

AI Rule Scaffolding (`/generator`):
- AI agents must create rules as isolated, zero-dependency pure functions without modifying existing domain files.
- Generates `<name>.rule.ts` with typed `Rule<TCtx>` and companion `<name>.rule.test.ts`.

MCP Server (`/mcp` & `bun run mcp`):
- Standard JSON-RPC 2.0 stdio MCP server for agentic IDEs and autonomous pair programmers.
- Exposes tools: `clrinf_inspect_architecture`, `clrinf_lint_rules`, `clrinf_scaffold_rule`, `clrinf_conformance_check`.

Examples: reusable document approval in a monolith and loopback HTTP adapter. Reference e-commerce cart business rules in `examples/rules/cart.rules.ts`. Obsolete `cli/` directory has been removed; polyglot code generation is owned by the canonical parent generators.
