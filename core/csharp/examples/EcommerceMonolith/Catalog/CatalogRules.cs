using ClrinfCS.Core;

namespace EcommerceMonolith.Catalog;

public sealed class EnsureProductPricePositiveRule : IBusinessRule<CreateProductCommand>
{
    public ValueTask<RuleResult> EvaluateAsync(CreateProductCommand command, RequestContext context, CancellationToken cancellationToken = default)
    {
        if (command.Price <= 0)
        {
            return ValueTask.FromResult(RuleResult.Failed("INVALID_PRODUCT_PRICE", $"Product price must be greater than zero. Received: {command.Price}"));
        }
        return ValueTask.FromResult(RuleResult.Success());
    }
}

public sealed class EnsureSkuUniqueRule(ICatalogRepository repository) : IBusinessRule<CreateProductCommand>
{
    public async ValueTask<RuleResult> EvaluateAsync(CreateProductCommand command, RequestContext context, CancellationToken cancellationToken = default)
    {
        var existing = await repository.GetBySkuAsync(command.TenantId, command.Sku);
        if (existing is not null)
        {
            return RuleResult.Failed("DUPLICATE_SKU", $"Product with SKU '{command.Sku}' already exists in tenant '{command.TenantId}'.");
        }
        return RuleResult.Success();
    }
}
