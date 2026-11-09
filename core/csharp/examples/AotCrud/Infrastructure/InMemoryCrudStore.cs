using ClrinfCS.Core;

namespace AotCrud.Infrastructure;

/// <summary>
/// Tiny in-memory adapter for the demo. A real application swaps this for
/// <c>EfCrudStore&lt;TEntity,TKey,TDbContext&gt;</c>.
/// </summary>
public sealed class InMemoryCrudStore<TEntity, TKey>(
    Func<TEntity, TKey> key,
    Action<TEntity, string> setTenant,
    Func<TEntity, string> getTenant) : ICrudStore<TEntity, TKey>
    where TEntity : class
{
    private readonly List<TEntity> items = [];
    private readonly Lock gate = new();

    public ValueTask<TEntity?> GetByIdAsync(TKey id, RequestContext context,
        CancellationToken cancellationToken = default)
    {
        lock (gate)
        {
            return ValueTask.FromResult(items.FirstOrDefault(e =>
                EqualityComparer<TKey>.Default.Equals(key(e), id) &&
                getTenant(e) == context.TenantId));
        }
    }

    public ValueTask<PagedResult<TEntity>> GetListAsync(CrudQuery query, RequestContext context,
        CancellationToken cancellationToken = default)
    {
        lock (gate)
        {
            var scoped = items.Where(e => getTenant(e) == context.TenantId).ToArray();
            var page = scoped.Skip(query.Skip).Take(query.PageSize).ToArray();
            return ValueTask.FromResult(
                new PagedResult<TEntity>(page, query.Page, query.PageSize, scoped.Length));
        }
    }

    public ValueTask<TEntity> AddAsync(TEntity entity, RequestContext context,
        CancellationToken cancellationToken = default)
    {
        setTenant(entity, context.TenantId);
        lock (gate)
        {
            items.Add(entity);
        }

        return ValueTask.FromResult(entity);
    }

    public ValueTask<TEntity?> UpdateAsync(TEntity entity, RequestContext context,
        CancellationToken cancellationToken = default)
    {
        setTenant(entity, context.TenantId);
        lock (gate)
        {
            var index = items.FindIndex(e =>
                EqualityComparer<TKey>.Default.Equals(key(e), key(entity)) &&
                getTenant(e) == context.TenantId);
            if (index < 0) return ValueTask.FromResult<TEntity?>(null);
            items[index] = entity;
            return ValueTask.FromResult<TEntity?>(entity);
        }
    }

    public ValueTask<bool> DeleteAsync(TEntity entity, RequestContext context,
        CancellationToken cancellationToken = default)
    {
        lock (gate)
        {
            var removed = items.RemoveAll(e =>
                EqualityComparer<TKey>.Default.Equals(key(e), key(entity)) &&
                getTenant(e) == context.TenantId);
            return ValueTask.FromResult(removed > 0);
        }
    }
}
