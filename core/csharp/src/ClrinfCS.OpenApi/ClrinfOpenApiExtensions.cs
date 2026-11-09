using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Mvc.Infrastructure;
using Microsoft.AspNetCore.Routing;
using Microsoft.Extensions.DependencyInjection;

namespace ClrinfCS.OpenApi;

internal sealed class ClrinfEndpointDataSourceRegistry
{
    public IEnumerable<EndpointDataSource>? DataSources { get; set; }
}

public static class ClrinfOpenApiExtensions
{
    public static IServiceCollection AddClrinfOpenApi(this IServiceCollection services, Action<OpenApiOptions>? configure = null)
    {
        var options = new OpenApiOptions();
        configure?.Invoke(options);

        services.AddSingleton(options);
        services.AddSingleton<ClrinfEndpointDataSourceRegistry>();
        services.AddTransient<OpenApiGenerator>(sp =>
        {
            var registry = sp.GetService<ClrinfEndpointDataSourceRegistry>();
            var sources = registry?.DataSources ?? sp.GetServices<EndpointDataSource>();
            return new OpenApiGenerator(
                sp.GetService<IActionDescriptorCollectionProvider>(),
                sources,
                sp.GetRequiredService<OpenApiOptions>());
        });

        return services;
    }

    public static IEndpointRouteBuilder MapClrinfOpenApi(this IEndpointRouteBuilder endpoints, string jsonPattern = "/openapi/v1.json")
    {
        var registry = endpoints.ServiceProvider.GetService<ClrinfEndpointDataSourceRegistry>();
        if (registry is not null)
        {
            registry.DataSources = endpoints.DataSources;
        }

        RequestDelegate serveJson = async (HttpContext context) =>
        {
            var generator = context.RequestServices.GetRequiredService<OpenApiGenerator>();
            var json = generator.GenerateJson();
            context.Response.ContentType = "application/json; charset=utf-8";
            await context.Response.WriteAsync(json, context.RequestAborted);
        };

        endpoints.MapGet(jsonPattern, serveJson).ExcludeFromDescription();

        RequestDelegate serveUi = async (HttpContext context) =>
        {
            var assembly = typeof(ClrinfOpenApiExtensions).Assembly;
            using var stream = assembly.GetManifestResourceStream("ClrinfCS.OpenApi.api-docs.html");
            if (stream != null)
            {
                context.Response.ContentType = "text/html; charset=utf-8";
                await stream.CopyToAsync(context.Response.Body, context.RequestAborted);
            }
            else
            {
                context.Response.StatusCode = 404;
                await context.Response.WriteAsync("API documentation UI resource not found.", context.RequestAborted);
            }
        };

        endpoints.MapGet("/dsi-arch/api", serveUi).ExcludeFromDescription();
        endpoints.MapGet("/swagger", serveUi).ExcludeFromDescription();
        endpoints.MapGet("/api-docs", serveUi).ExcludeFromDescription();

        return endpoints;
    }
}
