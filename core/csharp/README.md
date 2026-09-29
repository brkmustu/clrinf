# clrinf C# infrastructure and specialized code generator

Reusable **.NET 10** infrastructure for modular monoliths and distributed
services. Business domains are examples, not framework defaults. The existing
Clean Architecture/CQRS CLI remains available as a specialized scaffolder;
adopting the runtime does not require generation, CQRS, EF Core, Angular or
a service split.

## Runtime layout

| Project | Purpose |
| --- | --- |
| `src/ClrinfCS.Core` | Context, canonical errors/CloudEvents, explicit dispatcher, tenant storage and inbox/outbox ports; no database or web dependencies |
| `src/ClrinfCS.Adapters.Sqlite` | Durable local reference adapter using Microsoft.Data.Sqlite |
| `examples/Documents` | Shared example module used unchanged by both deployment modes |
| `examples/ModularMonolith` | In-process module delivery using the same event contracts |
| `examples/DistributedService` | HTTP producer/receiver with separate databases |
| `examples/CrmMonolith` | Containerized Modular Monolith reference (`Modules/` container, EF Core SQLite, Minimal APIs) |
| `examples/EcommerceMonolith` | Direct Bounded Contexts reference (Top-level slices, pure FP, Native Dispatcher, Cedar authz) |
| `src/ClrinfCS` / `src/ClrinfCS.OpenApi` | Roslyn AST worker CLI tool (lint, migrate, mcp) and native OpenAPI support |

Add project references to the core and the adapter you need, or package them for
your own NuGet feed. No hosted service, broker or dependency injection container
is required by the core.

```bash
dotnet add YourApplication.csproj reference /path/to/core/csharp/src/ClrinfCS.Core/ClrinfCS.Core.csproj
dotnet add YourApplication.csproj reference /path/to/core/csharp/src/ClrinfCS.Adapters.Sqlite/ClrinfCS.Adapters.Sqlite.csproj
```

### Contracts and dispatch

`RequestContext` requires `tenant_id`, `correlation_id`, `causation_id`.
`ErrorEnvelope` uses the canonical `error_code`, `message`, `correlation_id`,
`tenant_id`, `retryable` and optional object `details`.
`CloudEvent` uses CloudEvents 1.0 names (`tenantid`, `correlationid`,
`causationid`), object `data`, an explicit timestamp and JSON content type.
Optional `is_error` and `is_compensation` flags preserve `false` on the wire.
Always use `ContractJson.Deserialize<T>` at ingress: required constructor
parameters, primitive types and contract invariants are checked. This validates
the common envelope, not arbitrary domain payload schemas. Payloads are cloned
so their lifetime does not depend on a caller's `JsonDocument`.

Create contexts from **verified authentication**, not a tenant header or
untrusted event payload. `CloudEvent.ToContext()` retains tenant/correlation and
makes the consumed event ID the next operation's causation ID. Verify the event
tenant against the authenticated transport identity before calling it.

```csharp
var context = new RequestContext(authenticatedTenant, correlationId, requestId);
var dispatcher = new Dispatcher.Builder()
    .Register(new ApproveDocumentHandler(store))
    .Build();
await dispatcher.SendAsync(new ApproveDocument("document-42"), context, cancellationToken);
```

Handlers implement `IRequestHandler<TRequest,TResponse>` and receive explicit
context/cancellation. `Register(handler, behaviors...)` executes behaviors in
registration order, outermost first. Duplicate/missing routes fail explicitly.
The built route table is immutable; construct it per DI scope when handlers
depend on scoped services. Handlers and behaviors must be safe for their chosen
lifetime. The SDK makes no universal latency, allocation or throughput claims;
the historical benchmark projects measure only their particular workloads.

### Business Rules Engine & Roslyn Architectural Analyzers

To decouple business validation from core domain logic without leaking framework dependencies or throwing unstructured exceptions, `ClrinfCS.Core` provides an idiomatic business rule engine and compile-time Roslyn static analyzers:

#### 1. Defining and Registering Business Rules
Rules implement `IBusinessRule<TContext>` and execute inside `BusinessRulePipelineBehavior`:

