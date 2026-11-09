namespace AotCrud.Domain;

/// <summary>Demo port: a real implementation would query the product catalog.</summary>
public interface IProductCatalogService
{
    Task<bool> HasActiveProductsAsync(int categoryId, string tenantId, CancellationToken cancellationToken = default);
}

public sealed class DemoProductCatalog : IProductCatalogService
{
    public Task<bool> HasActiveProductsAsync(int categoryId, string tenantId,
        CancellationToken cancellationToken = default) =>
        Task.FromResult(categoryId == 1);
}
