namespace EticaretApp.Domain.Common.Exceptions;

public abstract record DomainError(string Code, string Message)
{
    public sealed record Validation(string Field, string Reason)
        : DomainError($"validation.{Field}", Reason);

    public sealed record NotFound(string Entity, string Id)
        : DomainError($"not_found.{Entity}", $"{Entity} '{Id}' not found.");

    public sealed record Conflict(string Reason)
        : DomainError("conflict", Reason);
}
