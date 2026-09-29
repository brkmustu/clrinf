using System.Threading;
using System.Threading.Tasks;

namespace PureNativeApp.Common;

public interface ICommand<out TResponse>
{
}

public interface IQuery<out TResponse>
{
}

public interface ICommandHandler<in TCommand, TResponse> where TCommand : ICommand<TResponse>
{
    ValueTask<TResponse> HandleAsync(TCommand command, CancellationToken cancellationToken = default);
}

public interface IQueryHandler<in TQuery, TResponse> where TQuery : IQuery<TResponse>
{
    ValueTask<TResponse> HandleAsync(TQuery query, CancellationToken cancellationToken = default);
}
