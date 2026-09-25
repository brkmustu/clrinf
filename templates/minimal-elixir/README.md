# Minimal Elixir

Standalone, standard-library-only application service / modular monolith
starter. Requires Elixir 1.14+ with a compatible Erlang/OTP runtime and Mix.
There are no Hex dependencies; no dependency download is needed.

From this directory:

```sh
mix run -e 'MinimalService.Demo.run()'
mix test
```

The demo prints `Hello, World! [demo-request]`.

`lib/core.ex` defines an application module behaviour, request context,
application error, result tuples, and direct dispatch. `lib/greeting.ex` owns
the request/response structs and blank-name validation. `lib/demo.ex` is the
composition root. `test/greeting_test.exs` exercises dispatch, context
propagation, and validation using standard-library ExUnit.

Add modules implementing the behaviour and pass them to the dispatcher.
Calls are synchronous in the caller's process, not messages to a GenServer.
The boundary accepts application-owned structs; malformed external input needs
validation in a transport adapter.

These are local example types, not generated wire contracts or an SDK dependency.
The context ID is caller-supplied tracing metadata, not authentication or an
idempotency key. This console host does not listen on a network port. There is
no persistence, retry, transaction, outbox, delivery, or exactly-once guarantee.
Add explicit adapters and their tests only when your application needs them.
