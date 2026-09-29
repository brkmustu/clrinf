namespace CrmMonolith.Common.Pipeline;

using System.Threading;
using System.Threading.Tasks;

public interface ICommand<out TResponse> { }
public interface IQuery<out TResponse> { }

public interface ICommandHandler<in TCommand, TResponse> where TCommand : ICommand<TResponse>
{
    ValueTask<TResponse> HandleAsync(TCommand command, CancellationToken ct = default);
}

public interface IQueryHandler<in TQuery, TResponse> where TQuery : IQuery<TResponse>
{
    ValueTask<TResponse> HandleAsync(TQuery query, CancellationToken ct = default);
}
