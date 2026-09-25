using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.EntityFrameworkCore;
using EticaretApp.Application.Common.Functional;

namespace EticaretApp.Persistence.Common.Functional;

public class QueryHandler<T, TResult>(DbContext context) where T : class
{
    public async Task<IEnumerable<TResult>> HandleAsync(
        Query<T, TResult> query,
        CancellationToken cancellationToken = default)
    {
        var items = await query.Apply(context.Set<T>()).ToListAsync(cancellationToken);
        return items.Select(query.Map);
    }
}