```csharp
using ClrinfCS.Core;

public record PlaceOrderCommand(string OrderId, decimal Amount) : IRequest<OrderPlacedResult>;

public class MinimumOrderAmountRule : IBusinessRule<PlaceOrderCommand>
{
    public int Priority => 1;

    public ValueTask<RuleResult> EvaluateAsync(
        PlaceOrderCommand command,
        RequestContext context,
        CancellationToken cancellationToken = default)
    {
        if (command.Amount < 50m)
        {
            return ValueTask.FromResult(RuleResult.Failed(
                "MIN_AMOUNT_NOT_MET",
                "Order amount must be at least $50.00."));
        }

        return ValueTask.FromResult(RuleResult.Success());
    }
}
```

Registered rules evaluate sequentially in priority order. On the first failure, execution halts immediately (short-circuit) and throws `BusinessRuleViolationException`, which converts cleanly to the canonical `ErrorEnvelope` via `ToErrorEnvelope(context)`.

#### 2. Static Architectural Analyzers
The `clrinfcs lint` analyzer inspects C# source trees using the Roslyn compiler platform:
- **`ARCH001` (Domain Layer Purity)**: Forbids direct dependencies on Entity Framework Core, ASP.NET Core, or Infrastructure inside the `Domain` layer.
- **`ARCH002` (Rule Convention)**: Mandates that rule classes in `Rules/` implement `IBusinessRule<TContext>`.
- **`ARCH003` (Controller DbContext Leakage)**: Prohibits injecting `DbContext` directly into API controllers or host endpoints; mandates `IDispatcher` or application services.
- **`ARCH004` (CQRS Request Contract)**: Ensures CQRS Command/Query classes implement `IRequest<TResponse>`.

Run standalone with `clrinfcs lint` or across the multi-language workspace using `clrinf-codegen lint --lang csharp`.

### Durable local transactions

```csharp
var store = new SqliteStore("application.db");
store.Execute(context, tx =>
{
    if (!tx.TryAccept("projection", incoming))
        return false; // Already applied by this consumer in this tenant.
    tx.Set("documents/42", "approved");
    tx.Enqueue(CloudEvent.Create(context, "clrinf/documents",
        "com.clrinf.documents.DocumentApproved.v1",
        JsonSerializer.SerializeToElement(new { document_id = "42" })));
    return true;
});
```

`Execute` uses one SQLite connection and one immediate transaction for the
tenant-scoped state, inbox identity and outbox enqueue. Exceptions and
cancellation before commit roll all three back. The callback is deliberately
synchronous: **do not pass async callbacks**, retain the transaction object,
nest transactions, or perform network I/O inside it. SQLite provider commands
are synchronous; cancellation is checked before starting and before commit,
not guaranteed to interrupt a database lock wait. The reference has a 30-second
lock timeout and WAL mode; use local durable disk, not network-mounted SQLite.
It is a single-file reference, not a high-availability cluster adapter.

All state keys include the trusted tenant. Inbox keys include tenant, consumer,
source and event ID. `TryAccept(consumer, envelope)` records the complete known
envelope content and compares retries using structural JSON equality, ignoring
object property order and normalizing timestamp offsets to UTC. Identical
content returns `false`; changed payload or metadata raises
`InboxConflictException`, rolling back the transaction if allowed to propagate.
The Documents projection uses this overload; its HTTP demo returns a canonical
409 `IDEMPOTENCY_CONFLICT` error on conflicts.

The backward-compatible `TryAccept(consumer, source, eventId)` checks **identity
only** and requires immutable event IDs/content to be guaranteed by the caller.
It cannot detect conflicting payloads. Existing adapters default to explicitly
rejecting the content-aware overload until they implement it. SQLite adds a
nullable content column to existing databases under a write transaction.
Existing identity-only entries remain intact; using the content-aware overload
against one raises a conflict because the original content cannot be verified.
Do not delete or silently backfill these markers from untrusted retries.

