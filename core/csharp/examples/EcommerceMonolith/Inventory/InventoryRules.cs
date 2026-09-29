using ClrinfCS.Core;

namespace EcommerceMonolith.Inventory;

public sealed class EnsurePositiveQuantityRule : IBusinessRule<ReserveStockCommand>
{
    public ValueTask<RuleResult> EvaluateAsync(ReserveStockCommand command, RequestContext context, CancellationToken cancellationToken = default)
    {
        if (command.Quantity <= 0)
        {
            return ValueTask.FromResult(RuleResult.Failed("INVALID_QUANTITY", $"Requested quantity must be positive. Received: {command.Quantity}"));
        }
        return ValueTask.FromResult(RuleResult.Success());
    }
}

public sealed class EnsureStockAvailabilityRule(IInventoryRepository repository) : IBusinessRule<ReserveStockCommand>
{
    public async ValueTask<RuleResult> EvaluateAsync(ReserveStockCommand command, RequestContext context, CancellationToken cancellationToken = default)
    {
        var item = await repository.GetByProductIdAsync(command.TenantId, command.ProductId);
        if (item is null || item.AvailableQuantity < command.Quantity)
        {
            var avail = item?.AvailableQuantity ?? 0;
            return RuleResult.Failed("STOCK_INSUFFICIENT", $"Insufficient stock for product '{command.ProductId}'. Available: {avail}, Requested: {command.Quantity}");
        }
        return RuleResult.Success();
    }
}
