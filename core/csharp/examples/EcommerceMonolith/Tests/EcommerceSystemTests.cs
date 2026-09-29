using ClrinfCS.Core;
using EcommerceMonolith.Catalog;
using EcommerceMonolith.Common.Functional;
using EcommerceMonolith.Domain.Common.Exceptions;
using EcommerceMonolith.Dispatching;
using EcommerceMonolith.Inventory;
using EcommerceMonolith.Orders;
using EcommerceMonolith.Security;
using Xunit;

namespace EcommerceMonolith.Tests;

public sealed class EcommerceSystemTests
{
    private readonly ICatalogRepository _catalogRepo = new InMemoryCatalogRepository();
    private readonly IInventoryRepository _inventoryRepo = new InMemoryInventoryRepository();
    private readonly IOrderRepository _orderRepo = new InMemoryOrderRepository();
    private readonly MultiTenantCedarAuthorizer _authorizer = EcommerceCedarFactory.CreateAuthorizer();

    private IDispatcher CreateDispatcherForUser(string tenantId, params string[] roles)
    {
        var claims = new AuthzClaims("user_test", roles, tenantId);
        return EcommerceDispatcherConfig.BuildNativeDispatcher(_catalogRepo, _inventoryRepo, _orderRepo, _authorizer, claims);
    }

    [Fact]
    public async Task CreateProduct_WithPositivePrice_Succeeds()
    {
        var dispatcher = CreateDispatcherForUser("tenant_acme", "StoreAdmin");
        var context = new RequestContext("tenant_acme", "corr_1", "cause_1");

        var cmd = new CreateProductCommand("tenant_acme", "SKU-KEYBOARD", "Mechanical Keyboard", 120.0m);
        var res = await dispatcher.SendAsync(cmd, context);

        Assert.True(res.IsSuccess);
        Assert.Equal("SKU-KEYBOARD", res.Value.Sku);
        Assert.Equal(120.0m, res.Value.Price);
        Assert.Equal("Active", res.Value.Status);
    }

    [Fact]
    public async Task CreateProduct_WithNegativeOrZeroPrice_FailsRule()
    {
        var dispatcher = CreateDispatcherForUser("tenant_acme", "StoreAdmin");
        var context = new RequestContext("tenant_acme", "corr_2", "cause_2");

        var cmd = new CreateProductCommand("tenant_acme", "SKU-FREE", "Zero Price Item", 0.0m);
        var ex = await Assert.ThrowsAsync<BusinessRuleViolationException>(() => dispatcher.SendAsync(cmd, context).AsTask());

        Assert.Equal("INVALID_PRODUCT_PRICE", ex.ErrorCode);
    }

    [Fact]
    public async Task CreateProduct_WithDuplicateSku_FailsRule()
    {
        var dispatcher = CreateDispatcherForUser("tenant_acme", "StoreAdmin");
        var context = new RequestContext("tenant_acme", "corr_3", "cause_3");

        var cmd1 = new CreateProductCommand("tenant_acme", "SKU-MOUSE", "Gaming Mouse", 60.0m);
        var res1 = await dispatcher.SendAsync(cmd1, context);
        Assert.True(res1.IsSuccess);

        var cmd2 = new CreateProductCommand("tenant_acme", "SKU-MOUSE", "Office Mouse", 40.0m);
        var ex = await Assert.ThrowsAsync<BusinessRuleViolationException>(() => dispatcher.SendAsync(cmd2, context).AsTask());

        Assert.Equal("DUPLICATE_SKU", ex.ErrorCode);
    }

    [Fact]
    public async Task GetProduct_NotFound_ReturnsFunctionalDomainError()
    {
        var dispatcher = CreateDispatcherForUser("tenant_acme", "Customer");
        var context = new RequestContext("tenant_acme", "corr_np", "cause_np");

        var query = new GetProductQuery("tenant_acme", "non_existent_prod");
        var res = await dispatcher.SendAsync(query, context);

        Assert.True(res.IsFailure);
        Assert.Equal("Product.NotFound", res.Error.Code);
    }

    [Fact]
    public async Task ReserveStock_WhenAvailable_Succeeds()
    {
        var dispatcher = CreateDispatcherForUser("tenant_acme", "StoreAdmin");
        var context = new RequestContext("tenant_acme", "corr_4", "cause_4");

        // 1. Set initial stock to 10
        var setRes = await dispatcher.SendAsync(new SetStockCommand("tenant_acme", "prod_1", 10), context);
        Assert.True(setRes.IsSuccess);

        // 2. Reserve 3 items
        var customerDispatcher = CreateDispatcherForUser("tenant_acme", "Customer");
        var reserveRes = await customerDispatcher.SendAsync(new ReserveStockCommand("tenant_acme", "prod_1", 3), context);

        Assert.True(reserveRes.IsSuccess);
        Assert.Equal(7, reserveRes.Value.AvailableQuantity);
        Assert.Equal(3, reserveRes.Value.ReservedQuantity);
    }

    [Fact]
    public async Task ReserveStock_WhenInsufficient_FailsRule()
    {
        var dispatcher = CreateDispatcherForUser("tenant_acme", "StoreAdmin");
        var context = new RequestContext("tenant_acme", "corr_5", "cause_5");

        await dispatcher.SendAsync(new SetStockCommand("tenant_acme", "prod_2", 2), context);

        var customerDispatcher = CreateDispatcherForUser("tenant_acme", "Customer");
        var ex = await Assert.ThrowsAsync<BusinessRuleViolationException>(() =>
            customerDispatcher.SendAsync(new ReserveStockCommand("tenant_acme", "prod_2", 5), context).AsTask());

        Assert.Equal("STOCK_INSUFFICIENT", ex.ErrorCode);
    }

