namespace CrmMonolith.Modules.Activity;

using Microsoft.Extensions.DependencyInjection;

public static class ActivityModule
{
    public static IServiceCollection AddActivityModule(this IServiceCollection services)
    {
        // Register module handlers, validators, and rules here
        return services;
    }
}
