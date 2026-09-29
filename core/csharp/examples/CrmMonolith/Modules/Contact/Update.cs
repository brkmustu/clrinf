namespace CrmMonolith.Modules.Contact;

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
        string FullName,
        string Email,
        string Phone,
        string Company
    ) : ICommand<Result<Response, DomainError>>;

    public sealed record Response(string Id);

    internal sealed class Handler(CrmDbContext db) :
        ICommandHandler<Command, Result<Response, DomainError>>
    {
        public async ValueTask<Result<Response, DomainError>> HandleAsync(Command request, CancellationToken ct)
        {
            var entity = await db.Contacts.FindAsync(new object[] { request.Id }, ct);
            if (entity is null)
                return Result<Response, DomainError>.Failure(new DomainError("Contact.NotFound", $"Contact with Id '{request.Id}' was not found."));

            entity.TenantId = request.TenantId;
            entity.FullName = request.FullName;
            entity.Email = request.Email;
            entity.Phone = request.Phone;
            entity.Company = request.Company;

            db.Contacts.Update(entity);
            await db.SaveChangesAsync(ct);

            return Result<Response, DomainError>.Success(new Response(entity.Id));
        }
    }
}
