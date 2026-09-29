namespace CrmMonolith.Modules.Deal;

using CrmMonolith.Common.Pipeline;
using CrmMonolith.Common.Functional;
using CrmMonolith.Domain.Common.Exceptions;
using CrmMonolith.Data;
using Microsoft.EntityFrameworkCore;

public static class GetById
{
    public sealed record Query(string Id) : IQuery<Result<Response, DomainError>>;

    public sealed record Response(
        string Id,
        string Title,
        string TenantId,
        string ContactId,
        decimal Amount,
        string Stage,
        int Probability
    );

    internal sealed class Handler(CrmDbContext db) :
        IQueryHandler<Query, Result<Response, DomainError>>
    {
        public async ValueTask<Result<Response, DomainError>> HandleAsync(Query request, CancellationToken ct)
        {
            var entity = await db.Deals
                .AsNoTracking()
                .FirstOrDefaultAsync(x => x.Id.Equals(request.Id), ct);

            if (entity is null)
                return Result<Response, DomainError>.Failure(new DomainError("Deal.NotFound", $"Deal with Id '{request.Id}' was not found."));

            var response = new Response(
                entity.Id,
                entity.Title,
                entity.TenantId,
                entity.ContactId,
                entity.Amount,
                entity.Stage,
                entity.Probability
            );

            return Result<Response, DomainError>.Success(response);
        }
    }
}
