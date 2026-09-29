using System;
using System.Collections.Frozen;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.Extensions.DependencyInjection;

namespace NativeApp.Common;

public interface ICommand<out TResponse> { }
public interface ICommand : ICommand<Unit> { }
public interface IQuery<out TResponse> { }

public readonly record struct Unit
{
    public static readonly Unit Value = new();
    public static readonly ValueTask<Unit> Task = ValueTask.FromResult(Value);
}

public delegate ValueTask<TResponse> RequestHandlerDelegate<TResponse>();

public interface ICommandHandler<in TRequest, TResponse>
{
    ValueTask<TResponse> HandleAsync(TRequest request, CancellationToken cancellationToken);
}

public interface ICommandHandler<in TRequest> : ICommandHandler<TRequest, Unit> { }

public interface IQueryHandler<in TRequest, TResponse>
{
    ValueTask<TResponse> HandleAsync(TRequest request, CancellationToken cancellationToken);
}

public interface IPipelineBehavior<in TRequest, TResponse>
{
    ValueTask<TResponse> HandleAsync(TRequest request, CancellationToken cancellationToken, RequestHandlerDelegate<TResponse> next);
}

public interface IDispatcher
{
    ValueTask<TResponse> SendAsync<TResponse>(ICommand<TResponse> command, CancellationToken cancellationToken = default);
    ValueTask SendAsync(ICommand command, CancellationToken cancellationToken = default);
    ValueTask<TResponse> QueryAsync<TResponse>(IQuery<TResponse> query, CancellationToken cancellationToken = default);
}

public sealed class Dispatcher : IDispatcher
{
    private readonly IServiceProvider _serviceProvider;
    private static readonly FrozenDictionary<Type, object> _handlerCache;

    // Lock-free static initialization via CLR type initializer
    static Dispatcher()
    {
        var registry = new Dictionary<Type, object>();
        
        // Compile-time / static registration hook for zero-reflection discovery
        GeneratedHandlerRegistry.RegisterHandlers(registry);

        // Assembly scanning fallback for dynamic discovery
        var assemblies = AppDomain.CurrentDomain.GetAssemblies();
        foreach (var assembly in assemblies)
        {
            if (assembly.IsDynamic) continue;
            Type[] types;
            try { types = assembly.GetTypes(); } catch { continue; }

            foreach (var type in types.Where(t => !t.IsAbstract && !t.IsInterface))
            {
                foreach (var iface in type.GetInterfaces().Where(i => i.IsGenericType))
                {
                    var genericDef = iface.GetGenericTypeDefinition();
                    if (genericDef == typeof(ICommandHandler<,>))
                    {
                        var args = iface.GetGenericArguments();
                        var requestType = args[0];
                        var responseType = args[1];

                        if (!registry.ContainsKey(requestType))
                        {
                            var wrapperType = typeof(CommandHandlerWrapperImpl<,>).MakeGenericType(requestType, responseType);
                            var wrapper = Activator.CreateInstance(wrapperType)!;
                            registry[requestType] = wrapper;
                        }
                    }
                    else if (genericDef == typeof(IQueryHandler<,>))
                    {
                        var args = iface.GetGenericArguments();
                        var requestType = args[0];
                        var responseType = args[1];

                        if (!registry.ContainsKey(requestType))
                        {
                            var wrapperType = typeof(QueryHandlerWrapperImpl<,>).MakeGenericType(requestType, responseType);
                            var wrapper = Activator.CreateInstance(wrapperType)!;
                            registry[requestType] = wrapper;
                        }
                    }
                }
            }
        }

        _handlerCache = registry.ToFrozenDictionary();
    }

