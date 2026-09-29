namespace CrmMonolith.Modules.Activity;

using CrmMonolith.Common.Pipeline;
using CrmMonolith.Common.Functional;
using CrmMonolith.Domain.Common.Exceptions;
using CrmMonolith.Data;
using Microsoft.EntityFrameworkCore;

public static class GetList
{
    public sealed record Query(int Page = 1, int PageSize = 20) : IQuery<Result<Response, DomainError>>;

    public sealed record ItemDto(
        string Id,
        string TenantId,
        string Title,
        string Type,
        DateTime DueDate,
        string? Notes,
        string? DealId,
        string? ContactId
    );

    public sealed record Response(IReadOnlyList<ItemDto> Items, int Page, int PageSize, int TotalCount);

    internal sealed class Handler(CrmDbContext db) :
        IQueryHandler<Query, Result<Response, DomainError>>
    {
        public async ValueTask<Result<Response, DomainError>> HandleAsync(Query request, CancellationToken ct)
        {
            var totalCount = await db.Activities.CountAsync(ct);

            var items = await db.Activities
                .AsNoTracking()
                .OrderBy(x => x.Id)
                .Skip((request.Page - 1) * request.PageSize)
                .Take(request.PageSize)
                .Select(x => new ItemDto(
                    x.Id,
                    x.TenantId,
                    x.Title,
                    x.Type,
                    x.DueDate,
                    x.Notes,
                    x.DealId,
                    x.ContactId
                ))
                .ToListAsync(ct);

            var response = new Response(items, request.Page, request.PageSize, totalCount);

            return Result<Response, DomainError>.Success(response);
        }
    }
}
