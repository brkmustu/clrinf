using System;
using System.Collections.Generic;
using System.Linq;
using System.Linq.Expressions;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Query;
using EticaretApp.Domain.Common.Entities;
using EticaretApp.Application.Common.Paging;
using EticaretApp.Application.Common.Repositories;

namespace EticaretApp.Persistence.Common;

public class EfRepositoryBase<TEntity, TId, TContext> : IAsyncRepository<TEntity, TId>, IRepository<TEntity, TId>
    where TEntity : class, IEntity<TId>, new()
    where TContext : DbContext
{
    protected readonly TContext Context;

    public EfRepositoryBase(TContext context)
    {
        Context = context;
    }

    public IQueryable<TEntity> Query() => Context.Set<TEntity>();

    public async Task<TEntity?> GetAsync(
        Expression<Func<TEntity, bool>> predicate,
        Func<IQueryable<TEntity>, IIncludableQueryable<TEntity, object>>? include = null,
        bool withDeleted = false,
        bool enableTracking = true,
        CancellationToken cancellationToken = default
    )
    {
        IQueryable<TEntity> queryable = Query();
        if (!enableTracking)
            queryable = queryable.AsNoTracking();
        if (include != null)
            queryable = include(queryable);
        if (withDeleted)
            queryable = queryable.IgnoreQueryFilters();
        return await queryable.FirstOrDefaultAsync(predicate, cancellationToken);
    }

    public async Task<IPaginate<TEntity>> GetListAsync(
        Expression<Func<TEntity, bool>>? predicate = null,
        Func<IQueryable<TEntity>, IOrderedQueryable<TEntity>>? orderBy = null,
        Func<IQueryable<TEntity>, IIncludableQueryable<TEntity, object>>? include = null,
        int index = 0,
        int size = 10,
        bool withDeleted = false,
        bool enableTracking = true,
        CancellationToken cancellationToken = default
    )
    {
        IQueryable<TEntity> queryable = Query();
        if (!enableTracking)
            queryable = queryable.AsNoTracking();
        if (include != null)
            queryable = include(queryable);
        if (withDeleted)
            queryable = queryable.IgnoreQueryFilters();
        if (predicate != null)
            queryable = queryable.Where(predicate);
        if (orderBy != null)
            return await orderBy(queryable).ToPaginateAsync(index, size, 0, cancellationToken);
        return await queryable.ToPaginateAsync(index, size, 0, cancellationToken);
    }

    public async Task<TEntity> AddAsync(TEntity entity)
    {
        entity.CreatedDate = DateTime.UtcNow;
        Context.Entry(entity).State = EntityState.Added;
        await Context.SaveChangesAsync();
        return entity;
    }

    public async Task<TEntity> UpdateAsync(TEntity entity)
    {
        entity.UpdatedDate = DateTime.UtcNow;
        Context.Entry(entity).State = EntityState.Modified;
        await Context.SaveChangesAsync();
        return entity;
    }

    public async Task<TEntity> DeleteAsync(TEntity entity, bool permanent = false)
    {
        if (permanent)
        {
            Context.Entry(entity).State = EntityState.Deleted;
        }
        else
        {
            entity.DeletedDate = DateTime.UtcNow;
            Context.Entry(entity).State = EntityState.Modified;
        }
        await Context.SaveChangesAsync();
        return entity;
    }

    public TEntity? Get(
        Expression<Func<TEntity, bool>> predicate,
        Func<IQueryable<TEntity>, IIncludableQueryable<TEntity, object>>? include = null,
        bool withDeleted = false,
        bool enableTracking = true
    )
    {
        IQueryable<TEntity> queryable = Query();
        if (!enableTracking)
            queryable = queryable.AsNoTracking();
        if (include != null)
            queryable = include(queryable);
        if (withDeleted)
            queryable = queryable.IgnoreQueryFilters();
        return queryable.FirstOrDefault(predicate);
    }

    public IPaginate<TEntity> GetList(
        Expression<Func<TEntity, bool>>? predicate = null,
        Func<IQueryable<TEntity>, IOrderedQueryable<TEntity>>? orderBy = null,
        Func<IQueryable<TEntity>, IIncludableQueryable<TEntity, object>>? include = null,
        int index = 0,
        int size = 10,
        bool withDeleted = false,
        bool enableTracking = true
    )
    {
        IQueryable<TEntity> queryable = Query();
        if (!enableTracking)
            queryable = queryable.AsNoTracking();
        if (include != null)
            queryable = include(queryable);
        if (withDeleted)
            queryable = queryable.IgnoreQueryFilters();
        if (predicate != null)
            queryable = queryable.Where(predicate);
        
        var list = orderBy != null ? orderBy(queryable).ToList() : queryable.ToList();
        return new Paginate<TEntity>(list, index, size);
    }

    public TEntity Add(TEntity entity)
    {
        entity.CreatedDate = DateTime.UtcNow;
        Context.Entry(entity).State = EntityState.Added;
        Context.SaveChanges();
        return entity;
    }

    public TEntity Update(TEntity entity)
    {
        entity.UpdatedDate = DateTime.UtcNow;
        Context.Entry(entity).State = EntityState.Modified;
        Context.SaveChanges();
        return entity;
    }

    public TEntity Delete(TEntity entity, bool permanent = false)
    {
        if (permanent)
        {
            Context.Entry(entity).State = EntityState.Deleted;
        }
        else
        {
            entity.DeletedDate = DateTime.UtcNow;
            Context.Entry(entity).State = EntityState.Modified;
        }
        Context.SaveChanges();
        return entity;
    }
}
