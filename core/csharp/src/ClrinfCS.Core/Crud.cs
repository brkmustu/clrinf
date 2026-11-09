namespace ClrinfCS.Core;

/// <summary>CRUD operations that can be generated for an entity.</summary>
[Flags]
public enum CrudOperations
{
    None = 0,
    Create = 1,
    Update = 2,
    Delete = 4,
    GetById = 8,
    GetList = 16,
    All = Create | Update | Delete | GetById | GetList
}

/// <summary>
/// Dispatch strategy used by generated CRUD endpoints.
/// <see cref="Default"/> is an "unset" sentinel: the attribute, then the
/// <c>ClrinfCrudDispatcher</c> MSBuild property, then <see cref="Native"/> wins.
/// </summary>
public enum CrudDispatcher
{
    Default = 0,
    Native = 1,
    MediatorNet = 2
}

/// <summary>
/// Marks an entity for compile-time CRUD handler/endpoint generation.
/// Only the entity, hook rules and explicitly ejected operations live on disk.
/// </summary>
[AttributeUsage(AttributeTargets.Class, Inherited = false, AllowMultiple = false)]
public sealed class GenerateCrudEndpointsAttribute : Attribute
{
    public CrudOperations Operations { get; set; } = CrudOperations.All;
    public string? RoutePrefix { get; set; }
    public CrudDispatcher Dispatcher { get; set; } = CrudDispatcher.Default;
}

/// <summary>Excludes a property from generated request/response DTOs.</summary>
[AttributeUsage(AttributeTargets.Property, Inherited = false, AllowMultiple = false)]
public sealed class CrudIgnoreAttribute : Attribute;

/// <summary>
/// Marker for entities whose rows are isolated by tenant. The EF adapter scopes
/// every read/write through <see cref="RequestContext.TenantId"/>.
/// </summary>
public interface ITenantScoped
{
    string TenantId { get; set; }
}

/// <summary>
/// Thrown by a store when an optimistic-concurrency check fails. Endpoints map
/// this to HTTP 409.
/// </summary>
public sealed class CrudConflictException(string message) : InvalidOperationException(message);

/// <summary>Paging/sorting request. Page numbers are 1-based; page size is bounded.</summary>
public sealed record CrudQuery
{
    public const int DefaultPageSize = 20;
    public const int MaxPageSize = 200;

    public CrudQuery(int page = 1, int pageSize = DefaultPageSize, string? sort = null)
    {
        Page = page < 1 ? 1 : page;
        PageSize = pageSize < 1 ? DefaultPageSize : (pageSize > MaxPageSize ? MaxPageSize : pageSize);
        Sort = string.IsNullOrWhiteSpace(sort) ? null : sort;
    }

    public int Page { get; }
    public int PageSize { get; }
    public string? Sort { get; }
    public int Skip => (Page - 1) * PageSize;
}

/// <summary>A page of results together with the total row count.</summary>
public sealed record PagedResult<T>(
    IReadOnlyList<T> Items, int Page, int PageSize, int TotalCount)
{
    public int TotalPages => PageSize <= 0 ? 0 : (TotalCount + PageSize - 1) / PageSize;

    public static PagedResult<T> Empty(int page, int pageSize) =>
        new(Array.Empty<T>(), page, pageSize, 0);
}

/// <summary>
/// Metadata attached to generated Minimal API endpoints so OpenAPI generators
/// can discover route and query parameters without delegate reflection.
/// </summary>
public sealed record CrudEndpointParameterMetadata(
    string Name,
    string Location,
    Type ParameterType,
    bool Required) : System.Runtime.CompilerServices.ITuple
{
    int System.Runtime.CompilerServices.ITuple.Length => 4;

    object? System.Runtime.CompilerServices.ITuple.this[int index] => index switch
    {
        0 => Name,
        1 => Location,
        2 => ParameterType,
        3 => Required,
        _ => null
    };
}

/// <summary>
/// Persistence port for generated CRUD handlers. Implementations own tenant
/// scoping; every operation receives the trusted <see cref="RequestContext"/>.
/// </summary>
public interface ICrudStore<TEntity, in TKey> where TEntity : class
{
    ValueTask<TEntity?> GetByIdAsync(TKey id, RequestContext context, CancellationToken cancellationToken = default);

