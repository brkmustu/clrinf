namespace CrmMonolith.Modules.Deal;

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
        string Title,
        string TenantId,
        string ContactId,
        decimal Amount,
        string Stage,
        int Probability
    );

    public sealed record Response(IReadOnlyList<ItemDto> Items, int Page, int PageSize, int TotalCount);

    internal sealed class Handler(CrmDbContext db) :
        IQueryHandler<Query, Result<Response, DomainError>>
    {
        public async ValueTask<Result<Response, DomainError>> HandleAsync(Query request, CancellationToken ct)
        {
            var totalCount = await db.Deals.CountAsync(ct);

            var items = await db.Deals
                .AsNoTracking()
                .OrderBy(x => x.Id)
                .Skip((request.Page - 1) * request.PageSize)
                .Take(request.PageSize)
                .Select(x => new ItemDto(
                    x.Id,
                    x.Title,
                    x.TenantId,
                    x.ContactId,
                    x.Amount,
                    x.Stage,
                    x.Probability
                ))
                .ToListAsync(ct);

            var response = new Response(items, request.Page, request.PageSize, totalCount);

            return Result<Response, DomainError>.Success(response);
        }
    }
}
