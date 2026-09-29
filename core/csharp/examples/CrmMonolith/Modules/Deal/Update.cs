namespace CrmMonolith.Modules.Deal;

using CrmMonolith.Common.Pipeline;
using CrmMonolith.Common.Functional;
using CrmMonolith.Domain.Common.Exceptions;
using CrmMonolith.Domain.Entities;
using CrmMonolith.Data;

public static class Update
{
    public sealed record Command(
        string Id,
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
            var entity = await db.Deals.FindAsync(new object[] { request.Id }, ct);
            if (entity is null)
                return Result<Response, DomainError>.Failure(new DomainError("Deal.NotFound", $"Deal with Id '{request.Id}' was not found."));

            entity.Title = request.Title;
            entity.TenantId = request.TenantId;
            entity.ContactId = request.ContactId;
            entity.Amount = request.Amount;
            entity.Stage = request.Stage;
            entity.Probability = request.Probability;

            db.Deals.Update(entity);
            await db.SaveChangesAsync(ct);

            return Result<Response, DomainError>.Success(new Response(entity.Id));
        }
    }
}
