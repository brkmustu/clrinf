namespace CrmMonolith.Modules.Activity;

using CrmMonolith.Common.Pipeline;
using CrmMonolith.Common.Functional;
using CrmMonolith.Domain.Common.Exceptions;
using CrmMonolith.Domain.Entities;
using CrmMonolith.Data;

public static class Create
{
    public sealed record Command(
        string TenantId,
        string Title,
        string Type,
        DateTime DueDate,
        string? Notes = null,
        string? DealId = null,
        string? ContactId = null
    ) : ICommand<Result<Response, DomainError>>;

    public sealed record Response(string Id);

    internal sealed class Handler(CrmDbContext db) :
        ICommandHandler<Command, Result<Response, DomainError>>
    {
        public async ValueTask<Result<Response, DomainError>> HandleAsync(Command request, CancellationToken ct)
        {
            var entity = new Activity
            {
                TenantId = request.TenantId,
                Title = request.Title,
                Type = request.Type,
                DueDate = request.DueDate,
                Notes = request.Notes,
                DealId = request.DealId,
                ContactId = request.ContactId,
            };

            db.Activities.Add(entity);
            await db.SaveChangesAsync(ct);

            return Result<Response, DomainError>.Success(new Response(entity.Id));
        }
    }
}
