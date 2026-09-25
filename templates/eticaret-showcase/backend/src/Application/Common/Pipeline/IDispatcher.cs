using System.Threading;
using System.Threading.Tasks;

namespace EticaretApp.Application.Common.Pipeline;

public interface IDispatcher
{
    ValueTask<TResponse> SendAsync<TResponse>(ICommand<TResponse> command, CancellationToken cancellationToken = default);
    ValueTask SendAsync(ICommand command, CancellationToken cancellationToken = default);
    ValueTask<TResponse> QueryAsync<TResponse>(IQuery<TResponse> query, CancellationToken cancellationToken = default);
}
