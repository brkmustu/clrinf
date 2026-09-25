using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.EntityFrameworkCore;

namespace EticaretApp.Application.Common.Paging;

public static class QueryablePaginateExtensions
{
    public static async Task<IPaginate<T>> ToPaginateAsync<T>(
        this IQueryable<T> source,
        int index,
        int size,
        int from = 0,
        CancellationToken cancellationToken = default)
    {
        int count = await source.CountAsync(cancellationToken).ConfigureAwait(false);
        List<T> items = await source.Skip((index - from) * size).Take(size).ToListAsync(cancellationToken).ConfigureAwait(false);

        return new Paginate<T>
        {
            Index = index,
            Size = size,
            From = from,
            Count = count,
            Items = items,
            Pages = (int)Math.Ceiling(count / (double)size)
        };
    }
}
