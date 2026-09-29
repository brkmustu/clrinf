namespace CrmMonolith.Modules.Deal;

using Microsoft.Extensions.DependencyInjection;

public static class DealModule
{
    public static IServiceCollection AddDealModule(this IServiceCollection services)
    {
        // Register module handlers, validators, and rules here
        return services;
    }
}
