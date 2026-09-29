using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Routing;
using Microsoft.Extensions.DependencyInjection;

namespace ClrinfCS.OpenApi;

public static class ClrinfOpenApiExtensions
{
    public static IServiceCollection AddClrinfOpenApi(this IServiceCollection services, Action<OpenApiOptions>? configure = null)
    {
        var options = new OpenApiOptions();
        configure?.Invoke(options);

        services.AddSingleton(options);
        services.AddTransient<OpenApiGenerator>();

        return services;
    }

    public static IEndpointRouteBuilder MapClrinfOpenApi(this IEndpointRouteBuilder endpoints, string jsonPattern = "/openapi/v1.json")
    {
        endpoints.MapGet(jsonPattern, (OpenApiGenerator generator) =>
        {
            var json = generator.GenerateJson();
            return Results.Content(json, "application/json; charset=utf-8");
        });

        RequestDelegate serveUi = async (HttpContext context) =>
        {
            var assembly = typeof(ClrinfOpenApiExtensions).Assembly;
            using var stream = assembly.GetManifestResourceStream("ClrinfCS.OpenApi.api-docs.html");
            if (stream != null)
            {
                context.Response.ContentType = "text/html; charset=utf-8";
                await stream.CopyToAsync(context.Response.Body);
            }
            else
            {
                context.Response.StatusCode = 404;
                await context.Response.WriteAsync("API documentation UI resource not found.");
            }
        };

        endpoints.MapGet("/dsi-arch/api", serveUi);
        endpoints.MapGet("/swagger", serveUi);
        endpoints.MapGet("/api-docs", serveUi);

        return endpoints;
    }
}
