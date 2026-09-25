using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using EticaretApp.Application.Common.Pipeline;
using EticaretApp.Application.Common.Validation;

namespace EticaretApp.Application.Common.Behaviors;

public sealed class ValidationBehavior<TRequest, TResponse> : IPipelineBehavior<TRequest, TResponse>
    where TRequest : notnull
{
    private readonly IEnumerable<IValidator<TRequest>> _validators;

    public ValidationBehavior(IEnumerable<IValidator<TRequest>> validators)
    {
        _validators = validators;
    }

    public async ValueTask<TResponse> HandleAsync(
        TRequest request,
        CancellationToken cancellationToken,
        RequestHandlerDelegate<TResponse> next)

    {
        if (_validators != null && _validators.Any())
        {
            foreach (var validator in _validators)
            {
                await validator.ValidateAsync(request, cancellationToken);
            }
        }

        return await next();
    }
}
