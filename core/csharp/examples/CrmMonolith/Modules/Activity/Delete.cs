namespace CrmMonolith.Modules.Activity;

using CrmMonolith.Common.Pipeline;
using CrmMonolith.Common.Functional;
using CrmMonolith.Domain.Common.Exceptions;
using CrmMonolith.Domain.Entities;
using CrmMonolith.Data;

public static class Delete
{
    public sealed record Command(string Id) : ICommand<Result<Response, DomainError>>;

    public sealed record Response(string Id);

    internal sealed class Handler(CrmDbContext db) :
        ICommandHandler<Command, Result<Response, DomainError>>
    {
        public async ValueTask<Result<Response, DomainError>> HandleAsync(Command request, CancellationToken ct)
        {
            var entity = await db.Activities.FindAsync(new object[] { request.Id }, ct);
            if (entity is null)
                return Result<Response, DomainError>.Failure(new DomainError("Activity.NotFound", $"Activity with Id '{request.Id}' was not found."));

            db.Activities.Remove(entity);
            await db.SaveChangesAsync(ct);

            return Result<Response, DomainError>.Success(new Response(request.Id));
        }
    }
}
