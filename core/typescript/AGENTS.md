# clrinfjs agent guidance

Scope changes to this submodule. Preserve submodule boundaries and user-owned git state.

Use Bun 1.3.14, `bun install --frozen-lockfile`, `bun run typecheck`, and `bun test`. Canonical fixtures belong to the parent repository under `tests/conformance/fixtures`; set `CLRINF_CONFORMANCE_DIR` for a standalone checkout. Missing fixtures must fail explicitly.

Keep `src/core` independent of Bun servers, NATS, Inspector, and domain examples. Validate unknown ingress at runtime. Preserve canonical snake_case context/error fields and CloudEvents extension spelling. Error and compensation flags are explicit and independent.

Rules and Error Handling:
- Business rules must be implemented as functional `Rule<T>` returning `Result<void, RuleViolation>`.
- Chain rules using `pipeRules(...rules)`.
- Anti-pattern: NEVER throw raw exceptions in business rules or services. Always propagate `Result`.
- Anti-pattern: Do NOT port C# class-based Mediator/Dispatcher or reflection DI to TypeScript. Keep rules and pipelines as pure functional compositions.

Memory adapters must be bounded and described as non-durable. Do not claim atomic database writes, cross-process deduplication, or exactly-once delivery. Inspector replay must remain local by default; no business-bus publish is currently supported.

The root TypeScript check covers core, adapters, Inspector, tests, and examples. Obsolete `cli/` scaffolding has been removed; polyglot code generation is managed by root/Rust generators.

See [agent-context.md](agent-context.md) and [README.md](README.md) for current boundaries.
