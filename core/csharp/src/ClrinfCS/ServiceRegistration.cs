using Microsoft.Extensions.DependencyInjection;

namespace ClrinfCS;

public static class ServiceRegistration
{
    public static IServiceCollection AddClrinfCSServices(this IServiceCollection services)
    {
        // Roslyn AST linter, migration engine, and MCP services
        return services;
    }
}

