using System.Threading;
using System.Threading.Tasks;
using Microsoft.Extensions.Logging;
using EticaretApp.Application.Common.Pipeline;

namespace EticaretApp.Application.Common.Behaviors;

public sealed class LoggingBehavior<TRequest, TResponse> : IPipelineBehavior<TRequest, TResponse>
    where TRequest : notnull
{
    private readonly ILogger<LoggingBehavior<TRequest, TResponse>> _logger;

    public LoggingBehavior(ILogger<LoggingBehavior<TRequest, TResponse>> logger)
    {
        _logger = logger;
    }

    public async ValueTask<TResponse> HandleAsync(
        TRequest request,
        CancellationToken cancellationToken,
        RequestHandlerDelegate<TResponse> next)

    {
        _logger.LogInformation("Handling request {RequestType}", typeof(TRequest).Name);
        var response = await next();
        _logger.LogInformation("Handled request {RequestType}", typeof(TRequest).Name);

        return response;
    }
}
