namespace CrmMonolith.Modules.Contact;

using CrmMonolith.Common.Pipeline;
using CrmMonolith.Common.Functional;
using CrmMonolith.Domain.Common.Exceptions;
using CrmMonolith.Domain.Entities;
using CrmMonolith.Data;

public static class Create
{
    public sealed record Command(
        string TenantId,
        string FullName,
        string Email,
        string? Phone = null,
        string? Company = null
    ) : ICommand<Result<Response, DomainError>>;

    public sealed record Response(string Id);

    internal sealed class Handler(CrmDbContext db) :
        ICommandHandler<Command, Result<Response, DomainError>>
    {
        public async ValueTask<Result<Response, DomainError>> HandleAsync(Command request, CancellationToken ct)
        {
            var entity = new Contact
            {
                TenantId = request.TenantId,
                FullName = request.FullName,
                Email = request.Email,
                Phone = request.Phone,
                Company = request.Company,
            };

            db.Contacts.Add(entity);
            await db.SaveChangesAsync(ct);

            return Result<Response, DomainError>.Success(new Response(entity.Id));
        }
    }
}
