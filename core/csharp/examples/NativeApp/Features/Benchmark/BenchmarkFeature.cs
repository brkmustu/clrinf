using System;
using System.Threading;
using System.Threading.Tasks;
using NativeApp.Common;

namespace NativeApp.Features.Benchmark;

public record BenchmarkCommand(string Message) : ICommand<string>;

public class BenchmarkCommandHandler : ICommandHandler<BenchmarkCommand, string>
{
    public ValueTask<string> HandleAsync(BenchmarkCommand request, CancellationToken cancellationToken)
    {
        return ValueTask.FromResult("NativeApp Command OK: " + request.Message);
    }
}

public record BenchmarkQuery(int Id) : IQuery<string>;

public class BenchmarkQueryHandler : IQueryHandler<BenchmarkQuery, string>
{
    public ValueTask<string> HandleAsync(BenchmarkQuery request, CancellationToken cancellationToken)
    {
        return ValueTask.FromResult("NativeApp Query OK: " + request.Id);
    }
}

public class PassThroughBehavior1<TRequest, TResponse> : IPipelineBehavior<TRequest, TResponse>
{
    public ValueTask<TResponse> HandleAsync(TRequest request, CancellationToken cancellationToken, RequestHandlerDelegate<TResponse> next)
    {
        return next();
    }
}

public class PassThroughBehavior2<TRequest, TResponse> : IPipelineBehavior<TRequest, TResponse>
{
    public ValueTask<TResponse> HandleAsync(TRequest request, CancellationToken cancellationToken, RequestHandlerDelegate<TResponse> next)
    {
        return next();
    }
}
