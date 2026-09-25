using System.Collections.Generic;

namespace EticaretApp.Domain.Common.Entities;

public readonly record struct DomainResult<TState>(
    TState State,
    IReadOnlyList<object> Events);
