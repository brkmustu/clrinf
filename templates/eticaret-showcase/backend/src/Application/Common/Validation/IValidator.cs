using System.Threading;
using System.Threading.Tasks;

namespace EticaretApp.Application.Common.Validation;

public interface IValidator<in T>
{
    Task ValidateAsync(T instance, CancellationToken cancellationToken = default);
}
