using Microsoft.Extensions.DependencyInjection;

namespace EcommerceMonolith.Inventory;

public static class InventoryModule
{
    public static IServiceCollection AddInventoryModule(this IServiceCollection services)
    {
        services.AddSingleton<IInventoryRepository, InMemoryInventoryRepository>();
        services.AddTransient<SetStockHandler>();
        services.AddTransient<ReserveStockHandler>();
        services.AddTransient<ReleaseStockHandler>();
        services.AddTransient<CommitStockHandler>();
        services.AddTransient<GetStockHandler>();
        services.AddTransient<EnsurePositiveQuantityRule>();
        services.AddTransient<EnsureStockAvailabilityRule>();
        return services;
    }
}
