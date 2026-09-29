using Microsoft.Extensions.DependencyInjection;

namespace EcommerceMonolith.Orders;

public static class OrdersModule
{
    public static IServiceCollection AddOrdersModule(this IServiceCollection services)
    {
        services.AddSingleton<IOrderRepository, InMemoryOrderRepository>();
        services.AddTransient<CreateOrderHandler>();
        services.AddTransient<CancelOrderHandler>();
        services.AddTransient<CompleteOrderHandler>();
        services.AddTransient<GetOrderHandler>();
        services.AddTransient<EnsureOrderItemsNotEmptyRule>();
        services.AddTransient<MinimumOrderAmountRule>();
        services.AddTransient<CannotCancelCompletedOrderRule>();
        return services;
    }
}
