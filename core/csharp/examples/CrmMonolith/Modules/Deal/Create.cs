namespace CrmMonolith.Modules.Deal;

using CrmMonolith.Common.Pipeline;
using CrmMonolith.Common.Functional;
using CrmMonolith.Domain.Common.Exceptions;
using CrmMonolith.Domain.Entities;
using CrmMonolith.Data;

public static class Create
{
    public sealed record Command(
        string Title,
        string TenantId,
        string ContactId,
        decimal Amount,
        string Stage,
        int Probability
    ) : ICommand<Result<Response, DomainError>>;

    public sealed record Response(string Id);

    internal sealed class Handler(CrmDbContext db) :
        ICommandHandler<Command, Result<Response, DomainError>>
    {
        public async ValueTask<Result<Response, DomainError>> HandleAsync(Command request, CancellationToken ct)
        {
            var entity = new Deal
            {
                Title = request.Title,
                TenantId = request.TenantId,
                ContactId = request.ContactId,
                Amount = request.Amount,
                Stage = request.Stage,
                Probability = request.Probability,
            };

            db.Deals.Add(entity);
            await db.SaveChangesAsync(ct);

            return Result<Response, DomainError>.Success(new Response(entity.Id));
        }
    }
}
