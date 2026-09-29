using System.Threading;
using System.Threading.Tasks;
using MediatR;

namespace MediatRApp.Features.Benchmark;

public record BenchmarkCommand(string Message) : IRequest<string>;

public class BenchmarkCommandHandler : IRequestHandler<BenchmarkCommand, string>
{
    public Task<string> Handle(BenchmarkCommand request, CancellationToken cancellationToken)
    {
        return Task.FromResult($"MediatR Handled Command: {request.Message}");
    }
}

public record BenchmarkQuery(int Id) : IRequest<string>;

public class BenchmarkQueryHandler : IRequestHandler<BenchmarkQuery, string>
{
    public Task<string> Handle(BenchmarkQuery request, CancellationToken cancellationToken)
    {
        return Task.FromResult($"MediatR Handled Query: {request.Id}");
    }
}

public class PassThroughBehavior1<TRequest, TResponse> : IPipelineBehavior<TRequest, TResponse> where TRequest : notnull
{
    public Task<TResponse> Handle(TRequest request, RequestHandlerDelegate<TResponse> next, CancellationToken cancellationToken)
    {
        return next();
    }
}

public class PassThroughBehavior2<TRequest, TResponse> : IPipelineBehavior<TRequest, TResponse> where TRequest : notnull
{
    public Task<TResponse> Handle(TRequest request, RequestHandlerDelegate<TResponse> next, CancellationToken cancellationToken)
    {
        return next();
    }
}