    ValueTask<PagedResult<TEntity>> GetListAsync(CrudQuery query, RequestContext context,
        CancellationToken cancellationToken = default);

    ValueTask<TEntity> AddAsync(TEntity entity, RequestContext context,
        CancellationToken cancellationToken = default);

    ValueTask<TEntity?> UpdateAsync(TEntity entity, RequestContext context,
        CancellationToken cancellationToken = default);

    ValueTask<bool> DeleteAsync(TEntity entity, RequestContext context,
        CancellationToken cancellationToken = default);
}

/// <summary>Hook evaluated with the fully loaded entity before an insert.</summary>
public interface IBeforeCreateRule<in TEntity> where TEntity : class
{
    int Priority => 0;

    ValueTask<RuleResult> EvaluateAsync(TEntity entity, RequestContext requestContext,
        CancellationToken cancellationToken = default);
}

/// <summary>Hook evaluated with the fully loaded entity before an update.</summary>
public interface IBeforeUpdateRule<in TEntity> where TEntity : class
{
    int Priority => 0;

    ValueTask<RuleResult> EvaluateAsync(TEntity entity, RequestContext requestContext,
        CancellationToken cancellationToken = default);
}

/// <summary>Hook evaluated with the fully loaded entity before a delete.</summary>
public interface IBeforeDeleteRule<in TEntity> where TEntity : class
{
    int Priority => 0;

    ValueTask<RuleResult> EvaluateAsync(TEntity entity, RequestContext requestContext,
        CancellationToken cancellationToken = default);
}

/// <summary>
/// Single execution point for entity hooks. Ordering, short-circuiting and the
/// translation to <see cref="BusinessRuleViolationException"/> live here, so
/// generated code and both dispatchers behave identically.
/// </summary>
public static class CrudRuleRunner
{
    public static ValueTask RunBeforeCreateAsync<TEntity>(IEnumerable<IBeforeCreateRule<TEntity>> rules,
        TEntity entity, RequestContext context, CancellationToken cancellationToken = default)
        where TEntity : class =>
        RunAsync(rules, static (rule, e, ctx, ct) => rule.EvaluateAsync(e, ctx, ct),
            entity, context, cancellationToken);

    public static ValueTask RunBeforeUpdateAsync<TEntity>(IEnumerable<IBeforeUpdateRule<TEntity>> rules,
        TEntity entity, RequestContext context, CancellationToken cancellationToken = default)
        where TEntity : class =>
        RunAsync(rules, static (rule, e, ctx, ct) => rule.EvaluateAsync(e, ctx, ct),
            entity, context, cancellationToken);

    public static ValueTask RunBeforeDeleteAsync<TEntity>(IEnumerable<IBeforeDeleteRule<TEntity>> rules,
        TEntity entity, RequestContext context, CancellationToken cancellationToken = default)
        where TEntity : class =>
        RunAsync(rules, static (rule, e, ctx, ct) => rule.EvaluateAsync(e, ctx, ct),
            entity, context, cancellationToken);

    private static async ValueTask RunAsync<TRule, TEntity>(IEnumerable<TRule> rules,
        Func<TRule, TEntity, RequestContext, CancellationToken, ValueTask<RuleResult>> evaluate,
        TEntity entity, RequestContext context, CancellationToken cancellationToken)
        where TRule : class
        where TEntity : class
    {
        ArgumentNullException.ThrowIfNull(rules);
        ArgumentNullException.ThrowIfNull(entity);
        ArgumentNullException.ThrowIfNull(context);
        cancellationToken.ThrowIfCancellationRequested();

        foreach (var rule in rules.OrderBy(static r => r is IBeforeCreateRule<TEntity> c ? c.Priority
                     : r is IBeforeUpdateRule<TEntity> u ? u.Priority
                     : r is IBeforeDeleteRule<TEntity> d ? d.Priority
                     : 0))
        {
            var result = await evaluate(rule, entity, context, cancellationToken).ConfigureAwait(false);
            if (!result.IsSuccess)
            {
                throw new BusinessRuleViolationException(
                    result.ErrorCode ?? "RULE_VIOLATION",
                    result.Message ?? "Business rule violation.");
            }
        }
    }
}
