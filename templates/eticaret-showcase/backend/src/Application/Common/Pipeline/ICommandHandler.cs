using System.Threading;
using System.Threading.Tasks;

namespace EticaretApp.Application.Common.Pipeline;

/// <summary>
/// Defines a handler for a command that produces a response.
/// </summary>
/// <typeparam name="TCommand">The command type.</typeparam>
/// <typeparam name="TResponse">The response type.</typeparam>
public interface ICommandHandler<in TCommand, TResponse> where TCommand : class
{
    ValueTask<TResponse> HandleAsync(TCommand command, CancellationToken cancellationToken = default);
}
