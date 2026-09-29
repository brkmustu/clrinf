using ClrinfCS.Core;
using EcommerceMonolith.Common.Functional;
using EcommerceMonolith.Domain.Common.Exceptions;

namespace EcommerceMonolith.Orders;

public sealed class CreateOrderHandler(IOrderRepository repository) : IRequestHandler<CreateOrderCommand, Result<Order, DomainError>>
{
    public async ValueTask<Result<Order, DomainError>> HandleAsync(CreateOrderCommand request, RequestContext context, CancellationToken cancellationToken)
    {
        var total = request.Items.Sum(i => i.Quantity * i.UnitPrice);
        var order = new Order(
            Id: Guid.NewGuid().ToString("N")[..8],
            TenantId: request.TenantId,
            CustomerId: request.CustomerId,
            Items: request.Items,
            TotalAmount: total,
            Status: "Pending"
        );
        var saved = await repository.SaveAsync(order);
        return Result<Order, DomainError>.Success(saved);
    }
}

public sealed class CancelOrderHandler(IOrderRepository repository) : IRequestHandler<CancelOrderCommand, Result<Order, DomainError>>
{
    public async ValueTask<Result<Order, DomainError>> HandleAsync(CancelOrderCommand request, RequestContext context, CancellationToken cancellationToken)
    {
        var order = await repository.GetByIdAsync(request.TenantId, request.OrderId);
        if (order is null)
        {
            return Result<Order, DomainError>.Failure(new DomainError("Order.NotFound", $"Order '{request.OrderId}' was not found."));
        }

        var updated = order with { Status = "Cancelled" };
        var saved = await repository.SaveAsync(updated);
        return Result<Order, DomainError>.Success(saved);
    }
}

public sealed class CompleteOrderHandler(IOrderRepository repository) : IRequestHandler<CompleteOrderCommand, Result<Order, DomainError>>
{
    public async ValueTask<Result<Order, DomainError>> HandleAsync(CompleteOrderCommand request, RequestContext context, CancellationToken cancellationToken)
    {
        var order = await repository.GetByIdAsync(request.TenantId, request.OrderId);
        if (order is null)
        {
            return Result<Order, DomainError>.Failure(new DomainError("Order.NotFound", $"Order '{request.OrderId}' was not found."));
        }

        var updated = order with { Status = "Completed" };
        var saved = await repository.SaveAsync(updated);
        return Result<Order, DomainError>.Success(saved);
    }
}

public sealed class GetOrderHandler(IOrderRepository repository) : IRequestHandler<GetOrderQuery, Result<Order, DomainError>>
{
    public async ValueTask<Result<Order, DomainError>> HandleAsync(GetOrderQuery request, RequestContext context, CancellationToken cancellationToken)
    {
        var order = await repository.GetByIdAsync(request.TenantId, request.OrderId);
        if (order is null)
        {
            return Result<Order, DomainError>.Failure(new DomainError("Order.NotFound", $"Order '{request.OrderId}' was not found."));
        }
        return Result<Order, DomainError>.Success(order);
    }
}
