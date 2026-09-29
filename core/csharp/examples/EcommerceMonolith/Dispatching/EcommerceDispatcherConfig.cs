using ClrinfCS.Core;
using EcommerceMonolith.Catalog;
using EcommerceMonolith.Common.Functional;
using EcommerceMonolith.Domain.Common.Exceptions;
using EcommerceMonolith.Inventory;
using EcommerceMonolith.Orders;
using EcommerceMonolith.Security;

namespace EcommerceMonolith.Dispatching;

public static class EcommerceDispatcherConfig
{
    public static IDispatcher BuildNativeDispatcher(
        ICatalogRepository catalogRepo,
        IInventoryRepository inventoryRepo,
        IOrderRepository orderRepo,
        MultiTenantCedarAuthorizer authorizer,
        AuthzClaims userClaims)
    {
        var builder = new Dispatcher.Builder();

        // 1. Catalog Handlers & Rules
        builder.Register(
            new CreateProductHandler(catalogRepo),
            [
                new EnsureProductPricePositiveRule(),
                new EnsureSkuUniqueRule(catalogRepo)
            ],
            new CedarAuthorizationPipelineBehavior<CreateProductCommand, Result<Product, DomainError>>(authorizer, userClaims)
        );

        builder.Register(
            new GetProductHandler(catalogRepo),
            new CedarAuthorizationPipelineBehavior<GetProductQuery, Result<Product, DomainError>>(authorizer, userClaims)
        );

        // 2. Inventory Handlers & Rules
        builder.Register(
            new SetStockHandler(inventoryRepo),
            new CedarAuthorizationPipelineBehavior<SetStockCommand, Result<StockItem, DomainError>>(authorizer, userClaims)
        );

        builder.Register(
            new ReserveStockHandler(inventoryRepo),
            [
                new EnsurePositiveQuantityRule(),
                new EnsureStockAvailabilityRule(inventoryRepo)
            ],
            new CedarAuthorizationPipelineBehavior<ReserveStockCommand, Result<StockItem, DomainError>>(authorizer, userClaims)
        );

        builder.Register(
            new ReleaseStockHandler(inventoryRepo),
            new CedarAuthorizationPipelineBehavior<ReleaseStockCommand, Result<StockItem, DomainError>>(authorizer, userClaims)
        );

        builder.Register(
            new CommitStockHandler(inventoryRepo),
            new CedarAuthorizationPipelineBehavior<CommitStockCommand, Result<StockItem, DomainError>>(authorizer, userClaims)
        );

        builder.Register(
            new GetStockHandler(inventoryRepo),
            new CedarAuthorizationPipelineBehavior<GetStockQuery, Result<StockItem, DomainError>>(authorizer, userClaims)
        );

        // 3. Orders Handlers & Rules
        builder.Register(
            new CreateOrderHandler(orderRepo),
            [
                new EnsureOrderItemsNotEmptyRule(),
                new MinimumOrderAmountRule()
            ],
            new CedarAuthorizationPipelineBehavior<CreateOrderCommand, Result<Order, DomainError>>(authorizer, userClaims)
        );

        builder.Register(
            new CancelOrderHandler(orderRepo),
            [
                new CannotCancelCompletedOrderRule(orderRepo)
            ],
            new CedarAuthorizationPipelineBehavior<CancelOrderCommand, Result<Order, DomainError>>(authorizer, userClaims)
        );

        builder.Register(
            new CompleteOrderHandler(orderRepo),
            new CedarAuthorizationPipelineBehavior<CompleteOrderCommand, Result<Order, DomainError>>(authorizer, userClaims)
        );

        builder.Register(
            new GetOrderHandler(orderRepo),
            new CedarAuthorizationPipelineBehavior<GetOrderQuery, Result<Order, DomainError>>(authorizer, userClaims)
        );

        return builder.Build();
    }
}
