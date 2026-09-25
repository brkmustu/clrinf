using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.EntityFrameworkCore;
using EticaretApp.Application.Common.Functional;

namespace EticaretApp.Persistence.Common.Functional;

public class QueryExecutor<T>(DbContext context) where T : class
{
    public async Task<IEnumerable<TResult>> ExecuteAsync<TResult>(
        Specification<T> spec,
        Func<T, TResult> selector,
        CancellationToken cancellationToken = default)
    {
        var query = context.Set<T>().AsQueryable();

        if (spec.Predicate != null)
            query = query.Where(spec.Predicate);

        foreach (var include in spec.Includes)
            query = query.Include(include);

        if (spec.OrderBy != null)
            query = query.OrderBy(spec.OrderBy);

        if (spec.OrderByDescending != null)
            query = query.OrderByDescending(spec.OrderByDescending);

        if (spec.Skip.HasValue)
            query = query.Skip(spec.Skip.Value);

        if (spec.Take.HasValue)
            query = query.Take(spec.Take.Value);

        var items = await query.ToListAsync(cancellationToken);
        return items.Select(selector);
    }
}
