using ClrinfCS.Core;
using EcommerceMonolith.Common.Functional;
using EcommerceMonolith.Domain.Common.Exceptions;
using EcommerceMonolith.Security;

namespace EcommerceMonolith.Orders;

public sealed record OrderItem(
    string ProductId,
    int Quantity,
    decimal UnitPrice
);

public sealed record Order(
    string Id,
    string TenantId,
    string CustomerId,
    IReadOnlyList<OrderItem> Items,
    decimal TotalAmount,
    string Status
);

public sealed record CreateOrderCommand(
    string TenantId,
    string CustomerId,
    IReadOnlyList<OrderItem> Items
) : IRequest<Result<Order, DomainError>>, IRequireOperationClaim
{
    public OperationClaim GetRequiredClaim() =>
        OperationClaim.WithTenant("Orders", "orders.create", TenantId);
}

public sealed record CancelOrderCommand(
    string TenantId,
    string OrderId
) : IRequest<Result<Order, DomainError>>, IRequireOperationClaim
{
    public OperationClaim GetRequiredClaim() =>
        OperationClaim.WithTenant("Orders", "orders.cancel", TenantId);
}

public sealed record CompleteOrderCommand(
    string TenantId,
    string OrderId
) : IRequest<Result<Order, DomainError>>, IRequireOperationClaim
{
    public OperationClaim GetRequiredClaim() =>
        OperationClaim.WithTenant("Orders", "orders.create", TenantId);
}

public sealed record GetOrderQuery(
    string TenantId,
    string OrderId
) : IRequest<Result<Order, DomainError>>, IRequireOperationClaim
{
    public OperationClaim GetRequiredClaim() =>
        OperationClaim.WithTenant("Orders", "orders.read", TenantId);
}

public interface IOrderRepository
{
    ValueTask<Order?> GetByIdAsync(string tenantId, string id);
    ValueTask<Order> SaveAsync(Order order);
}

public sealed class InMemoryOrderRepository : IOrderRepository
{
    private readonly Dictionary<string, Order> _storage = new();

    public ValueTask<Order?> GetByIdAsync(string tenantId, string id)
    {
        _storage.TryGetValue($"{tenantId}:{id}", out var order);
        return ValueTask.FromResult(order);
    }

    public ValueTask<Order> SaveAsync(Order order)
    {
        _storage[$"{order.TenantId}:{order.Id}"] = order;
        return ValueTask.FromResult(order);
    }
}
