using System;
using System.Collections.Generic;
using System.Linq.Expressions;

namespace EticaretApp.Application.Common.Functional;

public record Specification<T>
{
    public Expression<Func<T, bool>>? Predicate { get; init; }
    public List<Expression<Func<T, object>>> Includes { get; init; } = [];
    public int? Take { get; init; }
    public int? Skip { get; init; }
    public Expression<Func<T, object>>? OrderBy { get; init; }
    public Expression<Func<T, object>>? OrderByDescending { get; init; }
}
