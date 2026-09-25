using System.Linq;

namespace EticaretApp.Application.Common.Functional;

public abstract record Query<T, TResult>
{
    public abstract IQueryable<T> Apply(IQueryable<T> query);
    public abstract TResult Map(T entity);
}
