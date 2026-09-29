# clrinfjs

Reusable TypeScript infrastructure for monoliths and distributed services, plus an optional development Event Inspector. No commerce domain is required. This checkout targets **Bun 1.3.14** and strict TypeScript; exported entry points are TypeScript source, not prebuilt Node.js bundles.

## Boundaries

| Entry point | Responsibility | Import side effects |
| --- | --- | --- |
| `@clrinf/core` or `/core` | JSON context, errors, CloudEvents, functional Result, rule engine (`pipeRules`), idempotency/outbox ports, outbox dispatcher | None |
| `@clrinf/core/adapters` | Bounded, process-local memory adapters | None |
| `@clrinf/core/inspector` | Inspector engine and explicit `startInspector()` | None until called |
| `@clrinf/core/linter` | Deterministic TypeScript AST architectural linter (`ARCH_TS_001`, `ARCH_TS_003`) | None |
| `@clrinf/core/generator` | AI rule scaffolder (isolated rule + test generator) | None |
| `@clrinf/core/mcp` | Model Context Protocol (MCP) JSON-RPC 2.0 stdio server | None until started |
| `bun src/server.ts` | Opt-in HTTP/WebSocket Inspector and optional NATS subscriber | Starts configured listeners/connections |

The package is named `@clrinf/core`. The Inspector does not belong on an application's import path: use `@clrinf/core` (or `/core`) and `/adapters` directly.

## Start

```sh
bun install --frozen-lockfile
bun run typecheck
# Run deterministic architectural linter:
bun run lint:arch
# Start Model Context Protocol (MCP) server:
bun run mcp
# Run test suite:
bun test
# Standalone submodule checkout: first obtain the parent repository's canonical fixtures.
CLRINF_CONFORMANCE_DIR=/absolute/path/to/clrinf/tests/conformance/fixtures bun test
# Optional local broker test (runs in submodule CI using a real NATS service):
CLRINF_NATS_TEST_URL=nats://127.0.0.1:4222 bun test tests/nats.integration.test.ts
bun examples/monolith.ts
bun examples/http-service.ts
# Separate, optional developer tool:
bun start
```

Missing shared fixtures cause an explicit failure, never a skipped test or a locally duplicated contract. The submodule CI checks out the canonical fixtures from `brkmustu/clrinf` master; parent-repository CI supplies the fixtures from its own revision.

## Core API & Functional Rules Engine

`parseContext(unknown)`, `parseError(unknown)`, and `parseEvent(unknown)` validate at runtime and return detached, canonical objects; malformed values throw `ValidationError`. Unknown top-level fields are dropped. Event data and error details must be JSON objects; nested JSON arrays/scalars are supported, non-finite numbers and non-JSON values are rejected, and nesting is limited to 64 levels.

Context uses `tenant_id`, `correlation_id`, and `causation_id`. Error envelopes use `error_code`, `message`, `correlation_id`, `tenant_id`, `retryable`, and optional `details`. Events use CloudEvents `specversion: "1.0"` and `datacontenttype: "application/json"` with `id`, `source`, `type`, RFC3339 `time`, `tenantid`, `correlationid`, `causationid`, and object `data`. Optional `subject`, `is_error`, and `is_compensation` are preserved. Flags must be booleans and are independent: event names never imply failure or compensation.

### Functional Business Rules (`pipeRules` & `Result`)

To maintain clean architectural boundaries and eliminate raw `throw` statements in services:
- **`Result<T, E>`**: Monadic result container (`Ok<T>` / `Err<E>`).
- **`pipeRules(...rules)`**: Sequential composition of pure rule functions short-circuiting on first violation.
- **`violationToErrorEnvelope`**: Direct mapping to the cross-language canonical `ErrorEnvelope`.

