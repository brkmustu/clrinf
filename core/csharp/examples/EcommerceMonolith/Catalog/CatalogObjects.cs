using ClrinfCS.Core;
using EcommerceMonolith.Common.Functional;
using EcommerceMonolith.Domain.Common.Exceptions;
using EcommerceMonolith.Security;

namespace EcommerceMonolith.Catalog;

public sealed record Product(
    string Id,
    string TenantId,
    string Sku,
    string Name,
    decimal Price,
    string Status
);

public sealed record CreateProductCommand(
    string TenantId,
    string Sku,
    string Name,
    decimal Price
) : IRequest<Result<Product, DomainError>>, IRequireOperationClaim
{
    public OperationClaim GetRequiredClaim() =>
        OperationClaim.WithTenant("Catalog", "catalog.create", TenantId);
}

public sealed record GetProductQuery(
    string TenantId,
    string ProductId
) : IRequest<Result<Product, DomainError>>, IRequireOperationClaim
{
    public OperationClaim GetRequiredClaim() =>
        OperationClaim.WithTenant("Catalog", "catalog.read", TenantId);
}

public interface ICatalogRepository
{
    ValueTask<Product?> GetByIdAsync(string tenantId, string id);
    ValueTask<Product?> GetBySkuAsync(string tenantId, string sku);
    ValueTask<Product> SaveAsync(Product product);
}

public sealed class InMemoryCatalogRepository : ICatalogRepository
{
    private readonly Dictionary<string, Product> _storage = new();

    public ValueTask<Product?> GetByIdAsync(string tenantId, string id)
    {
        _storage.TryGetValue($"{tenantId}:{id}", out var product);
        return ValueTask.FromResult(product);
    }

    public ValueTask<Product?> GetBySkuAsync(string tenantId, string sku)
    {
        var found = _storage.Values.FirstOrDefault(p => p.TenantId == tenantId && p.Sku.Equals(sku, StringComparison.OrdinalIgnoreCase));
        return ValueTask.FromResult(found);
    }

    public ValueTask<Product> SaveAsync(Product product)
    {
        _storage[$"{product.TenantId}:{product.Id}"] = product;
        return ValueTask.FromResult(product);
    }
}