Outbox identity includes tenant, source and event ID;
duplicate enqueue raises an error and rolls back rather than silently replacing
an event. The adapter's string key/value state is a reference business store.
It does **not** share transactions with an unrelated EF Core database or an
external API. A production relational adapter must enlist business changes and
outbox records in that database's same transaction.

Publish after committing: `Claim(context, limit, lease)` returns deliveries with
lease tokens and attempt counts. Send them using your transport, then
`Complete(context, delivery)`. On known failure use
`Abandon(context, delivery, retryDelay)`. A process crash leaves the message
eligible when its lease expires; retries/restarts retain its event ID.
Expired or replaced tokens cannot acknowledge or release a newer lease.
An empty claim means no *currently eligible* messages, not a globally empty queue.

Delivery is **at least once**, not globally exactly once: a crash after remote
acceptance but before local acknowledgement can redeliver. Apply the consumer's
inbox marker and local mutation in one transaction. External effects require
their own idempotency keys. Lease renewal, automatic worker scheduling, dead
letters, retention/cleanup, general schema migration/versioning, encryption and metrics
are intentionally left to the hosting application. Completed rows and inbox
identities are retained; deleting them changes the deduplication horizon.

## Run the same module in either deployment mode

From this directory:

```bash
dotnet run --project examples/ModularMonolith -- /tmp/clrinf-monolith.db

# Terminal 1: local demonstration receiver (not an authenticated public API).
dotnet run --project examples/DistributedService -- serve /tmp/clrinf-receiver.db http://127.0.0.1:5089
# Terminal 2: commits to a distinct producer database, then sends durable events.
dotnet run --project examples/DistributedService -- send /tmp/clrinf-producer.db http://127.0.0.1:5089
curl http://127.0.0.1:5089/documents/document-example
```

Both examples execute `examples/Documents/DocumentModule.cs` and use the same
core contracts and tenant scope. The monolith dispatches events in process;
the distributed version serializes them over HTTP. The demo fixes the trusted
tenant to `tenant-demo`, binds to the supplied URL and has no authentication:
use loopback only. Replace this boundary with verified identity before exposing
it. The sender processes a bounded batch once, not a production worker; run it
again after lease/backoff expiry to recover pending deliveries. Each invocation
also creates a new approval command. HTTP failure is surfaced; a network/process
failure leaves the lease to expire rather than pretending delivery succeeded.

## Reference Monoliths: Modular Layout Strategies

`clrinf` demonstrates two distinct, first-class modular architecture styles across its reference applications:

| Pattern | Example Project | Directory Layout | Namespace Style | Best Fit For |
| :--- | :--- | :--- | :--- | :--- |
| **Containerized Modules** | `examples/CrmMonolith` | `Modules/<Module>/` (`Deal`, `Contact`, `Activity`) | `CrmMonolith.Modules.<Module>` | Large enterprise platforms with 15+ feature modules where separating business domains from infrastructure (`Data/`, `Common/`, `Policies/`) in the root directory keeps the workspace organized. Configured via `[backend.structure] folder_name = "Modules"`. |
| **Direct Bounded Contexts + Flat Decomposition** | `examples/EcommerceMonolith` | `<Module>/` (`Catalog`, `Inventory`, `Orders`) directly at root | `EcommerceMonolith.<Module>` | Domain-driven systems with well-defined Bounded Contexts where domains are primary first-class citizens. Uses low-cognitive-load `flat` decomposition inside each module (`<Module>Objects.cs`, `<Module>Rules.cs`, `<Module>Handlers.cs`, `<Module>Module.cs`). |

## Brownfield adoption and generator compatibility

Start with one existing module: introduce `RequestContext` at its verified
ingress, map external failures with `ErrorEnvelope.From`, and adopt CloudEvents
only at its integration boundary. Keep existing controllers, domain types and
MediatR/native handlers. Introduce a tenant-filtered persistence adapter and local
outbox before moving publication out of process. Consumers first add inbox
deduplication, then can move behind HTTP or a broker without changing payloads.
Keep a monolith until independent deployment is useful; distributing a system
is not a prerequisite for using these libraries.