```ts
import { pipeRules, rulePassed, ruleFailed, violationToErrorEnvelope, type Rule } from "@clrinf/core";

interface Order {
  id: string;
  total: number;
}

const checkPositiveTotal: Rule<Order> = (order) => {
  if (order.total <= 0) {
    return ruleFailed("INVALID_TOTAL", "Order total must be positive");
  }
  return rulePassed();
};

const validateOrder = pipeRules(checkPositiveTotal);

const result = await validateOrder({ id: "ord-1", total: -10 });
if (result.isErr) {
  const errorEnvelope = violationToErrorEnvelope(result.error, context);
  // Return typed canonical error
}
```

### TypeScript AST Architectural Linter (`bun run lint:arch`)

To maintain clean architectural discipline across the TypeScript codebase, `@clrinf/core/linter` parses source trees using the TypeScript Compiler API:
- **`ARCH_TS_001` (Forbidden Raw Throw in Business Rules)**: Flags raw `throw` statements in rule and service files. All validation logic must return monadic `Result<T, RuleViolation>` and map cleanly to `ErrorEnvelope`.
- **`ARCH_TS_003` (Prohibited Class-Based Mediator/Dispatcher)**: Forbids heavyweight MediatR-style class ports in TypeScript. Encourages direct functional composition (`pipeRules`) and idiomatic modules over reflection-light emulation.

Can be run standalone with `bun run lint:arch` or via the multi-language orchestrator `clrinf-codegen lint --lang typescript`.

```ts
import { createEvent, contextFromEvent, parseContext } from "@clrinf/core";

const context = parseContext({
  tenant_id: "tenant-example",
  correlation_id: "workflow-example",
  causation_id: "request-example",
});
const event = createEvent(context, {
  source: "clrinf/documents",
  type: "com.clrinf.documents.DocumentApproved.v1",
  data: { document_id: "document-example" },
});
const downstreamContext = contextFromEvent(event); // causation becomes event.id
```

These fields carry metadata, not authentication. Resolve and authorize tenant identity before constructing the context at a real ingress boundary.

## Reliability ports and memory limits

`IdempotencyStore.claim({tenant_id, operation, key})` atomically returns `acquired` with an ownership token, `busy`, or `completed` with cached JSON. `complete` and `release` require the current pending token. Keys are scoped by tenant and operation; callers must never reuse a key for a different logical input. The memory adapter defaults to 1,000 entries and five-minute retention **after completion**. Active claims never expire automatically; release failed work explicitly. A hung worker requires operator/application recovery, not an unsafe timed takeover.

`OutboxStore.enqueue(event)` rejects duplicate pending tenant/event IDs. `claim(limit, leaseMs)` returns ownership tokens and delivery attempt counts; `acknowledge` deletes only a currently owned, unexpired delivery. `retry` releases it with a nonnegative delay. Expired leases are redelivered; stale acknowledgements fail. The memory queue defaults to 1,000 entries and rejects excess work with `CapacityError`, rather than silently dropping pending events. The adapters bound entry counts, not payload bytes; applications must enforce ingress size limits.

`dispatchOutbox(store, publisher, options)` publishes sequentially and acknowledges successful deliveries. Publishing errors are surfaced and the failed item is scheduled for retry; other claimed items become available after their leases expire. Choose a lease comfortably longer than the entire batch duration, or a batch size of one for slow publishers. No lease renewal is provided. Delivery is **at least once**, not exactly once: a crash after publish and before acknowledgement can duplicate effects. Consumers must deduplicate.

**Memory adapters are not durable.** Restart loses claims, cached results, and pending events. They coordinate only within a single instance, do not share an application database transaction, and provide neither cross-process locking nor crash-safe business-state/inbox/outbox atomicity. For production, implement the ports over the application's database and coordinate business changes, inbox state, and outbox insertion in one transaction. The parent repository's C# SQLite reference demonstrates that boundary; these TypeScript adapters do not implement it.

## Examples and brownfield adoption

`examples/monolith.ts` demonstrates explicit dependencies and document approval without HTTP or NATS. `examples/http-service.ts` wraps the same workflow at `POST http://127.0.0.1:4300/documents/approve`:

