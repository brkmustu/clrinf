using System.Threading;
using System.Threading.Tasks;

namespace EticaretApp.Application.Common.Pipeline;

/// <summary>
/// Defines a handler for a query that returns data.
/// </summary>
/// <typeparam name="TQuery">The query type.</typeparam>
/// <typeparam name="TResponse">The response type.</typeparam>
public interface IQueryHandler<in TQuery, TResponse> where TQuery : class
{
    ValueTask<TResponse> HandleAsync(TQuery query, CancellationToken cancellationToken = default);
}
