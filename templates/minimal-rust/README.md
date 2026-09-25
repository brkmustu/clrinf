# Minimal Rust

Standalone, standard-library-only application service / modular monolith starter.
Requires Rust and Cargo (edition 2021 support); no package download is needed.

From this directory:

```sh
cargo run --offline
cargo test --offline
```

The demo prints `Hello, World! [demo-request]`.

`src/core.rs` defines a generic `ApplicationModule<Request>` boundary, a
`RequestContext`, an application error, and direct in-process dispatch.
`src/greeting.rs` owns the `Greet` request, `Greeting` response, validation,
and tests. `src/main.rs` is the composition root: it chooses the module and
supplies context. Add modules behind this boundary without introducing a broker.

These are local example types, not generated wire contracts or an SDK dependency.
The context ID is caller-supplied tracing metadata, not authentication or an
idempotency key. This console host does not listen on a network port. There is
no persistence, retry, transaction, outbox, delivery, or exactly-once guarantee.
Add explicit adapters and their tests only when your application needs them.
