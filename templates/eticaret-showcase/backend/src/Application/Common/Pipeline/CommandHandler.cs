using System.Threading;
using System.Threading.Tasks;

namespace EticaretApp.Application.Common.Pipeline;

/// <summary>
/// Convenience base class for command handlers that do not return a meaningful response.
/// Wraps <see cref="ICommandHandler{TCommand, Unit}"/> and returns <see cref="Unit.Value"/> automatically.
/// </summary>
public abstract class CommandHandler<TCommand> : ICommandHandler<TCommand, Unit>
    where TCommand : class, ICommand
{
    async ValueTask<Unit> ICommandHandler<TCommand, Unit>.HandleAsync(TCommand command, CancellationToken cancellationToken)
    {
        await HandleAsync(command, cancellationToken);
        return Unit.Value;
    }

    /// <summary>
    /// Handles the command. Override this method in the derived handler.
    /// </summary>
    protected abstract ValueTask HandleAsync(TCommand command, CancellationToken cancellationToken);
}
