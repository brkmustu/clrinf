using System.Threading;
using System.Threading.Tasks;
using PureNativeApp.Common;

namespace PureNativeApp.Features.Benchmark;

public record BenchmarkCommand(string Message) : ICommand<string>;

public class BenchmarkCommandHandler : ICommandHandler<BenchmarkCommand, string>
{
    public ValueTask<string> HandleAsync(BenchmarkCommand request, CancellationToken cancellationToken = default)
    {
        return ValueTask.FromResult("PureNativeApp Command OK: " + request.Message);
    }
}

public record BenchmarkQuery(int Id) : IQuery<string>;

public class BenchmarkQueryHandler : IQueryHandler<BenchmarkQuery, string>
{
    public ValueTask<string> HandleAsync(BenchmarkQuery request, CancellationToken cancellationToken = default)
    {
        return ValueTask.FromResult("PureNativeApp Query OK: " + request.Id);
    }
}
