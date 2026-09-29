namespace CrmMonolith.Modules.Activity;

using CrmMonolith.Common.Pipeline;
using CrmMonolith.Common.Functional;
using CrmMonolith.Domain.Common.Exceptions;
using CrmMonolith.Domain.Entities;
using CrmMonolith.Data;

public static class Update
{
    public sealed record Command(
        string Id,
        string TenantId,
        string Title,
        string Type,
        DateTime DueDate,
        string Notes,
        string DealId,
        string ContactId
    ) : ICommand<Result<Response, DomainError>>;

    public sealed record Response(string Id);

    internal sealed class Handler(CrmDbContext db) :
        ICommandHandler<Command, Result<Response, DomainError>>
    {
        public async ValueTask<Result<Response, DomainError>> HandleAsync(Command request, CancellationToken ct)
        {
            var entity = await db.Activities.FindAsync(new object[] { request.Id }, ct);
            if (entity is null)
                return Result<Response, DomainError>.Failure(new DomainError("Activity.NotFound", $"Activity with Id '{request.Id}' was not found."));

            entity.TenantId = request.TenantId;
            entity.Title = request.Title;
            entity.Type = request.Type;
            entity.DueDate = request.DueDate;
            entity.Notes = request.Notes;
            entity.DealId = request.DealId;
            entity.ContactId = request.ContactId;

            db.Activities.Update(entity);
            await db.SaveChangesAsync(ct);

            return Result<Response, DomainError>.Success(new Response(entity.Id));
        }
    }
}
