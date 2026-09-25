# Minimal C#

Standalone .NET 10 standard-library-only application service / modular monolith
starter. Requires the .NET 10 SDK and its local targeting packs.

From this directory:

```sh
dotnet run
dotnet run -- --self-test
```

The demo prints `Hello, World! [demo-request]`. The self-test command checks
successful dispatch, context propagation, and null/blank input rejection,
and exits nonzero on failure. No external test framework is required.
`NuGet.Config` clears remote package sources; there are no package references.

`Core.cs` defines `IApplicationModule<TRequest, TResponse>`, direct dispatch,
context, and success/failure results. `GreetingModule.cs` owns its request,
response, and validation. `Program.cs` composes the module and supplies context.
Add application modules behind the interface while keeping calls in-process.

These are local example types, not generated wire contracts or an SDK dependency.
The context ID is caller-supplied tracing metadata, not authentication or an
idempotency key. This console host does not listen on a network port. There is
no persistence, retry, transaction, outbox, delivery, or exactly-once guarantee.
Add explicit adapters and their tests only when your application needs them.