    public Dispatcher(IServiceProvider serviceProvider)
    {
        _serviceProvider = serviceProvider;
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public ValueTask<TResponse> SendAsync<TResponse>(ICommand<TResponse> command, CancellationToken cancellationToken = default)
    {
        return ExecuteAsync<TResponse>(command, cancellationToken);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public ValueTask SendAsync(ICommand command, CancellationToken cancellationToken = default)
    {
        var task = ExecuteAsync<Unit>(command, cancellationToken);
        if (task.IsCompletedSuccessfully)
        {
            task.GetAwaiter().GetResult();
            return ValueTask.CompletedTask;
        }
        return AwaitSendAsync(task);

        static async ValueTask AwaitSendAsync(ValueTask<Unit> task)
        {
            await task;
        }
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public ValueTask<TResponse> QueryAsync<TResponse>(IQuery<TResponse> query, CancellationToken cancellationToken = default)
    {
        return ExecuteAsync<TResponse>(query, cancellationToken);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    private ValueTask<TResponse> ExecuteAsync<TResponse>(object request, CancellationToken cancellationToken)
    {
        ArgumentNullException.ThrowIfNull(request);
        var requestType = request.GetType();

        if (!_handlerCache.TryGetValue(requestType, out var wrapper))
        {
            throw new InvalidOperationException($"No handler registered for request '{requestType.Name}'.");
        }

        return ((RequestHandlerWrapper<TResponse>)wrapper).Handle(_serviceProvider, request, cancellationToken);
    }
}

// Pre-generated static handler registry hook (Roslyn Source Generator target)
internal static class GeneratedHandlerRegistry
{
    public static void RegisterHandlers(Dictionary<Type, object> registry)
    {
        // Explicit static mapping example generated at build-time
        // registry[typeof(BenchmarkCommand)] = new CommandHandlerWrapperImpl<BenchmarkCommand, string>();
    }
}

internal abstract class RequestHandlerWrapper<TResponse>
{
    public abstract ValueTask<TResponse> Handle(IServiceProvider serviceProvider, object request, CancellationToken cancellationToken);
}

internal sealed class CommandHandlerWrapperImpl<TRequest, TResponse> : RequestHandlerWrapper<TResponse>
    where TRequest : class
{
    public override ValueTask<TResponse> Handle(IServiceProvider serviceProvider, object request, CancellationToken cancellationToken)
    {
        var typedRequest = (TRequest)request;
        var commandHandler = serviceProvider.GetRequiredService<ICommandHandler<TRequest, TResponse>>();
        var rawBehaviors = serviceProvider.GetService<IEnumerable<IPipelineBehavior<TRequest, TResponse>>>();
        var behaviors = rawBehaviors as IPipelineBehavior<TRequest, TResponse>[] ?? rawBehaviors?.ToArray() ?? Array.Empty<IPipelineBehavior<TRequest, TResponse>>();

        return PipelineExecutor.ExecuteCommand(typedRequest, cancellationToken, behaviors, commandHandler);
    }
}

internal sealed class QueryHandlerWrapperImpl<TRequest, TResponse> : RequestHandlerWrapper<TResponse>
    where TRequest : class
{
    public override ValueTask<TResponse> Handle(IServiceProvider serviceProvider, object request, CancellationToken cancellationToken)
    {
        var typedRequest = (TRequest)request;
        var queryHandler = serviceProvider.GetRequiredService<IQueryHandler<TRequest, TResponse>>();
        var rawBehaviors = serviceProvider.GetService<IEnumerable<IPipelineBehavior<TRequest, TResponse>>>();
        var behaviors = rawBehaviors as IPipelineBehavior<TRequest, TResponse>[] ?? rawBehaviors?.ToArray() ?? Array.Empty<IPipelineBehavior<TRequest, TResponse>>();

        return PipelineExecutor.ExecuteQuery(typedRequest, cancellationToken, behaviors, queryHandler);
    }
}

internal static class PipelineExecutor
{
    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static ValueTask<TResponse> ExecuteCommand<TRequest, TResponse>(
        TRequest request,
        CancellationToken cancellationToken,
        IPipelineBehavior<TRequest, TResponse>[] behaviors,
        ICommandHandler<TRequest, TResponse> handler)
        where TRequest : class
    {
        int count = behaviors.Length;
        if (count == 0)
        {
            return handler.HandleAsync(request, cancellationToken);
        }
        if (count == 1)
        {
            return behaviors[0].HandleAsync(request, cancellationToken, () => handler.HandleAsync(request, cancellationToken));
        }
        if (count == 2)
        {
            var b0 = behaviors[0];
            var b1 = behaviors[1];
            return b0.HandleAsync(request, cancellationToken, () => b1.HandleAsync(request, cancellationToken, () => handler.HandleAsync(request, cancellationToken)));
        }

        return ExecuteGenericCommand(request, cancellationToken, behaviors, handler);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static ValueTask<TResponse> ExecuteQuery<TRequest, TResponse>(
        TRequest request,
        CancellationToken cancellationToken,
        IPipelineBehavior<TRequest, TResponse>[] behaviors,
        IQueryHandler<TRequest, TResponse> handler)
        where TRequest : class
    {
        int count = behaviors.Length;
        if (count == 0)
        {
            return handler.HandleAsync(request, cancellationToken);
        }
        if (count == 1)
        {
            return behaviors[0].HandleAsync(request, cancellationToken, () => handler.HandleAsync(request, cancellationToken));
        }
        if (count == 2)
        {
            var b0 = behaviors[0];
            var b1 = behaviors[1];
            return b0.HandleAsync(request, cancellationToken, () => b1.HandleAsync(request, cancellationToken, () => handler.HandleAsync(request, cancellationToken)));
        }

        return ExecuteGenericQuery(request, cancellationToken, behaviors, handler);
    }

    private static ValueTask<TResponse> ExecuteGenericCommand<TRequest, TResponse>(
        TRequest request,
        CancellationToken cancellationToken,
        IPipelineBehavior<TRequest, TResponse>[] behaviors,
        ICommandHandler<TRequest, TResponse> handler)
        where TRequest : class
    {
        var context = new CommandPipelineContext<TRequest, TResponse>(behaviors, handler, request, cancellationToken);
        return context.InvokeNext();
    }

    private static ValueTask<TResponse> ExecuteGenericQuery<TRequest, TResponse>(
        TRequest request,
        CancellationToken cancellationToken,
        IPipelineBehavior<TRequest, TResponse>[] behaviors,
        IQueryHandler<TRequest, TResponse> handler)
        where TRequest : class
    {
        var context = new QueryPipelineContext<TRequest, TResponse>(behaviors, handler, request, cancellationToken);
        return context.InvokeNext();
    }

    private sealed class CommandPipelineContext<TRequest, TResponse>
        where TRequest : class
    {
        private readonly IPipelineBehavior<TRequest, TResponse>[] _behaviors;
        private readonly ICommandHandler<TRequest, TResponse> _handler;
        private readonly TRequest _request;
        private readonly CancellationToken _cancellationToken;
        private int _index;
        private readonly RequestHandlerDelegate<TResponse> _next;

        public CommandPipelineContext(
            IPipelineBehavior<TRequest, TResponse>[] behaviors,
            ICommandHandler<TRequest, TResponse> handler,
            TRequest request,
            CancellationToken cancellationToken)
        {
            _behaviors = behaviors;
            _handler = handler;
            _request = request;
            _cancellationToken = cancellationToken;
            _index = 0;
            _next = InvokeNext;
        }

        public ValueTask<TResponse> InvokeNext()
        {
            if (_index < _behaviors.Length)
            {
                var behavior = _behaviors[_index++];
                return behavior.HandleAsync(_request, _cancellationToken, _next);
            }
            return _handler.HandleAsync(_request, _cancellationToken);
        }
    }

    private sealed class QueryPipelineContext<TRequest, TResponse>
        where TRequest : class
    {
        private readonly IPipelineBehavior<TRequest, TResponse>[] _behaviors;
        private readonly IQueryHandler<TRequest, TResponse> _handler;
        private readonly TRequest _request;
        private readonly CancellationToken _cancellationToken;
        private int _index;
        private readonly RequestHandlerDelegate<TResponse> _next;

        public QueryPipelineContext(
            IPipelineBehavior<TRequest, TResponse>[] behaviors,
            IQueryHandler<TRequest, TResponse> handler,
            TRequest request,
            CancellationToken cancellationToken)
        {
            _behaviors = behaviors;
            _handler = handler;
            _request = request;
            _cancellationToken = cancellationToken;
            _index = 0;
            _next = InvokeNext;
        }

        public ValueTask<TResponse> InvokeNext()
        {
            if (_index < _behaviors.Length)
            {
                var behavior = _behaviors[_index++];
                return behavior.HandleAsync(_request, _cancellationToken, _next);
            }
            return _handler.HandleAsync(_request, _cancellationToken);
        }
    }
}

