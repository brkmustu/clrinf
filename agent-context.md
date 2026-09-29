# Agent context: clrinf

clrinf is a domain-independent contract-first foundation for distributed and
modular-monolith applications in Rust, C#, TypeScript and Elixir.

## Ownership

- Core language implementations live in this monorepo under `core/rust`, `core/csharp`, `core/typescript`, and `core/elixir`.
- Core wire formats belong to each language implementation under `core/<lang>/contracts/schemas` and `tools/clrinf-codegen/schemas`.
- Domain schemas belong to examples/contracts; commerce design notes belong to
  templates/eticaret-showcase/docs.
- Root CLI handles contract generation/catalog; C# retains specialized generation.
  The incomplete historical TypeScript CLI is not advertised as a working command.

## Invariants

Read each repository's contracts/constitution.md and VERSIONING.md. Preserve event names and
schema IDs during moves. HTTP context keys use tenant_id/correlation_id/causation_id;
CloudEvents use tenantid/correlationid/causationid.

Local ACID is allowed. Broker, database, authorization engine and deployment
platform are adapter choices. Do not use correlation_id alone as a dedupe key.
Do not describe volatile memory, Core NATS, or example sagas as durable delivery.

## Validation

Use existing Rust, xUnit, Bun and ExUnit runners. Shared fixtures live in
tests/conformance/fixtures. A missing fixture is an error, not a passing test.
Root CI covers language suites, generated code, docs links and generic E2E.

## Working tree

Do not commit, stage, push, change branches or discard work without user
authorization. Run appropriate language test suites and schema validations
before committing changes across `core/` and `tools/`. Never report working-tree
changes as a released cross-repository version.
