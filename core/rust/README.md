# clrinf Rust Infrastructure

Generic, high-reliability infrastructure for distributed services **and** modular monoliths built with Rust.

---

## Workspace Packages and Structure

- **`clrinf-core`**: Domain-neutral foundational abstractions. Has zero web framework (Axum) or database dependencies. Exposes `RequestContext`, `ErrorEnvelope`, `CloudEventEnvelope`, CQRS traits, and the `BusinessRule<T>` engine.
- **`clrinf-adapters`**: Axum HTTP middleware, contextual error normalization, and `MemoryWorkflowStore`.
- **`clrinf-cli`**: Language worker CLI, Syn AST architectural linter (`RUST_ARCH001`), AI business rule scaffolder, and MCP server.
- **`clrinf-generic`**: Example entry points for both modular monolith (`bin/monolith.rs`) and HTTP service (`bin/http-service.rs`).
- **`examples/`**: Dedicated isolated business services and showcases:
  - `examples/clrinf-auth`: Standalone JWT authentication service.
  - `examples/clrinf-kampanya`: Campaign & discount calculation domain service.

```sh
cargo run -p clrinf-generic --bin monolith
cargo run -p clrinf-generic --bin http-service
curl -X POST http://localhost:8080/messages/request-1 \
  -H 'content-type: application/json' -H 'x-tenant-id: tenant-example' \
  -H 'x-correlation-id: workflow-1' -H 'x-causation-id: request-1' \
  -d '{"text":"hello"}'
```

Both deployment modes invoke identical application logic and persistence ports. The HTTP example demonstrates a trusted perimeter boundary; deploy an authentication adapter before exposing endpoints to public networks.

---

## Business Rules Engine & Syn Architectural Linter

To maintain strict domain boundaries without relying on unstructured runtime exceptions or `panic!`, `clrinf-core` provides an idiomatic, strongly-typed business rule engine:

### 1. Defining and Evaluating Rules
```rust
use clrinf_core::rules::{BusinessRule, RulePipeline, RuleResult};
use clrinf_core::RequestContext;
use async_trait::async_trait;

pub struct Order {
    pub total: f64,
}

pub struct CheckMinimumOrderAmount;

#[async_trait]
impl BusinessRule<Order> for CheckMinimumOrderAmount {
    fn priority(&self) -> i32 { 1 }

    async fn evaluate(&self, order: &Order, _ctx: &RequestContext) -> RuleResult {
        if order.total < 50.0 {
            RuleResult::failed("MIN_AMOUNT_NOT_MET", "Order total must be at least 50.00")
        } else {
            RuleResult::success()
        }
    }
}

// In your application handler:
let pipeline = RulePipeline::new().add_rule(CheckMinimumOrderAmount);
match pipeline.evaluate(&order, &request_context).await {
    Ok(()) => { /* Proceed with execution */ },
    Err(violation) => {
        let error_envelope = violation.to_error_envelope(&request_context)?;
        // Return typed canonical error
    }
}
```

### 2. Static Architectural Linter (`RUST_ARCH001`)
The `clrinf-cli` tool analyzes Rust syntax trees via `syn`:
- Flags prohibited `panic!`, `unwrap()`, or `expect()` in domain handlers and rule files.
- Enforces deterministic `RuleResult` returns and explicit context propagation.
- Can be run standalone or invoked via `clrinf-codegen lint --lang rust`.

---

## Contracts, Propagation, and Errors

- **Context Invariants**: `RequestContext::new` validates tenant, correlation, and causation IDs (1-128 ASCII alphanumeric plus `.` `_` `:` `-`). Transport headers identify requested tenants, not authenticated membership proof.
- **Error Normalization**: Errors serialize canonical wire fields: `error_code`, `message`, `correlation_id`, `tenant_id`, `retryable`, and optional `details`. Untrusted headers are never echoed.
- **CloudEvents 1.0**: Complies strictly with CloudEvents specifications (`specversion: "1.0"`). Optional boolean flags `is_error` and `is_compensation` have no heuristic inference.

---

## Persistence and Reliability Ports

- **Ports**: `IdempotencyStore`, `OutboxStore`, and `WorkflowStore` are `Send + Sync` asynchronous abstractions.
- **Atomic Claiming**: `claim` uses tenant + operation + idempotency key with a lease duration and token fencing to prevent stale-worker takeovers.
- **Atomic Commit**: `commit` stores the replay response and outbox events atomically in a single persistence boundary.
- **Memory Adapter Limits**: Process-local, unbounded, and non-durable. Production environments should enlist business writes and outbox insertions into the application's transactional database.

---

## Development & Testing

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
docker build -t clrinf-generic .
docker build -f examples/clrinf-kampanya/Dockerfile -t clrinf-kampanya .
docker build -f examples/clrinf-auth/Dockerfile -t clrinf-auth .
```