    [Fact]
    public async Task ReleaseStock_NotFound_ReturnsFunctionalDomainError()
    {
        var customerDispatcher = CreateDispatcherForUser("tenant_acme", "Customer");
        var context = new RequestContext("tenant_acme", "corr_snf", "cause_snf");

        var res = await customerDispatcher.SendAsync(new ReleaseStockCommand("tenant_acme", "non_existent_prod", 1), context);
        Assert.True(res.IsFailure);
        Assert.Equal("StockItem.NotFound", res.Error.Code);
    }

    [Fact]
    public async Task CreateOrder_ValidItemsAndTotal_Succeeds()
    {
        var customerDispatcher = CreateDispatcherForUser("tenant_acme", "Customer");
        var context = new RequestContext("tenant_acme", "corr_6", "cause_6");

        var items = new List<OrderItem>
        {
            new("prod_kb", 1, 120.0m)
        };

        var res = await customerDispatcher.SendAsync(new CreateOrderCommand("tenant_acme", "customer_1", items), context);

        Assert.True(res.IsSuccess);
        Assert.Equal("Pending", res.Value.Status);
        Assert.Equal(120.0m, res.Value.TotalAmount);
    }

    [Fact]
    public async Task CreateOrder_BelowMinimumAmount_FailsRule()
    {
        var customerDispatcher = CreateDispatcherForUser("tenant_acme", "Customer");
        var context = new RequestContext("tenant_acme", "corr_7", "cause_7");

        var items = new List<OrderItem>
        {
            new("prod_sticker", 1, 5.0m) // Total 5.0 < Min 25.0
        };

        var ex = await Assert.ThrowsAsync<BusinessRuleViolationException>(() =>
            customerDispatcher.SendAsync(new CreateOrderCommand("tenant_acme", "customer_1", items), context).AsTask());

        Assert.Equal("MINIMUM_ORDER_AMOUNT_NOT_MET", ex.ErrorCode);
    }

    [Fact]
    public async Task CreateOrder_EmptyItems_FailsRule()
    {
        var customerDispatcher = CreateDispatcherForUser("tenant_acme", "Customer");
        var context = new RequestContext("tenant_acme", "corr_8", "cause_8");

        var ex = await Assert.ThrowsAsync<BusinessRuleViolationException>(() =>
            customerDispatcher.SendAsync(new CreateOrderCommand("tenant_acme", "customer_1", new List<OrderItem>()), context).AsTask());

        Assert.Equal("EMPTY_ORDER_ITEMS", ex.ErrorCode);
    }

    [Fact]
    public async Task OrderCancellation_And_StockRelease_Lifecycle()
    {
        var adminDispatcher = CreateDispatcherForUser("tenant_acme", "StoreAdmin");
        var customerDispatcher = CreateDispatcherForUser("tenant_acme", "Customer");
        var context = new RequestContext("tenant_acme", "corr_9", "cause_9");

        // 1. Initial stock: 10
        var setRes = await adminDispatcher.SendAsync(new SetStockCommand("tenant_acme", "prod_headset", 10), context);
        Assert.True(setRes.IsSuccess);

        // 2. Reserve 2 items
        var stockAfterReserve = await customerDispatcher.SendAsync(new ReserveStockCommand("tenant_acme", "prod_headset", 2), context);
        Assert.True(stockAfterReserve.IsSuccess);
        Assert.Equal(8, stockAfterReserve.Value.AvailableQuantity);
        Assert.Equal(2, stockAfterReserve.Value.ReservedQuantity);

        // 3. Create order
        var orderRes = await customerDispatcher.SendAsync(new CreateOrderCommand(
            "tenant_acme",
            "customer_1",
            [new("prod_headset", 2, 50.0m)]
        ), context);
        Assert.True(orderRes.IsSuccess);
        Assert.Equal("Pending", orderRes.Value.Status);

        // 4. Cancel order
        var cancelledRes = await customerDispatcher.SendAsync(new CancelOrderCommand("tenant_acme", orderRes.Value.Id), context);
        Assert.True(cancelledRes.IsSuccess);
        Assert.Equal("Cancelled", cancelledRes.Value.Status);

        // 5. Compensating action: Release reserved stock
        var releasedRes = await customerDispatcher.SendAsync(new ReleaseStockCommand("tenant_acme", "prod_headset", 2), context);
        Assert.True(releasedRes.IsSuccess);
        Assert.Equal(10, releasedRes.Value.AvailableQuantity);
        Assert.Equal(0, releasedRes.Value.ReservedQuantity);
    }

    [Fact]
    public async Task CancelOrder_NotFound_ReturnsFunctionalDomainError()
    {
        var customerDispatcher = CreateDispatcherForUser("tenant_acme", "Customer");
        var context = new RequestContext("tenant_acme", "corr_onf", "cause_onf");

        var res = await customerDispatcher.SendAsync(new CancelOrderCommand("tenant_acme", "non_existent_order"), context);
        Assert.True(res.IsFailure);
        Assert.Equal("Order.NotFound", res.Error.Code);
    }

    [Fact]
    public async Task MultiTenantIsolation_CrossTenantAccessDenied()
    {
        var tenantBDispatcher = CreateDispatcherForUser("tenant_beta", "Customer");

        // User from tenant_beta attempts to access tenant_acme's order
        var context = new RequestContext("tenant_beta", "corr_10", "cause_10");
        var cmd = new CreateOrderCommand("tenant_acme", "customer_1", [new("p1", 1, 50.0m)]);

        var ex = await Assert.ThrowsAsync<CedarAuthException>(() => tenantBDispatcher.SendAsync(cmd, context).AsTask());
        Assert.True(ex.IsTenantViolation);
    }
}
