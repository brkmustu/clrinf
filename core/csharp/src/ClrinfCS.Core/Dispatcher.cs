using System.Collections.Frozen;

namespace ClrinfCS.Core;

public interface IRequest<TResponse>;

public interface IRequestHandler<in TRequest, TResponse> where TRequest : IRequest<TResponse>
{
    ValueTask<TResponse> HandleAsync(TRequest request, RequestContext context, CancellationToken cancellationToken);
}

public delegate ValueTask<TResponse> RequestHandlerDelegate<TResponse>();

public interface IPipelineBehavior<in TRequest, TResponse> where TRequest : IRequest<TResponse>
{
    ValueTask<TResponse> HandleAsync(TRequest request, RequestContext context,
        CancellationToken cancellationToken, RequestHandlerDelegate<TResponse> next);
}

public interface IDispatcher
{
    ValueTask<TResponse> SendAsync<TResponse>(IRequest<TResponse> request, RequestContext context,
        CancellationToken cancellationToken = default);
}

/// <summary>Immutable, explicit routing; no process-wide cache or assembly scanning.</summary>
public sealed class Dispatcher : IDispatcher
{
    private readonly FrozenDictionary<Type, object> routes;
    private Dispatcher(Dictionary<Type, object> routes) => this.routes = routes.ToFrozenDictionary();

    public ValueTask<TResponse> SendAsync<TResponse>(IRequest<TResponse> request, RequestContext context,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(request);
        ArgumentNullException.ThrowIfNull(context);
        cancellationToken.ThrowIfCancellationRequested();
        if (!routes.TryGetValue(request.GetType(), out var route) || route is not Route<TResponse> typed)
            throw new InvalidOperationException($"No handler registered for {request.GetType().FullName} returning {typeof(TResponse).FullName}.");
        return typed.Invoke(request, context, cancellationToken);
    }

    private delegate ValueTask<TResponse> Route<TResponse>(IRequest<TResponse> request, RequestContext context,
        CancellationToken cancellationToken);

    public sealed class Builder
    {
        private readonly Dictionary<Type, object> routes = new();

        public Builder Register<TRequest, TResponse>(IRequestHandler<TRequest, TResponse> handler,
            params IPipelineBehavior<TRequest, TResponse>[] behaviors) where TRequest : IRequest<TResponse> =>
            Register(handler, null, behaviors);

        public Builder Register<TRequest, TResponse>(IRequestHandler<TRequest, TResponse> handler,
            IEnumerable<IBusinessRule<TRequest>>? rules,
            params IPipelineBehavior<TRequest, TResponse>[] behaviors) where TRequest : IRequest<TResponse>
        {
            ArgumentNullException.ThrowIfNull(handler);
            ArgumentNullException.ThrowIfNull(behaviors);
            if (behaviors.Any(b => b is null)) throw new ArgumentException("A behavior cannot be null.", nameof(behaviors));

            var pipelineList = new List<IPipelineBehavior<TRequest, TResponse>>();
            if (rules is not null)
            {
                var ruleArray = rules.ToArray();
                if (ruleArray.Length > 0)
                    pipelineList.Add(new BusinessRulePipelineBehavior<TRequest, TResponse>(ruleArray));
            }
            pipelineList.AddRange(behaviors);

            var pipeline = pipelineList.ToArray();
            Route<TResponse> route = (request, context, cancellationToken) =>
            {
                var typed = (TRequest)request;
                RequestHandlerDelegate<TResponse> next = () => handler.HandleAsync(typed, context, cancellationToken);
                for (var i = pipeline.Length - 1; i >= 0; i--)
                {
                    var behavior = pipeline[i];
                    var downstream = next;
                    next = () => behavior.HandleAsync(typed, context, cancellationToken, downstream);
                }
                return next();
            };
            if (!routes.TryAdd(typeof(TRequest), route))
                throw new InvalidOperationException($"A handler for {typeof(TRequest).FullName} is already registered.");
            return this;
        }

        public Dispatcher Build() => new(routes);
    }
}
