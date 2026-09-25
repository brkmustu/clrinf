using System;
using System.Collections.Generic;
using System.Linq;
using System.Linq.Expressions;
using Microsoft.EntityFrameworkCore.Query;
using EticaretApp.Domain.Common.Entities;
using EticaretApp.Application.Common.Paging;

namespace EticaretApp.Application.Common.Repositories;

public interface IRepository<TEntity, TId> where TEntity : class, IEntity<TId>, new()
{
    TEntity? Get(
        Expression<Func<TEntity, bool>> predicate,
        Func<IQueryable<TEntity>, IIncludableQueryable<TEntity, object>>? include = null,
        bool withDeleted = false,
        bool enableTracking = true
    );

    IPaginate<TEntity> GetList(
        Expression<Func<TEntity, bool>>? predicate = null,
        Func<IQueryable<TEntity>, IOrderedQueryable<TEntity>>? orderBy = null,
        Func<IQueryable<TEntity>, IIncludableQueryable<TEntity, object>>? include = null,
        int index = 0,
        int size = 10,
        bool withDeleted = false,
        bool enableTracking = true
    );

    TEntity Add(TEntity entity);
    TEntity Update(TEntity entity);
    TEntity Delete(TEntity entity, bool permanent = false);
}
