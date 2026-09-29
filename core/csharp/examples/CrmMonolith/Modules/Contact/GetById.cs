namespace CrmMonolith.Modules.Contact;

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
        string FullName,
        string Email,
        string? Phone,
        string? Company
    );

    internal sealed class Handler(CrmDbContext db) :
        IQueryHandler<Query, Result<Response, DomainError>>
    {
        public async ValueTask<Result<Response, DomainError>> HandleAsync(Query request, CancellationToken ct)
        {
            var entity = await db.Contacts
                .AsNoTracking()
                .FirstOrDefaultAsync(x => x.Id.Equals(request.Id), ct);

            if (entity is null)
                return Result<Response, DomainError>.Failure(new DomainError("Contact.NotFound", $"Contact with Id '{request.Id}' was not found."));

            var response = new Response(
                entity.Id,
                entity.TenantId,
                entity.FullName,
                entity.Email,
                entity.Phone,
                entity.Company
            );

            return Result<Response, DomainError>.Success(response);
        }
    }
}