```sh
curl http://127.0.0.1:4300/documents/approve \
  -H 'Content-Type: application/json' \
  -H 'X-Tenant-Id: tenant-example' \
  -H 'X-Correlation-Id: workflow-example' \
  -H 'X-Causation-Id: request-example' \
  -H 'Idempotency-Key: approve-document-1' \
  -d '{"document_id":"document-1"}'
```

The response is an accepted canonical event. The example deliberately leaves its bounded outbox pending for an explicitly configured dispatcher; it does not silently publish to a broker. Repeated keys replay the cached event. It is loopback-only, trusts example headers, and has **no authentication or durable storage**. See [brownfield adoption](docs/brownfield.md) before adapting it.

The optional old commerce flow now lives in `examples/showcases/commerce.ts`; it is never loaded by the core or default Inspector. The default Inspector simulation emits document workflow events.

## Inspector

Defaults: `HOST=127.0.0.1`, `PORT=4200`, standalone mode when `NATS_URL` is absent. Configuring `NATS_URL` requires a successful connection and subscription flush before HTTP starts; unavailable NATS fails startup instead of masquerading as a connected subscriber. This uses **NATS Core**, not a durable JetStream consumer: offline messages are not recovered.

`GET /health` reports `nats: disabled|connected|disconnected`; configured disconnection returns HTTP 503. `GET /api/events` supports `correlation_id` and `tenant_id` filters. `POST /api/events` validates canonical ingress; `DELETE /api/events` clears history. `/ws` emits `{type:"init",payload:[...]}`, `{type:"event",payload:event}`, and `{type:"clear"}`. `GET /api/waterfall` estimates spans from event timestamps; it is not OpenTelemetry tracing. Tenant/correlation pairs are grouped separately.

`POST /api/replay` accepts `{event_id, tenant_id?, mutate_correlation?}` or `{event, mutate_correlation?}`. Replay creates a fresh ID/time, retains tenant, and links causation to the original event. Ambiguous IDs across tenants require `tenant_id`. Replay is **Inspector-only**; `publish:true` is rejected, even when NATS is connected. `POST /api/simulate` records a neutral four-event document workflow without business-bus publishing.

Server history is bounded to 500 events and 1 MiB of serialized event data; each recorded event is limited to 256 KiB, as are HTTP request bodies and NATS ingress. Old observed events are evicted. The UI keeps at most 500 events; WebSocket clients are limited to 32, with 2 MiB backpressure limits and slow-client disconnection. On reconnect, clients receive current history rather than guaranteed delivery. Simulation creates no delayed, unbounded timer queue.

**Development tool, not a production service or tenant security boundary.** No tenant authorization, audit persistence, TLS termination, or data redaction is provided. Set `INSPECTOR_TOKEN` for shared development access (Bearer token or HTTP Basic user `inspector`, password equal to the token). Browser UI uses the Basic challenge; proxies must forward authenticated WebSocket upgrades. Put it behind TLS and authenticated ingress. `/health` is intentionally unauthenticated. Non-loopback binding without a token is rejected unless `INSPECTOR_ALLOW_UNAUTHENTICATED=true` explicitly acknowledges development-only exposure.

`docker compose --profile inspector up --build` exposes local development ports only. The Nomad example requires a token from `nomad/jobs/clrinfjs-inspector`; it is not a production-hardening recipe.

## Coverage and limits

Bun tests exercise shared conformance, malformed JSON/timestamps/flags, ownership and concurrent claims, tenant separation, expiry/retry/acknowledgement-loss behavior, capacity rejection, HTTP validation/authentication, live WebSockets, local replay, import safety, and the reusable examples. `CLRINF_NATS_TEST_URL` enables an additional real-broker outbox -> NATS -> Inspector test, including rejection of invalid envelopes and proof that replay does not publish; it is explicitly skipped without that variable and runs in submodule CI. The parent e2e suite owns the real Elixir -> NATS -> Inspector path. Browser layout, Nomad rollout, multi-process durability, and production security are not asserted by this package's tests.
