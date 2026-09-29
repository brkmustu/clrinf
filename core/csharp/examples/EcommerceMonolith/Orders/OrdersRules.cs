using ClrinfCS.Core;

namespace EcommerceMonolith.Orders;

public sealed class EnsureOrderItemsNotEmptyRule : IBusinessRule<CreateOrderCommand>
{
    public ValueTask<RuleResult> EvaluateAsync(CreateOrderCommand command, RequestContext context, CancellationToken cancellationToken = default)
    {
        if (command.Items == null || command.Items.Count == 0)
        {
            return ValueTask.FromResult(RuleResult.Failed("EMPTY_ORDER_ITEMS", "Order must contain at least one item."));
        }
        return ValueTask.FromResult(RuleResult.Success());
    }
}

public sealed class MinimumOrderAmountRule : IBusinessRule<CreateOrderCommand>
{
    private const decimal MinThreshold = 25.0m;

    public ValueTask<RuleResult> EvaluateAsync(CreateOrderCommand command, RequestContext context, CancellationToken cancellationToken = default)
    {
        var total = command.Items.Sum(i => i.Quantity * i.UnitPrice);
        if (total < MinThreshold)
        {
            return ValueTask.FromResult(RuleResult.Failed(
                "MINIMUM_ORDER_AMOUNT_NOT_MET",
                $"Order total {total:C} is below minimum required threshold {MinThreshold:C}."
            ));
        }
        return ValueTask.FromResult(RuleResult.Success());
    }
}

public sealed class CannotCancelCompletedOrderRule(IOrderRepository repository) : IBusinessRule<CancelOrderCommand>
{
    public async ValueTask<RuleResult> EvaluateAsync(CancelOrderCommand command, RequestContext context, CancellationToken cancellationToken = default)
    {
        var order = await repository.GetByIdAsync(command.TenantId, command.OrderId);
        if (order is not null && order.Status == "Completed")
        {
            return RuleResult.Failed("INVALID_ORDER_STATE", $"Cannot cancel completed order '{command.OrderId}'.");
        }
        return RuleResult.Success();
    }
}
