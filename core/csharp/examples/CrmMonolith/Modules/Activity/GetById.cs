namespace CrmMonolith.Modules.Activity;

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
        string TenantId,
        string Title,
        string Type,
        DateTime DueDate,
        string? Notes,
        string? DealId,
        string? ContactId
    );

    internal sealed class Handler(CrmDbContext db) :
        IQueryHandler<Query, Result<Response, DomainError>>
    {
        public async ValueTask<Result<Response, DomainError>> HandleAsync(Query request, CancellationToken ct)
        {
            var entity = await db.Activities
                .AsNoTracking()
                .FirstOrDefaultAsync(x => x.Id.Equals(request.Id), ct);

            if (entity is null)
                return Result<Response, DomainError>.Failure(new DomainError("Activity.NotFound", $"Activity with Id '{request.Id}' was not found."));

            var response = new Response(
                entity.Id,
                entity.TenantId,
                entity.Title,
                entity.Type,
                entity.DueDate,
                entity.Notes,
                entity.DealId,
                entity.ContactId
            );

            return Result<Response, DomainError>.Success(response);
        }
    }
}
