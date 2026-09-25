using System;
using System.Collections.Frozen;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.Extensions.DependencyInjection;

namespace EticaretApp.Application.Common.Pipeline;

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

    public ValueTask<TResponse> SendAsync<TResponse>(ICommand<TResponse> command, CancellationToken cancellationToken = default)
    {
        return ExecuteAsync<TResponse>(command, cancellationToken);
    }

    public async ValueTask SendAsync(ICommand command, CancellationToken cancellationToken = default)
    {
        await ExecuteAsync<Unit>(command, cancellationToken);
    }

    public ValueTask<TResponse> QueryAsync<TResponse>(IQuery<TResponse> query, CancellationToken cancellationToken = default)
    {
        return ExecuteAsync<TResponse>(query, cancellationToken);
    }

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
        // Explicit static mapping generated at build-time
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
        var behaviors = serviceProvider.GetServices<IPipelineBehavior<TRequest, TResponse>>();

        return PipelineExecutor.Execute(typedRequest, cancellationToken, behaviors, (req, ct) => commandHandler.HandleAsync(req, ct));
    }
}

internal sealed class QueryHandlerWrapperImpl<TRequest, TResponse> : RequestHandlerWrapper<TResponse>
    where TRequest : class
{
    public override ValueTask<TResponse> Handle(IServiceProvider serviceProvider, object request, CancellationToken cancellationToken)
    {
        var typedRequest = (TRequest)request;
        var queryHandler = serviceProvider.GetRequiredService<IQueryHandler<TRequest, TResponse>>();
        var behaviors = serviceProvider.GetServices<IPipelineBehavior<TRequest, TResponse>>();

        return PipelineExecutor.Execute(typedRequest, cancellationToken, behaviors, (req, ct) => queryHandler.HandleAsync(req, ct));
    }
}

internal static class PipelineExecutor
{
    // Single consolidated generic pipeline executor for both Command and Query handlers
    public static ValueTask<TResponse> Execute<TRequest, TResponse>(
        TRequest request,
        CancellationToken cancellationToken,
        IEnumerable<IPipelineBehavior<TRequest, TResponse>> behaviors,
        Func<TRequest, CancellationToken, ValueTask<TResponse>> handlerInvocation)
        where TRequest : class
    {
        if (behaviors is IPipelineBehavior<TRequest, TResponse>[] behaviorArray)
        {
            if (behaviorArray.Length == 0)
            {
                return handlerInvocation(request, cancellationToken);
            }

            int index = 0;
            RequestHandlerDelegate<TResponse> next = null!;
            next = () =>
            {
                if (index < behaviorArray.Length)
                {
                    var behavior = behaviorArray[index++];
                    return behavior.HandleAsync(request, cancellationToken, next);
                }
                return handlerInvocation(request, cancellationToken);
            };

            return next();
        }
        else
        {
            var list = behaviors.ToList();
            if (list.Count == 0)
            {
                return handlerInvocation(request, cancellationToken);
            }

            int index = 0;
            RequestHandlerDelegate<TResponse> next = null!;
            next = () =>
            {
                if (index < list.Count)
                {
                    var behavior = list[index++];
                    return behavior.HandleAsync(request, cancellationToken, next);
                }
                return handlerInvocation(request, cancellationToken);
            };

            return next();
        }
    }
}
