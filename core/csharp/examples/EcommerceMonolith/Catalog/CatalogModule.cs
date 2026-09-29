using Microsoft.Extensions.DependencyInjection;

namespace EcommerceMonolith.Catalog;

public static class CatalogModule
{
    public static IServiceCollection AddCatalogModule(this IServiceCollection services)
    {
        services.AddSingleton<ICatalogRepository, InMemoryCatalogRepository>();
        services.AddTransient<CreateProductHandler>();
        services.AddTransient<GetProductHandler>();
        services.AddTransient<EnsureProductPricePositiveRule>();
        services.AddTransient<EnsureSkuUniqueRule>();
        return services;
    }
}
