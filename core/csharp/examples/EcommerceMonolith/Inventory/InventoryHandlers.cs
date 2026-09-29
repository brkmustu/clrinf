using ClrinfCS.Core;
using EcommerceMonolith.Common.Functional;
using EcommerceMonolith.Domain.Common.Exceptions;

namespace EcommerceMonolith.Inventory;

public sealed class SetStockHandler(IInventoryRepository repository) : IRequestHandler<SetStockCommand, Result<StockItem, DomainError>>
{
    public async ValueTask<Result<StockItem, DomainError>> HandleAsync(SetStockCommand request, RequestContext context, CancellationToken cancellationToken)
    {
        var item = new StockItem(
            ProductId: request.ProductId,
            TenantId: request.TenantId,
            AvailableQuantity: request.InitialQuantity,
            ReservedQuantity: 0
        );
        var saved = await repository.SaveAsync(item);
        return Result<StockItem, DomainError>.Success(saved);
    }
}

public sealed class ReserveStockHandler(IInventoryRepository repository) : IRequestHandler<ReserveStockCommand, Result<StockItem, DomainError>>
{
    public async ValueTask<Result<StockItem, DomainError>> HandleAsync(ReserveStockCommand request, RequestContext context, CancellationToken cancellationToken)
    {
        var item = await repository.GetByProductIdAsync(request.TenantId, request.ProductId);
        if (item is null)
        {
            return Result<StockItem, DomainError>.Failure(new DomainError("StockItem.NotFound", $"Stock item for product '{request.ProductId}' was not found."));
        }

        var updated = item with
        {
            AvailableQuantity = item.AvailableQuantity - request.Quantity,
            ReservedQuantity = item.ReservedQuantity + request.Quantity
        };
        var saved = await repository.SaveAsync(updated);
        return Result<StockItem, DomainError>.Success(saved);
    }
}

public sealed class ReleaseStockHandler(IInventoryRepository repository) : IRequestHandler<ReleaseStockCommand, Result<StockItem, DomainError>>
{
    public async ValueTask<Result<StockItem, DomainError>> HandleAsync(ReleaseStockCommand request, RequestContext context, CancellationToken cancellationToken)
    {
        var item = await repository.GetByProductIdAsync(request.TenantId, request.ProductId);
        if (item is null)
        {
            return Result<StockItem, DomainError>.Failure(new DomainError("StockItem.NotFound", $"Stock item for product '{request.ProductId}' was not found."));
        }

        var releaseQty = Math.Min(item.ReservedQuantity, request.Quantity);
        var updated = item with
        {
            AvailableQuantity = item.AvailableQuantity + releaseQty,
            ReservedQuantity = item.ReservedQuantity - releaseQty
        };
        var saved = await repository.SaveAsync(updated);
        return Result<StockItem, DomainError>.Success(saved);
    }
}

public sealed class CommitStockHandler(IInventoryRepository repository) : IRequestHandler<CommitStockCommand, Result<StockItem, DomainError>>
{
    public async ValueTask<Result<StockItem, DomainError>> HandleAsync(CommitStockCommand request, RequestContext context, CancellationToken cancellationToken)
    {
        var item = await repository.GetByProductIdAsync(request.TenantId, request.ProductId);
        if (item is null)
        {
            return Result<StockItem, DomainError>.Failure(new DomainError("StockItem.NotFound", $"Stock item for product '{request.ProductId}' was not found."));
        }

        var commitQty = Math.Min(item.ReservedQuantity, request.Quantity);
        var updated = item with
        {
            ReservedQuantity = item.ReservedQuantity - commitQty
        };
        var saved = await repository.SaveAsync(updated);
        return Result<StockItem, DomainError>.Success(saved);
    }
}

public sealed class GetStockHandler(IInventoryRepository repository) : IRequestHandler<GetStockQuery, Result<StockItem, DomainError>>
{
    public async ValueTask<Result<StockItem, DomainError>> HandleAsync(GetStockQuery request, RequestContext context, CancellationToken cancellationToken)
    {
        var item = await repository.GetByProductIdAsync(request.TenantId, request.ProductId);
        if (item is null)
        {
            return Result<StockItem, DomainError>.Failure(new DomainError("StockItem.NotFound", $"Stock item for product '{request.ProductId}' was not found."));
        }
        return Result<StockItem, DomainError>.Success(item);
    }
}