The generator copies canonical core source files into a referenced `Clrinf.Core`
project and provides an authenticated HTTP context factory. These sources are
packaged from the runtime, not maintained as duplicate contract templates.
Generated legacy CQRS signatures remain compatible. Its internal FP
`DomainError` is not a wire error; convert it at boundaries. Generated EF CRUD
repositories are **not automatically tenant safe** and do not provide atomic
outbox integration. Apply those changes explicitly during migration.

## Validation

```bash
dotnet test tests/ClrInfTests/ClrInfTests.csproj
dotnet build clrinf.slnx
# Standalone submodule checkout:
CLRINF_CONFORMANCE_DIR=/path/to/shared/fixtures dotnet test tests/ClrInfTests/ClrInfTests.csproj
```

Tests consume the shared root `tests/conformance/fixtures/{valid,invalid}.json`
and fail explicitly when absent. Coverage includes serialization, dispatcher
ordering/cancellation, generated dispatcher compilation/execution, OpenAPI
contract names, local atomic rollback, reopen/restart recovery, tenant/consumer
isolation, concurrent consumers/publishers and stale lease fencing.

---

---

## ClrinfCS CLI: Roslyn AST Architectural Worker

`clrinfcs`, .NET 10 (LTS) projeleri için statik kod analizi (Roslyn AST), mimari dönüştürme ve migrasyon yönetimi sağlayan uzmanlaşmış **Worker CLI** aracıdır.

> [!NOTE]
> Kod üretimi (`new`, `add entity`, `add module`, `generate-all`) ve şablon orkestrasyonu merkezi Rust aracı olan **[`clrinf-codegen`](../../tools/clrinf-codegen)**'e taşınmıştır. `clrinfcs` aracı bu yapıda Roslyn AST tabanlı statik analiz ve dönüştürme worker'ı olarak çalışır.

---

## Yetenekler & Komutlar

### 1. Roslyn AST Statik Mimari Analiz (`clrinfcs lint`)
C# SyntaxTree ve SemanticModel üzerinde Clean Architecture kurallarını denetler:
- `ARCH001` (Domain Layer Purity): Domain katmanında EF Core, ASP.NET veya Infrastructure referanslarını yasaklar.
- `ARCH002` (Rule Convention): `Rules/` altındaki sınıfların `IBusinessRule<TContext>` uygulamasını zorunlu kılar.
- `ARCH003` (Controller DbContext Leakage): Controller veya Minimal API endpoint'lerinde doğrudan DbContext enjeksiyonunu engeller.
- `ARCH004` (CQRS Request Contract): CQRS Komut/Sorgu sınıflarının `IRequest<TResponse>` uygulamasını doğrular.

```bash
clrinfcs lint [--strict] [--json]
```

### 2. Roslyn AST Mimari Dönüşüm Motoru (`clrinfcs migrate`)
Mevcut projenizi sıfırdan yazmadan başka bir mimari profile dönüştürür (`flat -> layered -> clean-cqrs`):
```bash
# Yapılacak işlemleri güvenle simüle edin (Dry-Run):
clrinfcs migrate --to layered --dry-run
clrinfcs migrate --to clean-cqrs --dry-run

# Gerçek dönüşümü çalıştırın:
clrinfcs migrate --to clean-cqrs
```

### 3. EF Core Migrasyon Yönetimi (`clrinfcs migration`)
Migrasyonları katman yapısına uygun izole dizinlerde (Persistence/Migrations) oluşturur ve uygular:
```bash
# Yeni migrasyon ekle
clrinfcs migration add AddProductsTable

# Veritabanına migrasyonları uygula
clrinfcs migration apply
```

### 4. Model Context Protocol (MCP) Server (`clrinfcs mcp`)
AI ajanları için Roslyn AST linter ve mimari doğrulama fonksiyonlarını stdio JSON-RPC 2.0 üzerinden sunar:
```bash
clrinfcs mcp
```

### 5. AI Kural Sentezleyici (`clrinfcs ai rule`)
Belirtilen varlık ve kural için Roslyn uyumlu iş kuralı iskeleti sentezler:
```bash
clrinfcs ai rule CheckMaxDiscount --entity Order
```

