using System.Threading;
using System.Threading.Tasks;

namespace EticaretApp.Application.Common.Pipeline;

/// <summary>
/// Represents the next step in the pipeline chain.
/// </summary>
/// <typeparam name="TResponse">The response type of the pipeline.</typeparam>
public delegate ValueTask<TResponse> RequestHandlerDelegate<TResponse>();

/// <summary>
/// Defines a pipeline behavior that wraps handler execution.
/// </summary>
/// <typeparam name="TRequest">The request type (command or query).</typeparam>
/// <typeparam name="TResponse">The response type.</typeparam>
public interface IPipelineBehavior<in TRequest, TResponse>
{
    /// <summary>
    /// Executes the behavior logic before and/or after calling <paramref name="next"/>.
    /// </summary>
    ValueTask<TResponse> HandleAsync(
        TRequest request,
        CancellationToken cancellationToken,
        RequestHandlerDelegate<TResponse> next);
}
