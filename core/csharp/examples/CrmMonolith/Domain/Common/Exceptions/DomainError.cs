namespace CrmMonolith.Domain.Common.Exceptions;

using System.Collections.Generic;

public sealed record DomainError(string Code, string Message, IDictionary<string, object?>? Metadata = null);
