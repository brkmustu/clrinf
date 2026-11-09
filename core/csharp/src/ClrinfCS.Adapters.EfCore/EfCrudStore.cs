using System.Diagnostics.CodeAnalysis;
using System.Linq.Expressions;
using ClrinfCS.Core;
using Microsoft.EntityFrameworkCore;

namespace ClrinfCS.Adapters.EfCore;

/// <summary>
/// Provider-independent EF Core implementation of <see cref="ICrudStore{TEntity,TKey}"/>.
/// Every read and write is scoped to <see cref="RequestContext.TenantId"/> when the
/// entity implements <see cref="ITenantScoped"/>.
/// </summary>
public class EfCrudStore<
    [DynamicallyAccessedMembers(DynamicallyAccessedMemberTypes.PublicProperties)] TEntity,
    TKey,
    TDbContext> : ICrudStore<TEntity, TKey>
    where TEntity : class
    where TDbContext : DbContext
{
    private const string IdProperty = "Id";
    private readonly TDbContext dbContext;

    public EfCrudStore(TDbContext dbContext) =>
        this.dbContext = dbContext ?? throw new ArgumentNullException(nameof(dbContext));

    public async ValueTask<TEntity?> GetByIdAsync(TKey id, RequestContext context,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(context);
        return await Query(context)
            .FirstOrDefaultAsync(e => EF.Property<TKey>(e, IdProperty)!.Equals(id), cancellationToken)
            .ConfigureAwait(false);
    }

    public async ValueTask<PagedResult<TEntity>> GetListAsync(CrudQuery query, RequestContext context,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(query);
        ArgumentNullException.ThrowIfNull(context);

        var source = Query(context).AsNoTracking();
        source = ApplySort(source, query.Sort);

        var total = await source.CountAsync(cancellationToken).ConfigureAwait(false);
        var items = await source
            .Skip(query.Skip)
            .Take(query.PageSize)
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);

        return new PagedResult<TEntity>(items, query.Page, query.PageSize, total);
    }

    public async ValueTask<TEntity> AddAsync(TEntity entity, RequestContext context,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(entity);
        ArgumentNullException.ThrowIfNull(context);

        ApplyTenant(entity, context);
        dbContext.Set<TEntity>().Add(entity);
        await dbContext.SaveChangesAsync(cancellationToken).ConfigureAwait(false);
        return entity;
    }

    public async ValueTask<TEntity?> UpdateAsync(TEntity entity, RequestContext context,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(entity);
        ArgumentNullException.ThrowIfNull(context);

        ApplyTenant(entity, context);
        dbContext.Set<TEntity>().Update(entity);
        try
        {
            await dbContext.SaveChangesAsync(cancellationToken).ConfigureAwait(false);
        }
        catch (DbUpdateConcurrencyException)
        {
            throw new CrudConflictException(
                "The record was modified by another request. Reload and retry.");
        }

        return entity;
    }

    public async ValueTask<bool> DeleteAsync(TEntity entity, RequestContext context,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(entity);
        ArgumentNullException.ThrowIfNull(context);

        dbContext.Set<TEntity>().Remove(entity);
        try
        {
            await dbContext.SaveChangesAsync(cancellationToken).ConfigureAwait(false);
        }
        catch (DbUpdateConcurrencyException)
        {
            throw new CrudConflictException(
                "The record was modified by another request. Reload and retry.");
        }

        return true;
    }

    private IQueryable<TEntity> Query(RequestContext context)
    {
        IQueryable<TEntity> query = dbContext.Set<TEntity>();
        if (typeof(ITenantScoped).IsAssignableFrom(typeof(TEntity)))
        {
            var tenantId = context.TenantId;
            query = query.Where(e => EF.Property<string>(e, nameof(ITenantScoped.TenantId)) == tenantId);
        }

        return query;
    }

    private static void ApplyTenant(TEntity entity, RequestContext context)
    {
        if (entity is ITenantScoped scoped)
        {
            scoped.TenantId = context.TenantId;
        }
    }

    private static IQueryable<TEntity> ApplySort(IQueryable<TEntity> source, string? sort)
    {
        if (string.IsNullOrWhiteSpace(sort))
        {
            return source;
        }

        var descending = sort[0] == '-';
        var propertyName = descending ? sort[1..] : sort;
        if (string.IsNullOrWhiteSpace(propertyName))
        {
            return source;
        }

        var property = typeof(TEntity).GetProperty(propertyName,
            System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Instance |
            System.Reflection.BindingFlags.IgnoreCase);
        if (property is null)
        {
            return source;
        }

        var parameter = Expression.Parameter(typeof(TEntity), "entity");
        var body = Expression.Property(parameter, property);
        var lambda = Expression.Lambda(body, parameter);
        var method = (descending ? nameof(Queryable.OrderByDescending) : nameof(Queryable.OrderBy));
        var call = Expression.Call(
            typeof(Queryable),
            method,
            new[] { typeof(TEntity), property.PropertyType },
            source.Expression,
            Expression.Quote(lambda));

        return source.Provider.CreateQuery<TEntity>(call);
    }
}
