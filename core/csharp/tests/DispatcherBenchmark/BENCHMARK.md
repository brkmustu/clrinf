# Comprehensive CQRS Dispatcher Performance Report & Native .NET 10 Optimization

This benchmark compares the performance, execution latency, and memory allocation of:
1. **Native .NET 10 Pipeline Dispatcher** (`Application.Common.Pipeline.Dispatcher`) - powered by `FrozenDictionary<Type, RequestHandlerWrapper>`, `ValueTask<T>`, and zero reflection at dispatch time.
2. **MediatR v12.4.1** (Standard CQRS mediator baseline).
3. **Direct Typed Dispatcher** (Zero-reflection, direct generic DI lookup baseline).

---

## Key Native .NET 10 Optimizations Implemented

- **`FrozenDictionary<Type, RequestHandlerWrapper>`:**
  Request handler type scanning is executed once during initial startup. Dispatching performs an immutable $O(1)$ dictionary lookup with zero runtime reflection (`MakeGenericType` / `GetMethod().Invoke()` calls removed).
- **`ValueTask<T>` Return Types:**
  All handler and pipeline behavior signatures utilize `ValueTask<T>`, eliminating `Task` heap allocation overhead when handlers complete synchronously.
- **Allocation-Free Recursive Delegate Composition:**
  Behaviors iterate pre-resolved behavior arrays in reverse order directly, eliminating per-dispatch LINQ `.Reverse()` iterator allocations.

---

## Architecture Summary

```csharp
public sealed class Dispatcher : IDispatcher
{
    private static FrozenDictionary<Type, RequestHandlerWrapper>? _handlerCache;

    public ValueTask<TResponse> SendAsync<TResponse>(ICommand<TResponse> command, CancellationToken cancellationToken = default)
    {
        return ExecuteAsync<TResponse>(command, cancellationToken);
    }
}
```
