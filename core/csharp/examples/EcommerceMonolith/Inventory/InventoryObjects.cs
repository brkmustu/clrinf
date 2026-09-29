using ClrinfCS.Core;
using EcommerceMonolith.Common.Functional;
using EcommerceMonolith.Domain.Common.Exceptions;
using EcommerceMonolith.Security;

namespace EcommerceMonolith.Inventory;

public sealed record StockItem(
    string ProductId,
    string TenantId,
    int AvailableQuantity,
    int ReservedQuantity
);

public sealed record SetStockCommand(
    string TenantId,
    string ProductId,
    int InitialQuantity
) : IRequest<Result<StockItem, DomainError>>, IRequireOperationClaim
{
    public OperationClaim GetRequiredClaim() =>
        OperationClaim.WithTenant("Inventory", "inventory.reserve", TenantId);
}

public sealed record ReserveStockCommand(
    string TenantId,
    string ProductId,
    int Quantity
) : IRequest<Result<StockItem, DomainError>>, IRequireOperationClaim
{
    public OperationClaim GetRequiredClaim() =>
        OperationClaim.WithTenant("Inventory", "inventory.reserve", TenantId);
}

public sealed record ReleaseStockCommand(
    string TenantId,
    string ProductId,
    int Quantity
) : IRequest<Result<StockItem, DomainError>>, IRequireOperationClaim
{
    public OperationClaim GetRequiredClaim() =>
        OperationClaim.WithTenant("Inventory", "inventory.release", TenantId);
}

public sealed record CommitStockCommand(
    string TenantId,
    string ProductId,
    int Quantity
) : IRequest<Result<StockItem, DomainError>>, IRequireOperationClaim
{
    public OperationClaim GetRequiredClaim() =>
        OperationClaim.WithTenant("Inventory", "inventory.commit", TenantId);
}

public sealed record GetStockQuery(
    string TenantId,
    string ProductId
) : IRequest<Result<StockItem, DomainError>>, IRequireOperationClaim
{
    public OperationClaim GetRequiredClaim() =>
        OperationClaim.WithTenant("Inventory", "inventory.read", TenantId);
}

public interface IInventoryRepository
{
    ValueTask<StockItem?> GetByProductIdAsync(string tenantId, string productId);
    ValueTask<StockItem> SaveAsync(StockItem item);
}

public sealed class InMemoryInventoryRepository : IInventoryRepository
{
    private readonly Dictionary<string, StockItem> _storage = new();

    public ValueTask<StockItem?> GetByProductIdAsync(string tenantId, string productId)
    {
        _storage.TryGetValue($"{tenantId}:{productId}", out var item);
        return ValueTask.FromResult(item);
    }

    public ValueTask<StockItem> SaveAsync(StockItem item)
    {
        _storage[$"{item.TenantId}:{item.ProductId}"] = item;
        return ValueTask.FromResult(item);
    }
}
