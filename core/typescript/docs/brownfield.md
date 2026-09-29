# Brownfield adoption

Keep existing domain services, ORM, transaction boundaries, routes, and broker. `clrinfjs` is infrastructure, not a mandate to split a monolith or adopt a specific domain model.

1. Add the `/core` entry point at one ingress boundary. Validate incoming JSON with `parseEvent`, and resolve tenant identity using your existing authentication layer before constructing `Context`. Do not trust an external tenant header merely because it passes schema validation.
2. Pass context explicitly into existing application functions. Preserve correlation across work; use `contextFromEvent` when handling an event so subsequent causation refers to that event's ID.
3. Translate existing errors into the canonical envelope at your transport boundary. Keep internal exceptions and stack traces internal. Set `retryable` deliberately; mark event failure/compensation using explicit flags, never by matching an event name.
4. Introduce `IdempotencyStore` and `OutboxStore` behind existing repository boundaries. Memory implementations are for tests, demos, or accepted-loss workloads only. Stable operation keys must identify the same logical request and payload throughout their retention window.
5. For database-backed work, implement atomic claim/uniqueness and write business state, durable inbox/idempotency state, and outbox rows in one transaction. These simple ports alone do not create that transaction: expose a database unit of work in your application adapter. Do not sequence independent writes and label them atomic.
6. Run an explicit dispatcher using the existing transport. Accept that publish/acknowledge is at least once; scope consumer deduplication by tenant and event ID. Design retention, poison-message quarantine, retries, lease renewal, cancellation, and backpressure for your workload. The reference memory dispatcher intentionally does not implement all of these policies.
7. Attach the Inspector only to approved development traffic. It observes NATS Core or HTTP and keeps bounded, volatile history. Local replay is a visualization tool, not a safe command retry or a workflow compensation engine.

For a monolith, direct function calls plus a database transaction may be all that is required. Add HTTP and brokers only at actual deployment boundaries. The same canonical contracts can cross both boundaries without requiring a service split.

The [examples](../examples) show the call shape but intentionally omit durable business storage and real authentication. They are not production templates.
